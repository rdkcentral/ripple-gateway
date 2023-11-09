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
                PlayerErrorResponse, PlayerIdListenRequest, PlayerLoadRequestParams,
                PlayerLoadResponse, PlayerMediaSession, PlayerPlayRequest, PlayerPlayResponse,
                PlayerProgress, PlayerProgressRequest, PlayerProgressResponse,
                PlayerProvideProgress, PlayerProvideStatus, PlayerRequest,
                PlayerRequestWithContext, PlayerStatus, PlayerStatusRequest, PlayerStatusResponse,
                PlayerStopRequest, PlayerStopResponse, PLAYER_BASE_PROVIDER_CAPABILITY,
                PLAYER_LOAD_EVENT, PLAYER_LOAD_METHOD, PLAYER_ON_PROGRESS_CHANGED_EVENT,
                PLAYER_ON_STATUS_CHANGED_EVENT, PLAYER_PLAY_EVENT, PLAYER_PLAY_METHOD,
                PLAYER_PROGRESS_EVENT, PLAYER_PROGRESS_METHOD, PLAYER_STATUS_EVENT,
                PLAYER_STATUS_METHOD, PLAYER_STOP_EVENT, PLAYER_STOP_METHOD,
            },
            provider::{ProviderResponsePayload, ToProviderResponse},
        },
        gateway::rpc_gateway_api::CallContext,
    },
    async_trait::async_trait,
    log::debug,
    tokio::sync::oneshot,
    utils::rpc_utils::rpc_err,
};
use serde_json::{json, Value};

use crate::{
    firebolt::rpc::RippleRPCProvider,
    service::apps::{
        app_events::{AppEventDecorationError, AppEventDecorator, AppEvents},
        provider_broker::{ProviderBroker, ProviderBrokerRequest},
    },
    state::platform_state::PlatformState,
    utils::rpc_utils::rpc_add_event_listener_with_decorator,
};

#[derive(Clone)]
struct PlayerIdEventDecorator {
    // player_id: String,
}

#[async_trait]
impl AppEventDecorator for PlayerIdEventDecorator {
    async fn decorate(
        &self,
        _ps: &PlatformState,
        _ctx: &CallContext,
        _event_name: &str,
        val_in: &Value,
    ) -> Result<Value, AppEventDecorationError> {
        debug!("decorating {} {} {:?}", _event_name, val_in, _ctx);
        Ok(json!({ "playerId": val_in }))
    }

    fn dec_clone(&self) -> Box<dyn AppEventDecorator + Send + Sync> {
        Box::new(self.clone())
    }
}

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
        request: PlayerLoadRequestParams,
    ) -> RpcResult<PlayerMediaSession>;

    #[method(name = "player.loadResponse")]
    async fn load_response(
        &self,
        ctx: CallContext,
        request: PlayerLoadResponse,
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
        request: PlayerPlayResponse,
    ) -> RpcResult<Option<()>>;

    #[method(name = "player.playError")]
    async fn play_error(
        &self,
        ctx: CallContext,
        request: PlayerErrorResponse,
    ) -> RpcResult<Option<()>>;

    #[method(name = "player.onRequestStop")]
    async fn on_request_stop(
        &self,
        ctx: CallContext,
        request: ListenRequest,
    ) -> RpcResult<ListenerResponse>;

    #[method(name = "player.stop")]
    async fn stop(
        &self,
        ctx: CallContext,
        request: PlayerStopRequest,
    ) -> RpcResult<PlayerMediaSession>;

    #[method(name = "player.stopResponse")]
    async fn stop_response(
        &self,
        ctx: CallContext,
        request: PlayerStopResponse,
    ) -> RpcResult<Option<()>>;

    #[method(name = "player.stopError")]
    async fn stop_error(
        &self,
        ctx: CallContext,
        request: PlayerErrorResponse,
    ) -> RpcResult<Option<()>>;

    #[method(name = "player.onRequestStatus")]
    async fn on_request_status(
        &self,
        ctx: CallContext,
        request: ListenRequest,
    ) -> RpcResult<ListenerResponse>;

    #[method(name = "player.status")]
    async fn status(
        &self,
        ctx: CallContext,
        request: PlayerStatusRequest,
    ) -> RpcResult<PlayerStatus>;

    #[method(name = "player.statusResponse")]
    async fn status_response(
        &self,
        ctx: CallContext,
        request: PlayerStatusResponse,
    ) -> RpcResult<Option<()>>;

    #[method(name = "player.statusError")]
    async fn status_error(
        &self,
        ctx: CallContext,
        request: PlayerErrorResponse,
    ) -> RpcResult<Option<()>>;

    #[method(name = "player.onRequestProgress")]
    async fn on_request_progress(
        &self,
        ctx: CallContext,
        request: ListenRequest,
    ) -> RpcResult<ListenerResponse>;

    #[method(name = "player.progress")]
    async fn progress(
        &self,
        ctx: CallContext,
        request: PlayerProgressRequest,
    ) -> RpcResult<PlayerProgress>;

    #[method(name = "player.progressResponse")]
    async fn progress_response(
        &self,
        ctx: CallContext,
        request: PlayerProgressResponse,
    ) -> RpcResult<Option<()>>;

    #[method(name = "player.progressError")]
    async fn progress_error(
        &self,
        ctx: CallContext,
        request: PlayerErrorResponse,
    ) -> RpcResult<Option<()>>;

    #[method(name = "player.onProgressChanged")]
    async fn on_progress_changed(
        &self,
        ctx: CallContext,
        request: PlayerIdListenRequest,
    ) -> RpcResult<ListenerResponse>;

    #[method(name = "player.provideProgress")]
    async fn provide_progress(
        &self,
        ctx: CallContext,
        request: PlayerProvideProgress,
    ) -> RpcResult<()>;

    #[method(name = "player.onStatusChanged")]
    async fn on_status_changed(
        &self,
        ctx: CallContext,
        request: PlayerIdListenRequest,
    ) -> RpcResult<ListenerResponse>;

    #[method(name = "player.provideStatus")]
    async fn provide_status(&self, ctx: CallContext, request: PlayerProvideStatus)
        -> RpcResult<()>;
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
        request: PlayerLoadRequestParams,
    ) -> RpcResult<PlayerMediaSession> {
        let req = PlayerRequestWithContext {
            request: PlayerRequest::Load(request.request),
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
        resp: PlayerLoadResponse,
    ) -> RpcResult<Option<()>> {
        self.provider_response(resp).await
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

        match self
            .call_player_provider(req, PLAYER_BASE_PROVIDER_CAPABILITY)
            .await?
        {
            ProviderResponsePayload::PlayerPlay(play_response) => Ok(play_response), // TODO: spec says this should be Option<()> - KP said he will change the spec
            _ => Err(rpc_err("Invalid response back from provider")),
        }
    }

    async fn play_response(
        &self,
        _ctx: CallContext,
        resp: PlayerPlayResponse,
    ) -> RpcResult<Option<()>> {
        self.provider_response(resp).await
    }

    async fn play_error(
        &self,
        _ctx: CallContext,
        resp: PlayerErrorResponse,
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
        resp: PlayerStopResponse,
    ) -> RpcResult<Option<()>> {
        self.provider_response(resp).await
    }

    async fn stop_error(
        &self,
        _ctx: CallContext,
        resp: PlayerErrorResponse,
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
        resp: PlayerStatusResponse,
    ) -> RpcResult<Option<()>> {
        self.provider_response(resp).await
    }

    async fn status_error(
        &self,
        _ctx: CallContext,
        resp: PlayerErrorResponse,
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
        resp: PlayerProgressResponse,
    ) -> RpcResult<Option<()>> {
        self.provider_response(resp).await
    }

    async fn progress_error(
        &self,
        _ctx: CallContext,
        resp: PlayerErrorResponse,
    ) -> RpcResult<Option<()>> {
        self.provider_response(resp).await
    }

    async fn on_progress_changed(
        &self,
        ctx: CallContext,
        request: PlayerIdListenRequest,
    ) -> RpcResult<ListenerResponse> {
        debug!("opc {:?} {:?}", ctx, request);
        rpc_add_event_listener_with_decorator(
            &self.platform_state,
            ctx,
            request.into(),
            PLAYER_ON_PROGRESS_CHANGED_EVENT,
            Some(Box::new(PlayerIdEventDecorator {})),
        )
        .await
    }

    async fn provide_progress(
        &self,
        _ctx: CallContext,
        request: PlayerProvideProgress,
    ) -> RpcResult<()> {
        AppEvents::emit(
            &self.platform_state,
            PLAYER_ON_PROGRESS_CHANGED_EVENT,
            &serde_json::to_value(request)?,
        )
        .await;

        Ok(())
    }

    async fn on_status_changed(
        &self,
        ctx: CallContext,
        request: PlayerIdListenRequest,
    ) -> RpcResult<ListenerResponse> {
        rpc_add_event_listener_with_decorator(
            &self.platform_state,
            ctx,
            request.into(),
            PLAYER_ON_STATUS_CHANGED_EVENT,
            Some(Box::new(PlayerIdEventDecorator {})),
        )
        .await
    }

    async fn provide_status(
        &self,
        _ctx: CallContext,
        request: PlayerProvideStatus,
    ) -> RpcResult<()> {
        AppEvents::emit(
            &self.platform_state,
            PLAYER_ON_STATUS_CHANGED_EVENT,
            &serde_json::to_value(request)?,
        )
        .await;

        Ok(())
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
