use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenPagination {
    pub last_id: Option<Uuid>,
    pub page_size: Option<i32>,
}

#[allow(unused)]
impl TokenPagination {
    pub fn encode_next_page_token(last_id: &Uuid) -> String {
        last_id.to_string()
    }
}
