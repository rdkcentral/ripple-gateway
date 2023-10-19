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

use jsonrpsee::{core::RpcResult, proc_macros::rpc, RpcModule};
use ripple_sdk::{
    api::{
        firebolt::{
            fb_general::{ListenRequest, ListenerResponse},
            fb_player::{
                PlayerErrorResponse, PlayerLoadRequest, PlayerLoadResponseParams,
                PlayerMediaSession, PlayerPlayError, PlayerPlayRequest, PlayerPlayResponseParams,
                PlayerRequest, PlayerRequestWithContext, PLAYER_BASE_PROVIDER_CAPABILITY,
                PLAYER_LOAD_EVENT, PLAYER_LOAD_METHOD, PLAYER_PLAY_EVENT, PLAYER_PLAY_METHOD,
            },
            provider::{ProviderResponsePayload, ToProviderResponse},
        },
        gateway::rpc_gateway_api::CallContext,
    },
    async_trait::async_trait,
    tokio::sync::oneshot,
    utils::rpc_utils::rpc_err,
};

use crate::{
    firebolt::rpc::RippleRPCProvider,
    service::apps::provider_broker::{ProviderBroker, ProviderBrokerRequest},
    state::platform_state::PlatformState,
};

#[rpc(server)]
pub trait Player {
    #[method(name = "player.onRequestLoad")]
    async fn on_request_load(
        &self,
        ctx: CallContext,
        request: ListenRequest,
    ) -> RpcResult<ListenerResponse>;

    #[method(name = "player.load")]
    async fn load(
        &self,
        ctx: CallContext,
        request: PlayerLoadRequest,
    ) -> RpcResult<PlayerMediaSession>;

    #[method(name = "player.loadResponse")]
    async fn load_response(
        &self,
        ctx: CallContext,
        request: PlayerLoadResponseParams,
    ) -> RpcResult<Option<()>>;

    #[method(name = "player.loadError")]
    async fn load_error(
        &self,
        ctx: CallContext,
        request: PlayerErrorResponse,
    ) -> RpcResult<Option<()>>;

    #[method(name = "player.onRequestPlay")]
    async fn on_request_play(
        &self,
        ctx: CallContext,
        request: ListenRequest,
    ) -> RpcResult<ListenerResponse>;

    #[method(name = "player.play")]
    async fn play(
        &self,
        ctx: CallContext,
        request: PlayerPlayRequest,
    ) -> RpcResult<PlayerMediaSession>;

    #[method(name = "player.playResponse")]
    async fn play_response(
        &self,
        ctx: CallContext,
        request: PlayerPlayResponseParams,
    ) -> RpcResult<Option<()>>;

    #[method(name = "player.playError")]
    async fn play_error(&self, ctx: CallContext, request: PlayerPlayError)
        -> RpcResult<Option<()>>;
}

pub struct PlayerImpl {
    pub platform_state: PlatformState,
}

#[async_trait]
impl PlayerServer for PlayerImpl {
    async fn on_request_load(
        &self,
        ctx: CallContext,
        request: ListenRequest,
    ) -> RpcResult<ListenerResponse> {
        let listen = request.listen;
        ProviderBroker::register_or_unregister_provider(
            &self.platform_state,
            PLAYER_BASE_PROVIDER_CAPABILITY.to_owned(),
            PLAYER_LOAD_METHOD.to_owned(),
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

    async fn load(
        &self,
        ctx: CallContext,
        request: PlayerLoadRequest,
    ) -> RpcResult<PlayerMediaSession> {
        let req = PlayerRequestWithContext {
            request: PlayerRequest::Load(request),
            call_ctx: ctx,
        };

        match self.call_player_provider(req).await? {
            ProviderResponsePayload::PlayerLoad(load_response) => Ok(load_response),
            _ => Err(rpc_err("Invalid response back from provider")),
        }
    }

    async fn load_response(
        &self,
        _ctx: CallContext,
        resp: PlayerLoadResponseParams,
    ) -> RpcResult<Option<()>> {
        self.provider_response(resp.response).await
    }

    async fn load_error(
        &self,
        _ctx: CallContext,
        resp: PlayerErrorResponse,
    ) -> RpcResult<Option<()>> {
        self.provider_response(resp).await
    }

    async fn on_request_play(
        &self,
        ctx: CallContext,
        request: ListenRequest,
    ) -> RpcResult<ListenerResponse> {
        let listen = request.listen;
        ProviderBroker::register_or_unregister_provider(
            &self.platform_state,
            PLAYER_BASE_PROVIDER_CAPABILITY.to_owned(),
            PLAYER_PLAY_METHOD.to_owned(),
            PLAYER_PLAY_EVENT,
            ctx,
            request,
        )
        .await;
        Ok(ListenerResponse {
            listening: listen,
            event: PLAYER_PLAY_EVENT.into(),
        })
    }

    async fn play(
        &self,
        ctx: CallContext,
        request: PlayerPlayRequest,
    ) -> RpcResult<PlayerMediaSession> {
        let req = PlayerRequestWithContext {
            request: PlayerRequest::Play(request),
            call_ctx: ctx,
        };

        match self.call_player_provider(req).await? {
            ProviderResponsePayload::PlayerPlay(play_response) => Ok(play_response), // TODO: spec says this should be Option<()>
            _ => Err(rpc_err("Invalid response back from provider")),
        }
    }

    async fn play_response(
        &self,
        _ctx: CallContext,
        resp: PlayerPlayResponseParams,
    ) -> RpcResult<Option<()>> {
        self.provider_response(resp.response).await
    }

    async fn play_error(&self, _ctx: CallContext, resp: PlayerPlayError) -> RpcResult<Option<()>> {
        self.provider_response(resp).await
    }
}

impl PlayerImpl {
    async fn call_player_provider(
        &self,
        request: PlayerRequestWithContext,
    ) -> RpcResult<ProviderResponsePayload> {
        let method = String::from(request.request.to_provider_method());
        let (session_tx, session_rx) = oneshot::channel::<ProviderResponsePayload>();
        let pr_msg = ProviderBrokerRequest {
            // TODO which capability this rpc method providers should come from firebolt spec
            capability: PLAYER_BASE_PROVIDER_CAPABILITY.to_string(),
            method,
            caller: request.call_ctx.clone().into(),
            request: request.request.to_provider_request_payload(),
            tx: session_tx,
            app_id: None, // TODO: should we be using this?
        };
        ProviderBroker::invoke_method(&self.platform_state, pr_msg).await;
        match session_rx.await {
            Ok(result) => Ok(result),
            Err(_) => Err(rpc_err("Error returning back from player provider")),
        }
    }

    async fn provider_response<T>(&self, resp: T) -> RpcResult<Option<()>>
    where
        T: ToProviderResponse,
    {
        let msg = resp.to_provider_response();
        ProviderBroker::provider_response(&self.platform_state, msg).await;
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
