use async_trait::async_trait;

use crate::{
    context::RequestContext,
    error::HeimdallResult,
    models::{relation_tuple::RelationTuple, traversal::TraversalResult},
};

#[async_trait]
#[allow(unused)]
pub trait TraversalManager: Send + Sync {
    async fn traverse_subject_set_expansion(
        &self,
        ctx: &RequestContext,
        start: &RelationTuple,
    ) -> HeimdallResult<Vec<TraversalResult>>;
    async fn traverse_subject_set_rewrite(
        &self,
        ctx: &RequestContext,
        start: &RelationTuple,
        computed_subject_sets: &[String],
    ) -> HeimdallResult<Vec<TraversalResult>>;
}
