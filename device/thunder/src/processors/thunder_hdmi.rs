use serde::{Deserialize, Serialize};
use thunder_ripple_sdk::ripple_sdk::{
    api::{
        device::{
            device_hdmi::HdmiRequest,
            device_operator::{DeviceCallRequest, DeviceChannelParams, DeviceOperator},
        },
        firebolt::fb_hdmi::{GetAvailableInputsResponse, StartHdmiInputResponse},
    },
    async_trait::async_trait,
    extn::{
        client::extn_processor::{
            DefaultExtnStreamer, ExtnRequestProcessor, ExtnStreamProcessor, ExtnStreamer,
        },
        extn_client_message::{ExtnMessage, ExtnResponse},
    },
    serde_json,
    utils::error::RippleError,
};
use thunder_ripple_sdk::{
    client::thunder_plugin::ThunderPlugin,
    ripple_sdk::{extn::client::extn_client::ExtnClient, tokio::sync::mpsc},
    thunder_state::ThunderState,
};

#[derive(Debug)]
pub struct ThunderHdmiRequestProcessor {
    state: ThunderState,
    streamer: DefaultExtnStreamer,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AVInputGetInputDevicesParams {
    type_of_input: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AVInputStartHdmiInputParams {
    port_id: String,
    type_of_input: String,
}

impl ThunderHdmiRequestProcessor {
    pub fn new(state: ThunderState) -> ThunderHdmiRequestProcessor {
        ThunderHdmiRequestProcessor {
            state,
            streamer: DefaultExtnStreamer::new(),
        }
    }

    async fn get_available_inputs(state: ThunderState, req: ExtnMessage) -> bool {
        let params = AVInputGetInputDevicesParams {
            type_of_input: "HDMI".to_owned(),
        };

        let response = state
            .get_thunder_client()
            .call(DeviceCallRequest {
                method: ThunderPlugin::AVInput.method("getInputDevices"),
                params: serde_json::to_string(&params)
                    .map(DeviceChannelParams::Json)
                    .ok(),
            })
            .await;

        let response =
            serde_json::from_value::<GetAvailableInputsResponse>(response.message.clone())
                .map(|_| ExtnResponse::Value(response.message))
                .unwrap_or(ExtnResponse::Error(RippleError::InvalidOutput));

        Self::respond(state.get_client(), req, response)
            .await
            .is_ok()
    }

    async fn start_hdmi_input(state: ThunderState, port_id: String, req: ExtnMessage) -> bool {
        let params = AVInputStartHdmiInputParams {
            port_id,
            type_of_input: "HDMI".to_owned(),
        };

        let response = state
            .get_thunder_client()
            .call(DeviceCallRequest {
                method: ThunderPlugin::AVInput.method("startInput"),
                params: serde_json::to_string(&params)
                    .ok()
                    .map(DeviceChannelParams::Json),
            })
            .await;

        let response = serde_json::from_value::<StartHdmiInputResponse>(response.message.clone())
            .map(|_| ExtnResponse::Value(response.message))
            .unwrap_or(ExtnResponse::Error(RippleError::InvalidOutput));

        Self::respond(state.get_client(), req, response)
            .await
            .is_ok()
    }
}

impl ExtnStreamProcessor for ThunderHdmiRequestProcessor {
    type STATE = ThunderState;
    type VALUE = HdmiRequest;

    fn get_state(&self) -> Self::STATE {
        self.state.clone()
    }

    fn receiver(&mut self) -> mpsc::Receiver<ExtnMessage> {
        self.streamer.receiver()
    }

    fn sender(&self) -> mpsc::Sender<ExtnMessage> {
        self.streamer.sender()
    }
}

#[async_trait]
impl ExtnRequestProcessor for ThunderHdmiRequestProcessor {
    fn get_client(&self) -> ExtnClient {
        self.state.get_client()
    }

    async fn process_request(
        state: Self::STATE,
        msg: ExtnMessage,
        extracted_message: Self::VALUE,
    ) -> bool {
        match extracted_message {
            HdmiRequest::GetAvailableInputs => Self::get_available_inputs(state.clone(), msg).await,
            HdmiRequest::SetActiveInput(port_id) => {
                Self::start_hdmi_input(state.clone(), port_id, msg).await
            }
        }
    }
}
