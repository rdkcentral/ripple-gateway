use serde::{Deserialize, Serialize};

use crate::{
    extn::extn_client_message::{ExtnPayload, ExtnPayloadProvider, ExtnRequest},
    framework::ripple_contract::{DeviceContract, RippleContract},
};

use super::device_request::DeviceRequest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HdmiRequest {
    GetAvailableInputs,
    SetActiveInput(String),
}

impl ExtnPayloadProvider for HdmiRequest {
    fn get_extn_payload(&self) -> ExtnPayload {
        ExtnPayload::Request(ExtnRequest::Device(DeviceRequest::Hdmi(self.clone())))
    }

    fn get_from_payload(payload: ExtnPayload) -> Option<Self> {
        match payload {
            ExtnPayload::Request(request) => match request {
                ExtnRequest::Device(r) => match r {
                    DeviceRequest::Hdmi(d) => return Some(d),
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }
        None
    }

    fn contract() -> RippleContract {
        RippleContract::Device(DeviceContract::Hdmi)
    }
}
