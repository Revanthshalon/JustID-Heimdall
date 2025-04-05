use std::collections::HashMap;

use async_trait::async_trait;
use sqlx::PgPool;
use tracing::{info_span, trace};
use uuid::Uuid;

use crate::{
    context::RequestContext, error::HeimdallResult, models::query::TokenPagination,
    persistance::schema::UuidMapping,
};

use super::traits::UuidMappingManager;

pub struct UuidMappingService {
    pool: PgPool,
}

const CHUNK_SIZE_INSERT_UUID_MAPPINGS: usize = 15000;

#[allow(unused)]
impl UuidMappingService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn batch_from_uuids(
        &self,
        _ctx: &RequestContext,
        ids: &[Uuid],
        pagination_params: &TokenPagination,
    ) -> HeimdallResult<Vec<String>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        trace!("looking up UUIDS");

        let page_size = pagination_params.page_size.unwrap_or(100) as usize;

        let mut id_idx: HashMap<Uuid, Vec<usize>> = HashMap::with_capacity(ids.len());

        for (i, id) in ids.iter().enumerate() {
            id_idx.entry(*id).or_default().push(i)
        }

        let mut results = vec![String::new(); ids.len()];

        let keys: Vec<&Uuid> = id_idx.keys().collect();

        for id_chunk in keys.chunks(page_size) {
            let id_params = id_chunk.iter().copied().cloned().collect::<Vec<Uuid>>();

            let uuid_mapping_result: Vec<UuidMapping> = sqlx::query_as(
                "SELECT id, string_representation FROM heimdall_uuid_mappings WHERE id = ANY($1)",
            )
            .bind(id_params)
            .fetch_all(&self.pool)
            .await?;

            for row in uuid_mapping_result {
                if let (Some(indices), ref string_representation) =
                    (id_idx.get(&row.id), row.string_representation)
                {
                    for idx in indices {
                        unsafe {
                            /*
                             * SAFETY: We have a list of uuids which we are using to fix the
                             * capacity of the vector. This vector is supposed to have a string
                             * representation of each element (uuids) from the db. So this should be
                             * safe
                             */
                            *results.get_unchecked_mut(*idx) = string_representation.clone();
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    async fn insert_uuids(&self, values: &[UuidMapping]) -> HeimdallResult<()> {
        let mut ids = Vec::with_capacity(values.len());
        let mut string_reps = Vec::with_capacity(values.len());
        for value in values {
            ids.push(value.id);
            string_reps.push(value.string_representation.clone());
        }
        sqlx::query(
            "INSERT INTO heimdall_uuid_mappings (id, string_representation) SELECT * FROM UNNEST($1::UUID[], $2::VARCHAR[]) ON CONFLICT (id) DO NOTHING"
        )
            .bind(ids)
            .bind(string_reps)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

#[async_trait]
impl UuidMappingManager for UuidMappingService {
    async fn map_strings_to_uuids(
        &self,
        ctx: &RequestContext,
        values: &[String],
    ) -> HeimdallResult<Vec<Uuid>> {
        if values.is_empty() {
            return Ok(Vec::new());
        }

        let span = info_span!("map_strings_to_uuids");
        let _guard = span.enter();

        let ids = self.map_strings_to_uuids_readonly(ctx, values).await?;

        trace!(values = ?values, ids = ?ids, "adding UUID mappings");

        let mut mappings = Vec::with_capacity(values.len());

        for (string_representation, id) in values.iter().zip(ids.iter()) {
            mappings.push(UuidMapping {
                id: *id,
                string_representation: string_representation.clone(),
            });
        }

        mappings.sort_by(|a, b| a.id.cmp(&b.id));
        mappings.dedup_by(|a, b| a.id.eq(&b.id));

        span.record("mappings_length", mappings.len());

        for mapping in mappings.chunks(CHUNK_SIZE_INSERT_UUID_MAPPINGS) {
            self.insert_uuids(mapping).await?;
        }

        Ok(ids)
    }

    async fn map_strings_to_uuids_readonly(
        &self,
        ctx: &RequestContext,
        values: &[String],
    ) -> HeimdallResult<Vec<Uuid>> {
        let result = values
            .iter()
            .map(|s| Uuid::new_v5(ctx.network_id(), s.as_bytes()))
            .collect::<Vec<Uuid>>();
        Ok(result)
    }

    async fn map_uuids_to_strings(
        &self,
        ctx: &RequestContext,
        ids: &[Uuid],
        pagination_params: &TokenPagination,
    ) -> HeimdallResult<Vec<String>> {
        let span = info_span!("map_uuids_to_strings");
        let _guard = span.enter();

        self.batch_from_uuids(ctx, ids, pagination_params).await
    }
}
