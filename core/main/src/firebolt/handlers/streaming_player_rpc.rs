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
                PlayerErrorResponse, StreamingPlayerCreateRequest, StreamingPlayerCreateResponse,
                StreamingPlayerInstance, StreamingPlayerRequest, StreamingPlayerRequestWithContext,
                PLAYER_STREAMING_PROVIDER_CAPABILITY, STREAMING_PLAYER_CREATE_EVENT,
                STREAMING_PLAYER_CREATE_METHOD,
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

use crate::{
    firebolt::rpc::RippleRPCProvider,
    service::apps::provider_broker::{ProviderBroker, ProviderBrokerRequest},
    state::platform_state::PlatformState,
};

#[rpc(server)]
pub trait StreamingPlayer {
    #[method(name = "streamingplayer.onRequestCreate")]
    async fn on_request_streaming_player_create(
        &self,
        ctx: CallContext,
        request: ListenRequest,
    ) -> RpcResult<ListenerResponse>;

    #[method(name = "streamingplayer.create")]
    async fn streaming_player_create(&self, ctx: CallContext)
        -> RpcResult<StreamingPlayerInstance>;

    #[method(name = "streamingplayer.createResponse")]
    async fn streaming_player_create_response(
        &self,
        ctx: CallContext,
        request: StreamingPlayerCreateResponse,
    ) -> RpcResult<Option<()>>;

    #[method(name = "streamingplayer.createError")]
    async fn streaming_player_create_error(
        &self,
        ctx: CallContext,
        request: PlayerErrorResponse,
    ) -> RpcResult<Option<()>>;
}

pub struct StreamingPlayerImpl {
    pub platform_state: PlatformState,
}

#[async_trait]
impl StreamingPlayerServer for StreamingPlayerImpl {
    async fn on_request_streaming_player_create(
        &self,
        ctx: CallContext,
        request: ListenRequest,
    ) -> RpcResult<ListenerResponse> {
        let listen = request.listen;
        debug!(
            "consts {} {}",
            STREAMING_PLAYER_CREATE_EVENT, PLAYER_STREAMING_PROVIDER_CAPABILITY
        );
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
    ) -> RpcResult<StreamingPlayerInstance> {
        let req = StreamingPlayerRequestWithContext {
            request: StreamingPlayerRequest::Create(StreamingPlayerCreateRequest),
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
        ctx: CallContext,
        resp: StreamingPlayerCreateResponse,
    ) -> RpcResult<Option<()>> {
        let mut resp = resp.clone();
        resp.result.prefix_app_id_to_player_id(&ctx.app_id);
        self.provider_response(resp).await
    }

    async fn streaming_player_create_error(
        &self,
        _ctx: CallContext,
        resp: PlayerErrorResponse,
    ) -> RpcResult<Option<()>> {
        self.provider_response(resp).await
    }
}

impl StreamingPlayerImpl {
    async fn call_player_provider(
        &self,
        request: StreamingPlayerRequestWithContext,
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
            app_id: None,
        };
        ProviderBroker::invoke_method(&self.platform_state, pr_msg).await;
        match session_rx.await {
            Ok(result) => Ok(result),
            Err(_) => Err(rpc_err("Error returning back from player provider")), // TODO: print the error
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

pub struct StreamingPlayerRPCProvider;

impl RippleRPCProvider<StreamingPlayerImpl> for StreamingPlayerRPCProvider {
    fn provide(state: PlatformState) -> RpcModule<StreamingPlayerImpl> {
        (StreamingPlayerImpl {
            platform_state: state,
        })
        .into_rpc()
    }
}
