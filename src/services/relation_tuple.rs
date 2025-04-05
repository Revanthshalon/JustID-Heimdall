use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, QueryBuilder};
use tracing::info_span;
use uuid::Uuid;

use crate::{
    context::RequestContext,
    error::{HeimdallError, HeimdallResult},
    models::{
        query::{TokenPagination, relation_tuple::RelationTupleQuery},
        relation_tuple::{RelationTuple, Subject, SubjectID, SubjectSet},
        response::PaginatedResponse,
    },
    persistance::schema::RelationTuple as DbRelationTuple,
};

use super::traits::RelationTupleManager;

#[derive(Debug)]
pub struct RelationTupleService {
    pool: PgPool,
}

const CHUNK_SIZE_INSERT_TUPLE: usize = 3000;
const CHUNK_SIZE_DELETE_TUPLE: usize = 100;

impl RelationTupleService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn with_network<'a>(builder: &mut QueryBuilder<'a, Postgres>, ctx: &'a RequestContext) {
        builder.push(" nid = ");
        builder.push_bind(ctx.network_id());
    }

    fn with_query_filters<'a>(
        builder: &mut QueryBuilder<'a, Postgres>,
        rs_query: &'a RelationTupleQuery,
    ) {
        if let Some(ref namespace) = rs_query.namespace {
            builder.push(" AND namespace = ");
            builder.push_bind(namespace);
        }
        if let Some(ref object) = rs_query.object {
            builder.push(" AND object  = ");
            builder.push_bind(object);
        }
        if let Some(ref relation) = rs_query.relation {
            builder.push(" AND relation = ");
            builder.push_bind(relation);
        }
        if let Some(ref subject) = rs_query.subject {
            Self::with_subject_filters(builder, subject);
        }
    }

    fn with_subject_filters<'a>(builder: &mut QueryBuilder<'a, Postgres>, subject: &'a Subject) {
        match subject {
            Subject::Direct(SubjectID { id }) => {
                builder.push(" AND subject_id = ");
                builder.push_bind(*id);
                builder.push(" AND subject_set_namespace IS NULL AND subject_set_object IS NULL AND subject_set_relation IS NULL");
            }
            Subject::Set(SubjectSet {
                namespace,
                object,
                relation,
            }) => {
                builder.push(" AND subject_set_namespace = ");
                builder.push_bind(namespace);
                builder.push(" AND subject_set_object = ");
                builder.push_bind(*object);
                builder.push(" AND subject_set_relation = ");
                builder.push(relation);
                builder.push(" AND subject_id IS NULL");
            }
        }
    }
}

#[async_trait]
impl RelationTupleManager for RelationTupleService {
    async fn write_relation_tuples(
        &self,
        ctx: &RequestContext,
        rs: &[RelationTuple],
    ) -> HeimdallResult<()> {
        if rs.is_empty() {
            return Err(HeimdallError::MalformedInput);
        }

        let span = info_span!("write_relation_tuples", relation_tuple_count = rs.len());
        let _guard = span.enter();

        let commit_time = Utc::now();

        let mut tx = self.pool.begin().await?;

        for rs_chunk in rs.chunks(CHUNK_SIZE_INSERT_TUPLE) {
            let shard_ids: Vec<Uuid> = vec![Uuid::new_v4(); rs_chunk.len()];
            let nids = vec![*ctx.network_id(); rs_chunk.len()];
            let mut namespaces: Vec<String> = Vec::with_capacity(rs_chunk.len());
            let mut objects: Vec<Uuid> = Vec::with_capacity(rs_chunk.len());
            let mut relations: Vec<String> = Vec::with_capacity(rs_chunk.len());
            let mut subject_ids: Vec<Option<Uuid>> = Vec::with_capacity(rs_chunk.len());
            let mut subject_set_namespaces: Vec<Option<String>> =
                Vec::with_capacity(rs_chunk.len());
            let mut subject_set_objects: Vec<Option<Uuid>> = Vec::with_capacity(rs_chunk.len());
            let mut subject_set_relations: Vec<Option<String>> = Vec::with_capacity(rs_chunk.len());
            let commit_times: Vec<DateTime<Utc>> = vec![commit_time; rs_chunk.len()];
            for r in rs_chunk {
                namespaces.push(r.namespace.clone());
                objects.push(r.object);
                relations.push(r.relation.clone());
                match &r.subject {
                    Subject::Direct(SubjectID { id }) => {
                        subject_ids.push(Some(*id));
                        subject_set_namespaces.push(None);
                        subject_set_objects.push(None);
                        subject_set_relations.push(None);
                    }
                    Subject::Set(SubjectSet {
                        namespace,
                        object,
                        relation,
                    }) => {
                        subject_ids.push(None);
                        subject_set_namespaces.push(Some(namespace.clone()));
                        subject_set_objects.push(Some(*object));
                        subject_set_relations.push(Some(relation.clone()));
                    }
                }
            }

            sqlx::query(
                "INSERT INTO heimdall_relation_tuples 
                (shard_id, nid, namespace, object, relation, subject_id, subject_set_namespace, subject_set_object, subject_set_relation, commit_time)
                SELECT * FROM UNNEST 
                ($1::UUID[], $2::UUID[], $3::VARCHAR[], $4::UUID[], $5::VARCHAR[], $6::UUID[], $7, $8, $9 SELECT * FROM UNNEST ($1::UUID[], $2::UUID[], $3::VARCHAR[], $4::UUID[], $5::VARCHAR[], $6::UUID[], $7::VARCHAR[], $8::UUID[], $9::VARCHAR[], $10::TIMESTAMPTZ[])",
            )
                .bind(shard_ids)
                .bind(nids)
                .bind(namespaces)
                .bind(objects)
                .bind(relations)
                .bind(subject_ids)
                .bind(subject_set_namespaces)
                .bind(subject_set_objects)
                .bind(subject_set_relations)
                .bind(commit_times)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn get_relation_tuples(
        &self,
        ctx: &RequestContext,
        rs_query: &RelationTupleQuery,
        pagination_params: &TokenPagination,
    ) -> HeimdallResult<PaginatedResponse<Vec<RelationTuple>>> {
        let span = info_span!("get_relation_tuples");
        let _guard = span.enter();

        let limit = pagination_params.page_size.unwrap_or(100);

        let mut builder = QueryBuilder::new(
            "SELECT 
                shard_id,
                nid,
                namespace,
                object,
                relation,
                subject_id,
                subject_set_namespace,
                subject_set_object,
                subject_set_relation
                commit_time
            FROM heimdall_relation_tuples WHERE ",
        );
        Self::with_network(&mut builder, ctx);
        Self::with_query_filters(&mut builder, rs_query);
        builder.push(" AND shard_id > ");
        builder.push_bind(pagination_params.last_id);
        builder.push("ORDER BY shard_id LIMIT ");
        builder.push(limit + 1);

        let mut query_result: Vec<DbRelationTuple> =
            builder.build_query_as().fetch_all(&self.pool).await?;

        let next_page_token = if query_result.len() > limit as usize {
            let extra_row = query_result.pop().unwrap();
            TokenPagination::encode_next_page_token(&extra_row.shard_id)
        } else {
            Uuid::nil().to_string()
        };

        let response = PaginatedResponse {
            data: query_result.into_iter().map(Into::into).collect(),
            token: next_page_token,
        };

        Ok(response)
    }

    async fn exists_relation_tuples(
        &self,
        ctx: &RequestContext,
        rs_query: &RelationTupleQuery,
    ) -> HeimdallResult<bool> {
        let span = info_span!("exists_relation_tuples");
        let _guard = span.enter();

        let mut exists_query_builder =
            QueryBuilder::new("SELECT 1 FROM heimdall_relation_tuples WHERE");
        Self::with_network(&mut exists_query_builder, ctx);
        Self::with_query_filters(&mut exists_query_builder, rs_query);
        let mut builder = QueryBuilder::new("SELECT EXISTS (");
        builder.push(exists_query_builder.sql());
        builder.push(")");
        let exists: bool = builder.build_query_scalar().fetch_one(&self.pool).await?;
        Ok(exists)
    }

    async fn delete_relation_tuples(
        &self,
        ctx: &RequestContext,
        rs: &[RelationTuple],
    ) -> HeimdallResult<()> {
        if rs.is_empty() {
            return Ok(());
        }

        let span = info_span!("delete_relation_tuples", count = rs.len());
        let _guard = span.enter();

        let mut tx = self.pool.begin().await?;

        for rs_chunk in rs.chunks(CHUNK_SIZE_DELETE_TUPLE) {
            let mut namespaces: Vec<String> = Vec::with_capacity(rs_chunk.len());
            let mut objects: Vec<Uuid> = Vec::with_capacity(rs_chunk.len());
            let mut relations: Vec<String> = Vec::with_capacity(rs_chunk.len());
            let mut subject_ids: Vec<Option<Uuid>> = Vec::with_capacity(rs_chunk.len());
            let mut subject_set_namespaces: Vec<Option<String>> =
                Vec::with_capacity(rs_chunk.len());
            let mut subject_set_objects: Vec<Option<Uuid>> = Vec::with_capacity(rs_chunk.len());
            let mut subject_set_relations: Vec<Option<String>> = Vec::with_capacity(rs_chunk.len());
            let nids = vec![ctx.network_id(); rs_chunk.len()];

            for tuple in rs_chunk {
                namespaces.push(tuple.namespace.clone());
                objects.push(tuple.object);
                relations.push(tuple.relation.clone());
                match &tuple.subject {
                    Subject::Direct(SubjectID { id }) => {
                        subject_ids.push(Some(*id));
                        subject_set_namespaces.push(None);
                        subject_set_objects.push(None);
                        subject_set_relations.push(None);
                    }
                    Subject::Set(SubjectSet {
                        namespace,
                        object,
                        relation,
                    }) => {
                        subject_ids.push(None);
                        subject_set_namespaces.push(Some(namespace.clone()));
                        subject_set_objects.push(Some(*object));
                        subject_set_relations.push(Some(relation.clone()));
                    }
                }
            }

            sqlx::query("DELETE FROM heimdall_relation_tuples t
                        USING UNNEST($1::VARCHAR[], $2::UUID[], $3::VARCHAR[], $4::UUID[], $5::VARCHAR[], $6::UUID[], $7::VARCHAR[], $8::UUID[])
                        AS u(namespace, object, relation, subject_id, subject_set_namespace, subject_set_object, subject_set_relation, nid)
                        WHERE 
                        t.namespace = u.namespace AND
                        t.object = u.object AND 
                        t.relation = u.relation AND
                        t.subject_id = u.subject_id AND
                        t.subject_set_namespace = u.subject_set_namespace AND
                        t.subject_set_object = u.subject_set_object AND
                        t.subject_set_relation = u.subject_set_relation AND 
                        t.nid = u.nid
                        ")
                .bind(namespaces)
                .bind(objects)
                .bind(relations)
                .bind(subject_ids)
                .bind(subject_set_namespaces)
                .bind(subject_set_objects)
                .bind(subject_set_relations)
                .bind(nids)
                .execute(&mut *tx)
            .await?;
        }

        // NOTE: if the transaction is not commited, it rolls back automatically
        tx.commit().await?;
        Ok(())
    }

    async fn delete_all_relation_tuples(
        &self,
        ctx: &RequestContext,
        rs_query: &RelationTupleQuery,
    ) -> HeimdallResult<()> {
        let span = info_span!("delete_relation_tuples");
        let _guard = span.enter();

        let mut tx = self.pool.begin().await?;

        let mut builder = QueryBuilder::new("DELETE FROM heimdall_relation_tuples WHERE ");

        Self::with_network(&mut builder, ctx);
        Self::with_query_filters(&mut builder, rs_query);

        builder.build().execute(&mut *tx).await?;

        tx.commit().await?;
        Ok(())
    }
}
