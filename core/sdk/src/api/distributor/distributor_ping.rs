use serde::{Deserialize, Serialize};

use crate::{
    extn::extn_client_message::{ExtnPayload, ExtnPayloadProvider, ExtnRequest},
    framework::ripple_contract::RippleContract,
};

use super::distributor_request::DistributorRequest;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DistributorPingRequest {}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DistributorPingResponse {}

impl ExtnPayloadProvider for DistributorPingRequest {
    fn get_from_payload(payload: ExtnPayload) -> Option<Self> {
        if let ExtnPayload::Request(ExtnRequest::Distributor(DistributorRequest::Ping(p))) = payload
        {
            return Some(p);
        }

        None
    }

    fn get_extn_payload(&self) -> ExtnPayload {
        ExtnPayload::Request(ExtnRequest::Distributor(DistributorRequest::Ping(
            self.clone(),
        )))
    }

    fn contract() -> RippleContract {
        RippleContract::Ping
    }

    fn get_contract(&self) -> crate::framework::ripple_contract::RippleContract {
        Self::contract()
    }
}
