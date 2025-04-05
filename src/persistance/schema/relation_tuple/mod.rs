use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
#[allow(unused)]
pub struct RelationTuple {
    pub shard_id: Uuid,
    pub nid: Uuid,
    pub namespace: String,
    pub object: Uuid,
    pub relation: String,
    pub subject_id: Option<Uuid>,
    pub subject_set_namespace: Option<String>,
    pub subject_set_object: Option<Uuid>,
    pub subject_set_relation: Option<String>,
    pub commit_time: DateTime<Utc>,
}
