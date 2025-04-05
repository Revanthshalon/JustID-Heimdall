use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct SubjectExapandedRelationTupleRow {
    pub shard_id: Uuid,
    pub subject_set_namespace: String,
    pub subject_set_object: Uuid,
    pub subject_set_relation: String,
    pub found: bool,
}
