// Copyright 2023 Comcast Cable Communications Management, LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0
//

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    api::gateway::rpc_gateway_api::CallContext,
    extn::extn_client_message::{ExtnPayload, ExtnPayloadProvider, ExtnRequest, ExtnResponse},
    framework::ripple_contract::RippleContract,
};

use super::provider::ProviderRequestPayload;

pub const PLAYER_LOAD_EVENT: &str = "player.onRequestLoad";
pub const PLAYER_LOAD_METHOD: &str = "load";
pub const PLAYER_BASE_PROVIDER_CAPABILITY: &str = "xrn:firebolt:capability:player:base";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PlayerRequest {
    Load(LoadRequest),
    Play,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PlayerResponse {
    Load(LoadResponse),
    Play,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerRequestWithContext {
    pub request: PlayerRequest,
    pub call_ctx: CallContext,
}

impl PlayerRequest {
    pub fn to_provider_request_payload(&self) -> ProviderRequestPayload {
        match self {
            Self::Load(load_request) => ProviderRequestPayload::PlayerLoad(load_request.clone()),
            Self::Play => ProviderRequestPayload::PlayerPlay,
        }
    }

    pub fn to_provider_method(&self) -> &str {
        match self {
            Self::Load(_) => PLAYER_LOAD_METHOD,
            Self::Play => "",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoadRequest {
    pub player_id: String, // TODO: spec shows this prefixed with the appId - do we need to do that?
    pub locator: String,
    pub metadata: Option<HashMap<String, String>>,
    pub autoplay: Option<bool>,
}

// impl From<LoadRequestWithContext> for LoadRequest {
//     fn from(load_req: LoadRequestWithContext) -> Self {
//         LoadRequest {
//             player_id: load_req.player_id,
//             locator: load_req.locator,
//             metadata: load_req.metadata,
//             autoplay: load_req.autoplay,
//         }
//     }
// }

impl ExtnPayloadProvider for PlayerRequestWithContext {
    fn get_extn_payload(&self) -> ExtnPayload {
        ExtnPayload::Request(ExtnRequest::Player(self.clone()))
    }

    fn get_from_payload(payload: ExtnPayload) -> Option<Self> {
        if let ExtnPayload::Request(ExtnRequest::Player(r)) = payload {
            return Some(r);
        }

        None
    }

    fn contract() -> RippleContract {
        RippleContract::Player(crate::api::player::PlayerAdjective::Base)
    }
}

// impl ExtnPayloadProvider for LoadRequest {
//     fn get_extn_payload(&self) -> ExtnPayload {
//         ExtnPayload::Request(ExtnRequest::Player(PlayerRequest::Load(self.clone())))
//     }

//     fn get_from_payload(payload: ExtnPayload) -> Option<Self> {
//         if let ExtnPayload::Request(ExtnRequest::Player(PlayerRequest::Load(r))) = payload {
//             return Some(r);
//         }

//         None
//     }

//     fn contract() -> RippleContract {
//         RippleContract::Player(crate::api::player::PlayerAdjective::Base)
//     }
// }

// #[derive(Serialize, Deserialize, Debug, Clone)]
// #[serde(rename_all = "camelCase")]
// pub struct LoadRequestWithContext {
//     pub player_id: String,
//     pub locator: String,
//     pub metadata: Option<HashMap<String, String>>,
//     pub autoplay: Option<bool>,
//     pub capability: Option<String>,
//     pub call_ctx: CallContext,
// }

// impl ExtnPayloadProvider for LoadRequestWithContext {
//     fn get_extn_payload(&self) -> ExtnPayload {
//         ExtnPayload::Request(ExtnRequest::Player( crate::api::player::PlayerRequest::Load(sel)self.clone()))
//     }

//     fn get_from_payload(payload: ExtnPayload) -> Option<Self> {
//         if let ExtnPayload::Request(ExtnRequest::Player(r)) = payload {
//             return Some(r);
//         }

//         None
//     }

//     fn contract() -> RippleContract {
//         RippleContract::Player(crate::api::player::PlayerAdjective::Base)
//     }
// }

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoadResponseResult {
    pub media_session_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoadResponse {
    pub correlation_id: String,
    pub result: LoadResponseResult,
}

impl LoadResponse {
    // pub fn get_result(&self) -> Option<bool> {
    //     self.granted
    // }

    // pub fn get_reason(&self) -> PlayerResultReason {
    //     self.reason.clone()
    // }

    pub fn new(correlation_id: String, result: LoadResponseResult) -> Self {
        LoadResponse {
            correlation_id,
            result,
        }
    }
}

impl ExtnPayloadProvider for PlayerResponse {
    fn get_extn_payload(&self) -> ExtnPayload {
        ExtnPayload::Response(ExtnResponse::Player(self.clone()))
    }

    fn get_from_payload(payload: ExtnPayload) -> Option<Self> {
        if let ExtnPayload::Response(ExtnResponse::Player(r)) = payload {
            return Some(r);
        }

        None
    }

    fn contract() -> RippleContract {
        RippleContract::Player(crate::api::player::PlayerAdjective::Base)
    }
}

// impl ExtnPayloadProvider for LoadResponse {
//     fn get_extn_payload(&self) -> ExtnPayload {
//         ExtnPayload::Response(ExtnResponse::Player(PlayerResponse::Load(self.clone())))
//     }

//     fn get_from_payload(payload: ExtnPayload) -> Option<Self> {
//         if let ExtnPayload::Response(ExtnResponse::Player(PlayerResponse::Load(r))) = payload {
//             return Some(r);
//         }

//         None
//     }

//     fn contract() -> RippleContract {
//         RippleContract::Player(crate::api::player::PlayerAdjective::Base)
//     }
// }
