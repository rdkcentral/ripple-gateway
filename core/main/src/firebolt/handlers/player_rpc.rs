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

use crate::{
    firebolt::rpc::RippleRPCProvider, service::apps::provider_broker::ProviderBroker,
    state::platform_state::PlatformState,
};
use jsonrpsee::{core::RpcResult, proc_macros::rpc, RpcModule};
use ripple_sdk::api::{
    firebolt::{
        fb_general::{ListenRequest, ListenerResponse},
        fb_player::{PlayerResponse, PLAYER_BASE_PROVIDER_CAPABILITY, PLAYER_LOAD_EVENT},
        provider::{
            ExternalProviderResponse, FocusRequest, ProviderResponse, ProviderResponsePayload,
        },
    },
    gateway::rpc_gateway_api::CallContext,
};
use ripple_sdk::async_trait::async_trait;

#[rpc(server)]
pub trait Player {
    #[method(name = "player.onRequest")]
    async fn on_request_challenge(
        &self,
        ctx: CallContext,
        request: ListenRequest,
    ) -> RpcResult<ListenerResponse>;

    #[method(name = "player.challengeFocus")]
    async fn challenge_focus(
        &self,
        ctx: CallContext,
        request: FocusRequest,
    ) -> RpcResult<Option<()>>;

    #[method(name = "player.challengeResponse")]
    async fn challenge_response(
        &self,
        ctx: CallContext,
        resp: ExternalProviderResponse<PlayerResponse>,
    ) -> RpcResult<Option<()>>;
}

pub struct PlayerImpl {
    pub platform_state: PlatformState,
}

#[async_trait]
impl PlayerServer for PlayerImpl {
    async fn on_request_challenge(
        &self,
        ctx: CallContext,
        request: ListenRequest,
    ) -> RpcResult<ListenerResponse> {
        let listen = request.listen;
        ProviderBroker::register_or_unregister_provider(
            &self.platform_state,
            String::from(PLAYER_BASE_PROVIDER_CAPABILITY),
            String::from("challenge"),
            PLAYER_LOAD_EVENT,
            ctx,
            request,
        )
        .await;
        Ok(ListenerResponse {
            listening: listen,
            event: PLAYER_LOAD_EVENT.into(),
        })
    }

    async fn challenge_response(
        &self,
        _ctx: CallContext,
        resp: ExternalProviderResponse<PlayerResponse>,
    ) -> RpcResult<Option<()>> {
        let msg = ProviderResponse {
            correlation_id: resp.correlation_id,
            result: ProviderResponsePayload::PlayerResponse(resp.result),
        };
        ProviderBroker::provider_response(&self.platform_state, msg).await;
        Ok(None)
    }

    async fn challenge_focus(
        &self,
        ctx: CallContext,
        request: FocusRequest,
    ) -> RpcResult<Option<()>> {
        ProviderBroker::focus(
            &self.platform_state,
            ctx,
            PLAYER_BASE_PROVIDER_CAPABILITY.to_string(),
            request,
        )
        .await;
        Ok(None)
    }
}

pub struct PlayerRPCProvider;

impl RippleRPCProvider<PlayerImpl> for PlayerRPCProvider {
    fn provide(state: PlatformState) -> RpcModule<PlayerImpl> {
        (PlayerImpl {
            platform_state: state,
        })
        .into_rpc()
    }
}
