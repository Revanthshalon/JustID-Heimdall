use async_trait::async_trait;

use crate::{
    context::RequestContext,
    error::HeimdallResult,
    models::{
        query::{TokenPagination, relation_tuple::RelationTupleQuery},
        relation_tuple::RelationTuple,
        response::PaginatedResponse,
    },
};

#[async_trait]
#[allow(unused)]
pub trait RelationTupleManager: Send + Sync {
    async fn write_relation_tuples(
        &self,
        ctx: &RequestContext,
        rs: &[RelationTuple],
    ) -> HeimdallResult<()>;

    async fn get_relation_tuples(
        &self,
        ctx: &RequestContext,
        rs_query: &RelationTupleQuery,
        pagination_params: &TokenPagination,
    ) -> HeimdallResult<PaginatedResponse<Vec<RelationTuple>>>;

    async fn exists_relation_tuples(
        &self,
        ctx: &RequestContext,
        rs_query: &RelationTupleQuery,
    ) -> HeimdallResult<bool>;

    async fn delete_relation_tuples(
        &self,
        ctx: &RequestContext,
        rs: &[RelationTuple],
    ) -> HeimdallResult<()>;

    async fn delete_all_relation_tuples(
        &self,
        ctx: &RequestContext,
        rs_query: &RelationTupleQuery,
    ) -> HeimdallResult<()>;
}
