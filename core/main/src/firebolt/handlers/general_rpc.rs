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

use std::sync::Arc;

use crate::{
    firebolt::rpc::RippleRPCProvider, service::apps::provider_broker::ProviderBroker,
    state::platform_state::PlatformState,
};
use jsonrpsee::core::server::rpc_module::Methods;
use jsonrpsee::{core::RpcResult, proc_macros::rpc, RpcModule};
use ripple_sdk::async_trait::async_trait;
use ripple_sdk::{
    api::{
        firebolt::{
            fb_general::{ListenRequest, ListenerResponse},
            provider::{
                ChallengeError, ChallengeResponse, ExternalProviderResponse, FocusRequest,
                ProviderResponse, ProviderResponsePayload, ACK_CHALLENGE_CAPABILITY,
                ACK_CHALLENGE_EVENT,
            },
        },
        gateway::rpc_gateway_api::CallContext,
    },
    log::debug,
};

pub struct GeneralChallengeImpl {
    pub platform_state: PlatformState,
}

impl GeneralChallengeImpl {
    async fn on_request_challenge(
        &self,
        ctx: CallContext,
        request: ListenRequest,
    ) -> RpcResult<ListenerResponse> {
        let listen = request.listen;
        debug!("GeneralChallenge provider registered :{:?}", request);
        ProviderBroker::register_or_unregister_provider(
            &self.platform_state,
            String::from(ACK_CHALLENGE_CAPABILITY),
            String::from("challenge"),
            ACK_CHALLENGE_EVENT,
            ctx,
            request,
        )
        .await;

        Ok(ListenerResponse {
            listening: listen,
            event: ACK_CHALLENGE_EVENT.into(),
        })
    }

    async fn challenge_response(
        &self,
        _ctx: CallContext,
        resp: ExternalProviderResponse<ChallengeResponse>,
    ) -> RpcResult<Option<()>> {
        ProviderBroker::provider_response(
            &self.platform_state,
            ProviderResponse {
                correlation_id: resp.correlation_id,
                result: ProviderResponsePayload::ChallengeResponse(resp.result),
            },
        )
        .await;
        Ok(None)
    }

    async fn challenge_error(
        &self,
        _ctx: CallContext,
        resp: ExternalProviderResponse<ChallengeError>,
    ) -> RpcResult<Option<()>> {
        ProviderBroker::provider_response(
            &self.platform_state,
            ProviderResponse {
                correlation_id: resp.correlation_id,
                result: ProviderResponsePayload::ChallengeError(resp.result),
            },
        )
        .await;
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
            ACK_CHALLENGE_CAPABILITY.to_string(),
            request,
        )
        .await;
        Ok(None)
    }
}

pub struct GeneralChallengeRPCProvider;

impl RippleRPCProvider<GeneralChallengeImpl> for GeneralChallengeRPCProvider {
    fn provide(state: PlatformState) -> RpcModule<GeneralChallengeImpl> {
        RpcModule::new(GeneralChallengeImpl {
            platform_state: state.clone(),
        })
    }
}
