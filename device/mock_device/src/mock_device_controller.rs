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

use std::fmt::Display;

use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use ripple_sdk::{
    api::{
        gateway::rpc_gateway_api::CallContext,
        mock_websocket_server::{
            AddRequestResponseParams, EmitEventParams, MockWebsocketServerRequest,
            MockWebsocketServerResponse, RemoveRequestParams,
        },
    },
    async_trait::async_trait,
    extn::client::extn_client::ExtnClient,
    log::debug,
    tokio::runtime::Runtime,
    utils::error::RippleError,
};

#[derive(Debug, Clone)]
enum MockDeviceControllerError {
    RequestFailed(RippleError),
    ExtnCommunicationFailed,
}

impl Display for MockDeviceControllerError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::result::Result<(), ::std::fmt::Error> {
        match self.clone() {
            MockDeviceControllerError::RequestFailed(err) => {
                f.write_str(format!("Failed to complete the request. RippleError {err:?}").as_str())
            }
            MockDeviceControllerError::ExtnCommunicationFailed => {
                f.write_str("Failed to communicate with the Mock Device extension")
            }
        }
    }
}

#[rpc(server)]
pub trait MockDeviceController {
    #[method(name = "mockdevice.addRequestResponse")]
    async fn add_request_response(
        &self,
        ctx: CallContext,
        req: AddRequestResponseParams,
    ) -> RpcResult<MockWebsocketServerResponse>;

    #[method(name = "mockdevice.removeRequest")]
    async fn remove_request(
        &self,
        ctx: CallContext,
        req: RemoveRequestParams,
    ) -> RpcResult<MockWebsocketServerResponse>;

    #[method(name = "mockdevice.emitEvent")]
    async fn emit_event(
        &self,
        ctx: CallContext,
        req: EmitEventParams,
    ) -> RpcResult<MockWebsocketServerResponse>;
}

pub struct MockDeviceController {
    client: ExtnClient,
    rt: Runtime,
}

impl MockDeviceController {
    pub fn new(client: ExtnClient) -> MockDeviceController {
        MockDeviceController {
            client,
            rt: Runtime::new().unwrap(),
        }
    }

    async fn request(
        &self,
        request: MockWebsocketServerRequest,
    ) -> Result<MockWebsocketServerResponse, MockDeviceControllerError> {
        debug!("request={request:?}");
        let mut client = self.client.clone();
        self.rt
            .spawn(async move {
                client
                    .standalone_request(request, 5000)
                    .await
                    .map_err(MockDeviceControllerError::RequestFailed)
            })
            .await
            .map_err(|_e| MockDeviceControllerError::ExtnCommunicationFailed)?
    }
}

#[async_trait]
impl MockDeviceControllerServer for MockDeviceController {
    async fn add_request_response(
        &self,
        _ctx: CallContext,
        req: AddRequestResponseParams,
    ) -> RpcResult<MockWebsocketServerResponse> {
        let res = self
            .request(MockWebsocketServerRequest::AddRequestResponse(req))
            .await
            .map_err(|e| jsonrpsee::core::Error::Custom(e.to_string()))?;

        Ok(res)
    }

    async fn remove_request(
        &self,
        _ctx: CallContext,
        req: RemoveRequestParams,
    ) -> RpcResult<MockWebsocketServerResponse> {
        let res = self
            .request(MockWebsocketServerRequest::RemoveRequest(req))
            .await
            .map_err(|e| jsonrpsee::core::Error::Custom(e.to_string()))?;

        Ok(res)
    }

    async fn emit_event(
        &self,
        _ctx: CallContext,
        req: EmitEventParams,
    ) -> RpcResult<MockWebsocketServerResponse> {
        let res = self
            .request(MockWebsocketServerRequest::EmitEvent(req))
            .await
            .map_err(|e| jsonrpsee::core::Error::Custom(e.to_string()))?;

        Ok(res)
    }
}