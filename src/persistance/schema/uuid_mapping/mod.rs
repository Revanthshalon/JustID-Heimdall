use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UuidMapping {
    pub id: Uuid,
    pub string_representation: String,
}
