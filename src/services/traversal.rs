use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};
use tracing::info_span;
use uuid::Uuid;

use crate::{
    context::RequestContext,
    error::HeimdallResult,
    models::{
        relation_tuple::{RelationTuple, Subject, SubjectID, SubjectSet},
        traversal::{Traversal, TraversalResult},
    },
    persistance::schema::SubjectExapandedRelationTupleRow,
};

use super::traits::TraversalManager;

pub struct TraversalService {
    pool: PgPool,
}

const QUERY_LIMIT: i32 = 1000;

impl TraversalService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn with_subject_filter<'a>(builder: &mut QueryBuilder<'a, Postgres>, subject: &'a Subject) {
        match subject {
            Subject::Direct(SubjectID { id }) => {
                builder.push(" subject_id = ");
                builder.push_bind(*id);
                builder.push(" AND subject_set_namespace IS NULL AND subject_set_object IS NULL AND subject_set_relation IS NULL");
            }
            Subject::Set(SubjectSet {
                namespace,
                object,
                relation,
            }) => {
                builder.push(" subject_id IS NULL");
                builder.push(" AND subject_set_namespace = ");
                builder.push_bind(namespace.clone());
                builder.push(" AND subject_set_object = ");
                builder.push_bind(*object);
                builder.push(" AND subject_set_relation = ");
                builder.push_bind(relation.clone());
            }
        }
    }
}

#[async_trait]
impl TraversalManager for TraversalService {
    async fn traverse_subject_set_expansion(
        &self,
        ctx: &RequestContext,
        start: &RelationTuple,
    ) -> HeimdallResult<Vec<TraversalResult>> {
        let span = info_span!("traverse_subject_set_expansion");
        let _guard = span.enter();

        let mut exists_builder = QueryBuilder::new(
            "SELECT 1 FROM heimdall_relation_tuples WHERE nid = current.nid AND namespace = current.subject_set_namespace AND relation = current.subject_set_relation AND ",
        );
        Self::with_subject_filter(&mut exists_builder, &start.subject);

        let mut shard_id = Uuid::nil();
        let mut results = Vec::new();

        loop {
            let mut builder = QueryBuilder::new(
                r#"SELECT current.shard_id AS shard_id,
                          current.subject_set_namespace as namespace,
                          current.subject_set_object as object,
                          current.subject_set_relation as relation,
                          EXISTS ("#,
            );
            builder.push(exists_builder.sql());
            builder.push(") AS found FROM heimdall_relation_tuples AS current WHERE");
            builder.push(" current.nid = ");
            builder.push_bind(ctx.network_id());
            builder.push(" AND current.shard_id > ");
            builder.push_bind(shard_id);
            builder.push(" AND current.namespace = ");
            builder.push_bind(start.namespace.clone());
            builder.push(" AND current.object = ");
            builder.push_bind(start.object);
            builder.push(" AND current.relation = ");
            builder.push_bind(start.relation.clone());
            builder.push(" AND current.subject_id IS NULL");
            builder.push(" ORDER BY current.shard_id");
            builder.push(" LIMIT ");
            builder.push_bind(QUERY_LIMIT);

            let rows: Vec<SubjectExapandedRelationTupleRow> =
                builder.build_query_as().fetch_all(&self.pool).await?;

            if rows.is_empty() {
                break;
            }

            for row in rows.iter() {
                let to = RelationTuple {
                    namespace: row.subject_set_namespace.clone(),
                    object: row.subject_set_object,
                    relation: row.subject_set_relation.clone(),
                    subject: start.subject.clone(),
                };
                let result = TraversalResult {
                    from: start.clone(),
                    to,
                    via: Traversal::SubjectSetExpand,
                    found: row.found,
                };

                results.push(result);

                if row.found {
                    return Ok(results);
                }
                shard_id = row.shard_id;
            }

            if rows.len() < QUERY_LIMIT as usize {
                break;
            }
        }
        Ok(results)
    }

    async fn traverse_subject_set_rewrite(
        &self,
        _ctx: &RequestContext,
        _start: &RelationTuple,
        _computed_suject_set: &[String],
    ) -> HeimdallResult<Vec<TraversalResult>> {
        todo!()
    }
}
