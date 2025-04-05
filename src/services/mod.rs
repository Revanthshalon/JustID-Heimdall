use std::sync::Arc;

use relation_tuple::RelationTupleService;
use sqlx::PgPool;
use traits::{RelationTupleManager, TraversalManager, UuidMappingManager};
use traversal::TraversalService;
use uuid_mapper::UuidMappingService;

pub mod network;
pub mod relation_tuple;
pub mod traits;
pub mod traversal;
pub mod uuid_mapper;

#[derive(Clone)]
#[allow(unused)]
pub struct Services {
    pub relation_tuple_service: Arc<dyn RelationTupleManager>,
    pub uuid_mapping_service: Arc<dyn UuidMappingManager>,
    pub traversal_service: Arc<dyn TraversalManager>,
}

#[allow(unused)]
impl Services {
    pub fn new(pool: PgPool) -> Self {
        let relation_tuple_service = Arc::new(RelationTupleService::new(pool.clone()));
        let uuid_mapping_service = Arc::new(UuidMappingService::new(pool.clone()));
        let traversal_service = Arc::new(TraversalService::new(pool.clone()));
        Self {
            relation_tuple_service,
            uuid_mapping_service,
            traversal_service,
        }
    }
}
