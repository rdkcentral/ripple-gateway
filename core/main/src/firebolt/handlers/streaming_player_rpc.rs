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
                PlayerErrorResponseParams, PlayerLoadRequest, PlayerLoadResponseParams,
                PlayerMediaSession, PlayerPlayRequest, PlayerPlayResponseParams, PlayerProgress,
                PlayerProgressRequest, PlayerProgressResponseParams, PlayerRequest,
                PlayerRequestWithContext, PlayerStatus, PlayerStatusRequest,
                PlayerStatusResponseParams, PlayerStopRequest, PlayerStopResponseParams,
                StreamingPlayerCreateRequest, StreamingPlayerCreateResponseParams,
                StreamingPlayerInstance, PLAYER_BASE_PROVIDER_CAPABILITY, PLAYER_LOAD_EVENT,
                PLAYER_LOAD_METHOD, PLAYER_PLAY_EVENT, PLAYER_PLAY_METHOD, PLAYER_PROGRESS_EVENT,
                PLAYER_PROGRESS_METHOD, PLAYER_STATUS_EVENT, PLAYER_STATUS_METHOD,
                PLAYER_STOP_EVENT, PLAYER_STOP_METHOD, PLAYER_STREAMING_PROVIDER_CAPABILITY,
                STREAMING_PLAYER_CREATE_EVENT, STREAMING_PLAYER_CREATE_METHOD,
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
pub trait StreamingPlayer {
    #[method(name = "streamingPlayer.onRequestCreate")]
    async fn on_request_streaming_player_create(
        &self,
        ctx: CallContext,
        request: ListenRequest,
    ) -> RpcResult<ListenerResponse>;

    #[method(name = "streamingPlayer.create")]
    async fn streaming_player_create(
        &self,
        ctx: CallContext,
        request: StreamingPlayerCreateRequest,
    ) -> RpcResult<StreamingPlayerInstance>;

    #[method(name = "streamingPlayer.createResponse")]
    async fn streaming_player_create_response(
        &self,
        ctx: CallContext,
        request: StreamingPlayerCreateResponseParams,
    ) -> RpcResult<Option<()>>;

    #[method(name = "streamingPlayer.createError")]
    async fn streaming_player_create_error(
        &self,
        ctx: CallContext,
        request: PlayerErrorResponseParams,
    ) -> RpcResult<Option<()>>;
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

        match self
            .call_player_provider(req, PLAYER_BASE_PROVIDER_CAPABILITY)
            .await?
        {
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
        resp: PlayerErrorResponseParams,
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

        match self
            .call_player_provider(req, PLAYER_BASE_PROVIDER_CAPABILITY)
            .await?
        {
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

    async fn play_error(
        &self,
        _ctx: CallContext,
        resp: PlayerErrorResponseParams,
    ) -> RpcResult<Option<()>> {
        self.provider_response(resp).await
    }

    async fn on_request_stop(
        &self,
        ctx: CallContext,
        request: ListenRequest,
    ) -> RpcResult<ListenerResponse> {
        let listen = request.listen;
        ProviderBroker::register_or_unregister_provider(
            &self.platform_state,
            PLAYER_BASE_PROVIDER_CAPABILITY.to_owned(),
            PLAYER_STOP_METHOD.to_owned(),
            PLAYER_STOP_EVENT,
            ctx,
            request,
        )
        .await;
        Ok(ListenerResponse {
            listening: listen,
            event: PLAYER_STOP_EVENT.into(),
        })
    }

    async fn stop(
        &self,
        ctx: CallContext,
        request: PlayerStopRequest,
    ) -> RpcResult<PlayerMediaSession> {
        let req = PlayerRequestWithContext {
            request: PlayerRequest::Stop(request),
            call_ctx: ctx,
        };

        match self
            .call_player_provider(req, PLAYER_BASE_PROVIDER_CAPABILITY)
            .await?
        {
            ProviderResponsePayload::PlayerStop(stop_response) => Ok(stop_response), // TODO: spec says this should be Option<()>
            _ => Err(rpc_err("Invalid response back from provider")),
        }
    }

    async fn stop_response(
        &self,
        _ctx: CallContext,
        resp: PlayerStopResponseParams,
    ) -> RpcResult<Option<()>> {
        self.provider_response(resp.response).await
    }

    async fn stop_error(
        &self,
        _ctx: CallContext,
        resp: PlayerErrorResponseParams,
    ) -> RpcResult<Option<()>> {
        self.provider_response(resp).await
    }

    async fn on_request_status(
        &self,
        ctx: CallContext,
        request: ListenRequest,
    ) -> RpcResult<ListenerResponse> {
        let listen = request.listen;
        ProviderBroker::register_or_unregister_provider(
            &self.platform_state,
            PLAYER_BASE_PROVIDER_CAPABILITY.to_owned(),
            PLAYER_STATUS_METHOD.to_owned(),
            PLAYER_STATUS_EVENT,
            ctx,
            request,
        )
        .await;
        Ok(ListenerResponse {
            listening: listen,
            event: PLAYER_STATUS_EVENT.into(),
        })
    }

    async fn status(
        &self,
        ctx: CallContext,
        request: PlayerStatusRequest,
    ) -> RpcResult<PlayerStatus> {
        let req = PlayerRequestWithContext {
            request: PlayerRequest::Status(request),
            call_ctx: ctx,
        };

        match self
            .call_player_provider(req, PLAYER_BASE_PROVIDER_CAPABILITY)
            .await?
        {
            ProviderResponsePayload::PlayerStatus(status_response) => Ok(status_response),
            _ => Err(rpc_err("Invalid response back from provider")),
        }
    }

    async fn status_response(
        &self,
        _ctx: CallContext,
        resp: PlayerStatusResponseParams,
    ) -> RpcResult<Option<()>> {
        self.provider_response(resp.response).await
    }

    async fn status_error(
        &self,
        _ctx: CallContext,
        resp: PlayerErrorResponseParams,
    ) -> RpcResult<Option<()>> {
        self.provider_response(resp).await
    }

    async fn on_request_progress(
        &self,
        ctx: CallContext,
        request: ListenRequest,
    ) -> RpcResult<ListenerResponse> {
        let listen = request.listen;
        ProviderBroker::register_or_unregister_provider(
            &self.platform_state,
            PLAYER_BASE_PROVIDER_CAPABILITY.to_owned(),
            PLAYER_PROGRESS_METHOD.to_owned(),
            PLAYER_PROGRESS_EVENT,
            ctx,
            request,
        )
        .await;
        Ok(ListenerResponse {
            listening: listen,
            event: PLAYER_PROGRESS_EVENT.into(),
        })
    }

    async fn progress(
        &self,
        ctx: CallContext,
        request: PlayerProgressRequest,
    ) -> RpcResult<PlayerProgress> {
        let req = PlayerRequestWithContext {
            request: PlayerRequest::Progress(request),
            call_ctx: ctx,
        };

        match self
            .call_player_provider(req, PLAYER_BASE_PROVIDER_CAPABILITY)
            .await?
        {
            ProviderResponsePayload::PlayerProgress(progress_response) => Ok(progress_response),
            _ => Err(rpc_err("Invalid response back from provider")),
        }
    }

    async fn progress_response(
        &self,
        _ctx: CallContext,
        resp: PlayerProgressResponseParams,
    ) -> RpcResult<Option<()>> {
        self.provider_response(resp.response).await
    }

    async fn progress_error(
        &self,
        _ctx: CallContext,
        resp: PlayerErrorResponseParams,
    ) -> RpcResult<Option<()>> {
        self.provider_response(resp).await
    }

    async fn on_request_streaming_player_create(
        &self,
        ctx: CallContext,
        request: ListenRequest,
    ) -> RpcResult<ListenerResponse> {
        let listen = request.listen;
        ProviderBroker::register_or_unregister_provider(
            &self.platform_state,
            PLAYER_STREAMING_PROVIDER_CAPABILITY.to_owned(),
            STREAMING_PLAYER_CREATE_METHOD.to_owned(),
            STREAMING_PLAYER_CREATE_EVENT,
            ctx,
            request,
        )
        .await;
        Ok(ListenerResponse {
            listening: listen,
            event: STREAMING_PLAYER_CREATE_EVENT.into(),
        })
    }

    async fn streaming_player_create(
        &self,
        ctx: CallContext,
        request: StreamingPlayerCreateRequest,
    ) -> RpcResult<StreamingPlayerInstance> {
        let req = PlayerRequestWithContext {
            request: PlayerRequest::StreamingPlayerCreate(request),
            call_ctx: ctx,
        };

        match self
            .call_player_provider(req, PLAYER_STREAMING_PROVIDER_CAPABILITY)
            .await?
        {
            ProviderResponsePayload::StreamingPlayerCreate(instance) => Ok(instance),
            _ => Err(rpc_err("Invalid response back from provider")),
        }
    }

    async fn streaming_player_create_response(
        &self,
        _ctx: CallContext,
        resp: StreamingPlayerCreateResponseParams,
    ) -> RpcResult<Option<()>> {
        self.provider_response(resp.response).await
    }

    async fn streaming_player_create_error(
        &self,
        _ctx: CallContext,
        resp: PlayerErrorResponseParams,
    ) -> RpcResult<Option<()>> {
        self.provider_response(resp).await
    }
}

impl PlayerImpl {
    async fn call_player_provider(
        &self,
        request: PlayerRequestWithContext,
        capability: &str,
    ) -> RpcResult<ProviderResponsePayload> {
        let method = String::from(request.request.to_provider_method());
        let (session_tx, session_rx) = oneshot::channel::<ProviderResponsePayload>();
        let pr_msg = ProviderBrokerRequest {
            // TODO which capability this rpc method providers should come from firebolt spec
            capability: capability.to_string(),
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
