#[async_trait]
trait RippleClientTMT {
    pub async fn send_extn_request(&self, msg: ExtnMessage) -> Result<(), String>;
}