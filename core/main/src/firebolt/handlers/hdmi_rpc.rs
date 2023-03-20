use crate::{
    firebolt::rpc::RippleRPCProvider, state::platform_state::PlatformState,
    utils::rpc_utils::rpc_err,
};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    proc_macros::rpc,
    RpcModule,
};
use ripple_sdk::api::{
    firebolt::fb_hdmi::GetAvailableInputsResponse, gateway::rpc_gateway_api::CallContext,
};
use ripple_sdk::serde_json;
use ripple_sdk::{
    api::device::device_hdmi::HdmiRequest,
    extn::extn_client_message::{ExtnPayload, ExtnResponse},
};

#[rpc(server)]
pub trait Hdmi {
    #[method(name = "hdmi.getAvailableInputs")]
    async fn get_available_inputs(&self, ctx: CallContext)
        -> RpcResult<GetAvailableInputsResponse>;
}

#[derive(Debug)]
pub struct HdmiImpl {
    pub state: PlatformState,
}

#[async_trait]
impl HdmiServer for HdmiImpl {
    async fn get_available_inputs(
        &self,
        _ctx: CallContext,
    ) -> RpcResult<GetAvailableInputsResponse> {
        if let Ok(response) = self
            .state
            .get_client()
            .send_extn_request(HdmiRequest::GetAvailableInputs)
            .await
        {
            match response.payload {
                ExtnPayload::Response(payload) => match payload {
                    ExtnResponse::Value(value) => {
                        if let Ok(res) = serde_json::from_value::<GetAvailableInputsResponse>(value)
                        {
                            return Ok(res);
                        }
                    }
                    _ => (),
                },
                _ => (),
            }
        }

        Err(rpc_err("FB error response TBD"))
    }
}

pub struct HdmiRPCProvider;
impl RippleRPCProvider<HdmiImpl> for HdmiRPCProvider {
    fn provide(state: PlatformState) -> RpcModule<HdmiImpl> {
        (HdmiImpl { state }).into_rpc()
    }
}
