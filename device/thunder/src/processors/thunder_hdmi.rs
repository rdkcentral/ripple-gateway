use thunder_ripple_sdk::ripple_sdk::{
    api::{
        device::{
            device_hdmi::HdmiRequest,
            device_operator::{DeviceCallRequest, DeviceOperator},
        },
        firebolt::fb_hdmi::GetAvailableInputsResponse,
    },
    async_trait::async_trait,
    extn::{
        client::extn_processor::{
            DefaultExtnStreamer, ExtnRequestProcessor, ExtnStreamProcessor, ExtnStreamer,
        },
        extn_client_message::{ExtnMessage, ExtnResponse},
    },
    log::info,
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

impl ThunderHdmiRequestProcessor {
    pub fn new(state: ThunderState) -> ThunderHdmiRequestProcessor {
        ThunderHdmiRequestProcessor {
            state,
            streamer: DefaultExtnStreamer::new(),
        }
    }

    async fn get_available_inputs(state: ThunderState, req: ExtnMessage) -> bool {
        let response = state
            .get_thunder_client()
            .call(DeviceCallRequest {
                method: ThunderPlugin::Hdmi.method("getHDMIInputDevices"),
                params: None,
            })
            .await;
        info!("{}", response.message);

        let response = if let Ok(_res) =
            serde_json::from_value::<GetAvailableInputsResponse>(response.message.clone())
        {
            ExtnResponse::Value(response.message)
        } else {
            ExtnResponse::Error(RippleError::InvalidOutput)
        };

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
        }
    }
}
