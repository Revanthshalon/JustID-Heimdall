use async_trait::async_trait;
use uuid::Uuid;

use crate::{context::RequestContext, error::HeimdallResult, models::query::TokenPagination};

#[async_trait]
#[allow(unused)]
pub trait UuidMappingManager: Send + Sync {
    async fn map_strings_to_uuids(
        &self,
        ctx: &RequestContext,
        values: &[String],
    ) -> HeimdallResult<Vec<Uuid>>;

    async fn map_strings_to_uuids_readonly(
        &self,
        ctx: &RequestContext,
        values: &[String],
    ) -> HeimdallResult<Vec<Uuid>>;

    async fn map_uuids_to_strings(
        &self,
        ctx: &RequestContext,
        ids: &[Uuid],
        pagination_params: &TokenPagination,
    ) -> HeimdallResult<Vec<String>>;
}
