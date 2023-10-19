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

use super::provider::{
    ProviderRequestPayload, ProviderResponse, ProviderResponsePayload, ToProviderResponse,
};

pub const PLAYER_LOAD_EVENT: &str = "player.onRequestLoad";
pub const PLAYER_LOAD_METHOD: &str = "load";
pub const PLAYER_PLAY_EVENT: &str = "player.onRequestPlay";
pub const PLAYER_PLAY_METHOD: &str = "play";
pub const PLAYER_STOP_EVENT: &str = "player.onRequestStop";
pub const PLAYER_STOP_METHOD: &str = "stop";

pub const PLAYER_BASE_PROVIDER_CAPABILITY: &str = "xrn:firebolt:capability:player:base";

// TODO: track playerIds to app ids, validate playerIds and add errors for unfound and invalid ids

// TODO: try impl serialize to remove enum encoding level
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PlayerRequest {
    Load(PlayerLoadRequest),
    Play(PlayerPlayRequest),
    Stop(PlayerStopRequest),
}

impl PlayerRequest {
    pub fn to_provider_request_payload(&self) -> ProviderRequestPayload {
        match self {
            Self::Load(load_request) => ProviderRequestPayload::PlayerLoad(load_request.clone()),
            Self::Play(play_request) => ProviderRequestPayload::PlayerPlay(play_request.clone()),
            Self::Stop(stop_request) => ProviderRequestPayload::PlayerStop(stop_request.clone()),
        }
    }

    pub fn to_provider_method(&self) -> &str {
        match self {
            Self::Load(_) => PLAYER_LOAD_METHOD,
            Self::Play(_) => PLAYER_PLAY_METHOD,
            Self::Stop(_) => PLAYER_STOP_METHOD,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerRequestWithContext {
    pub request: PlayerRequest,
    pub call_ctx: CallContext,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerLoadRequest {
    pub player_id: String, // TODO: spec shows this prefixed with the appId - do we need to do that?
    pub locator: String,
    pub metadata: Option<HashMap<String, String>>,
    pub autoplay: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerPlayRequest {
    pub player_id: String, // TODO: spec shows this prefixed with the appId - do we need to do that?
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerStopRequest {
    pub player_id: String, // TODO: spec shows this prefixed with the appId - do we need to do that?
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PlayerProviderResponse {
    Load(PlayerLoadResponse),
    LoadError(PlayerErrorResponseParams),
    Play(PlayerPlayResponse),
    PlayError(PlayerErrorResponseParams),
    Stop(PlayerPlayResponse),
    StopError(PlayerErrorResponseParams),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PlayerResponse {
    Load(PlayerMediaSession),
    Play(PlayerMediaSession),
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerMediaSession {
    pub media_session_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerLoadResponseParams {
    pub response: PlayerLoadResponse,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerLoadResponse {
    pub correlation_id: String,
    pub result: PlayerMediaSession,
}

impl PlayerLoadResponse {
    pub fn new(correlation_id: String, result: PlayerMediaSession) -> Self {
        Self {
            correlation_id,
            result,
        }
    }
}

impl ToProviderResponse for PlayerLoadResponse {
    fn to_provider_response(&self) -> ProviderResponse {
        ProviderResponse {
            correlation_id: self.correlation_id.clone(),
            result: ProviderResponsePayload::PlayerLoad(self.result.clone()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerErrorResponse {
    pub code: u32,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerErrorResponseParams {
    pub correlation_id: String,
    pub error: PlayerErrorResponse,
}

impl PlayerErrorResponseParams {
    pub fn new(correlation_id: String, error: PlayerErrorResponse) -> Self {
        Self {
            correlation_id,
            error,
        }
    }
}

impl ToProviderResponse for PlayerErrorResponseParams {
    fn to_provider_response(&self) -> ProviderResponse {
        ProviderResponse {
            correlation_id: self.correlation_id.clone(),
            result: ProviderResponsePayload::PlayerLoadError(self.clone()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerPlayResponseParams {
    pub response: PlayerPlayResponse,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerPlayResponse {
    pub correlation_id: String,
    pub result: PlayerMediaSession,
}

impl PlayerPlayResponse {
    pub fn new(correlation_id: String, result: PlayerMediaSession) -> Self {
        Self {
            correlation_id,
            result,
        }
    }
}

impl ToProviderResponse for PlayerPlayResponse {
    fn to_provider_response(&self) -> ProviderResponse {
        ProviderResponse {
            correlation_id: self.correlation_id.clone(),
            result: ProviderResponsePayload::PlayerPlay(self.result.clone()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerStopResponseParams {
    pub response: PlayerStopResponse,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerStopResponse {
    pub correlation_id: String,
    pub result: PlayerMediaSession,
}

impl PlayerStopResponse {
    pub fn new(correlation_id: String, result: PlayerMediaSession) -> Self {
        Self {
            correlation_id,
            result,
        }
    }
}

impl ToProviderResponse for PlayerStopResponse {
    fn to_provider_response(&self) -> ProviderResponse {
        ProviderResponse {
            correlation_id: self.correlation_id.clone(),
            result: ProviderResponsePayload::PlayerStop(self.result.clone()),
        }
    }
}
