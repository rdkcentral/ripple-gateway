use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetAvailableInputsResponse {
    pub devices: Vec<HdmiInput>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HdmiInput {
    pub id: i32,
    pub locator: String,
    pub connected: String,
}
