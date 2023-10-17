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

use super::provider::{ProviderRequestPayload, ProviderResponse, ProviderResponsePayload};

pub const PLAYER_LOAD_EVENT: &str = "player.onRequestLoad";
pub const PLAYER_LOAD_METHOD: &str = "load";
pub const PLAYER_BASE_PROVIDER_CAPABILITY: &str = "xrn:firebolt:capability:player:base";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PlayerRequest {
    Load(PlayerLoadRequest),
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
pub struct PlayerLoadRequest {
    pub player_id: String, // TODO: spec shows this prefixed with the appId - do we need to do that?
    pub locator: String,
    pub metadata: Option<HashMap<String, String>>,
    pub autoplay: Option<bool>,
}

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

pub trait PlayerProviderResponse {
    fn to_provider_response(&self) -> ProviderResponse;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoadResponseResult {
    pub media_session_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerLoadResponse {
    pub correlation_id: String,
    pub result: LoadResponseResult,
}

impl PlayerLoadResponse {
    pub fn new(correlation_id: String, result: LoadResponseResult) -> Self {
        Self {
            correlation_id,
            result,
        }
    }
}

impl PlayerProviderResponse for PlayerLoadResponse {
    fn to_provider_response(&self) -> ProviderResponse {
        ProviderResponse {
            correlation_id: self.correlation_id.clone(),
            result: ProviderResponsePayload::PlayerLoad(self.clone()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoadErrorResult {
    pub code: u32,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerLoadError {
    pub correlation_id: String,
    pub result: LoadResponseResult,
}

impl PlayerLoadError {
    pub fn new(correlation_id: String, result: LoadResponseResult) -> Self {
        Self {
            correlation_id,
            result,
        }
    }
}

impl PlayerProviderResponse for PlayerLoadError {
    fn to_provider_response(&self) -> ProviderResponse {
        ProviderResponse {
            correlation_id: self.correlation_id.clone(),
            result: ProviderResponsePayload::PlayerLoadError(self.clone()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PlayerResponse {
    Load(PlayerLoadResponse),
    LoadError(PlayerLoadError),
    Play,
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
