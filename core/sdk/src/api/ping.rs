pub enum PingRequestPayload {
    Ping,
}
pub enum PingResponsePayload {
    Pong,
}
pub struct PingRequest {
    pub id: String,
    pub payload: PingRequestPayload,
}
pub struct PingResponse {
    pub id: String,
    pub payload: PingResponsePayload,
}