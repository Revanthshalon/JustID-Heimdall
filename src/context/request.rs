use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestContext {
    network_id: Uuid,
    request_id: String,
    trace_id: String,
}

impl RequestContext {
    pub fn network_id(&self) -> &Uuid {
        &self.network_id
    }
}
