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

use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::{mpsc, oneshot};

use crate::{
    api::firebolt::{fb_general::ListenRequest, fb_openrpc::FireboltOpenRpcMethod},
    extn::extn_client_message::{ExtnPayload, ExtnPayloadProvider, ExtnRequest},
    framework::ripple_contract::RippleContract,
};

#[derive(Debug, Clone, Default)]
pub struct CallerSession {
    pub session_id: Option<String>,
    pub app_id: Option<String>,
}

impl From<CallContext> for CallerSession {
    fn from(ctx: CallContext) -> Self {
        CallerSession {
            session_id: Some(ctx.session_id),
            app_id: Some(ctx.app_id),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppIdentification {
    pub app_id: String,
}

impl From<CallContext> for AppIdentification {
    fn from(ctx: CallContext) -> Self {
        AppIdentification { app_id: ctx.app_id }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct CallContext {
    pub session_id: String,
    pub request_id: String,
    pub app_id: String,
    pub call_id: u64,
    pub protocol: ApiProtocol,
    pub method: String,
    pub cid: Option<String>,
    pub gateway_secure: bool,
}

impl CallContext {
    // TODO: refactor this to use less arguments
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        session_id: String,
        request_id: String,
        app_id: String,
        call_id: u64,
        protocol: ApiProtocol,
        method: String,
        cid: Option<String>,
        gateway_secure: bool,
    ) -> CallContext {
        CallContext {
            session_id,
            request_id,
            app_id,
            call_id,
            protocol,
            method,
            cid,
            gateway_secure,
        }
    }

    pub fn get_id(&self) -> String {
        if let Some(cid) = &self.cid {
            return cid.clone();
        }
        self.session_id.clone()
    }
}

impl crate::Mockable for CallContext {
    fn mock() -> Self {
        CallContext {
            session_id: "session_id".to_owned(),
            request_id: "1".to_owned(),
            app_id: "some_app_id".to_owned(),
            call_id: 1,
            protocol: ApiProtocol::JsonRpc,
            method: "module.method".to_owned(),
            cid: Some("cid".to_owned()),
            gateway_secure: true,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ApiProtocol {
    Bridge,
    Extn,
    JsonRpc,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ApiMessage {
    pub protocol: ApiProtocol,
    pub jsonrpc_msg: String,
    pub request_id: String,
}

/// Holds a message in jsonrpc protocol format and the protocol that it should be converted into
/// The protocol of the request is passed in context and then when
/// a response is being written, that protocol is passed here so the transport layers know which protocol
/// should be used for the response.
impl ApiMessage {
    pub fn new(protocol: ApiProtocol, jsonrpc_msg: String, request_id: String) -> ApiMessage {
        ApiMessage {
            protocol,
            jsonrpc_msg,
            request_id,
        }
    }

    pub fn is_error(&self) -> bool {
        // currently only these json rpsee errors are used in Ripple
        self.jsonrpc_msg.contains("Custom error:") || self.jsonrpc_msg.contains("Method not found")
    }
}

#[derive(Deserialize)]
struct ApiBaseRequest {
    jsonrpc: Option<String>,
}

impl ApiBaseRequest {
    fn is_jsonrpc(&self) -> bool {
        self.jsonrpc.is_some()
    }
}

#[derive(Serialize, Deserialize)]
pub struct JsonRpcApiRequest {
    pub jsonrpc: String,
    pub id: Option<u64>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcApiResponse {
    pub jsonrpc: String,
    pub id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
    #[serde(skip_serializing)]
    pub method: Option<String>,
    #[serde(skip_serializing)]
    pub params: Option<Value>,
}

impl crate::Mockable for JsonRpcApiResponse {
    fn mock() -> Self {
        JsonRpcApiResponse {
            jsonrpc: "2.0".to_owned(),
            result: None,
            id: None,
            error: None,
            method: None,
            params: None,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct RpcRequest {
    pub method: String,
    pub params_json: String,
    pub ctx: CallContext,
}

impl ExtnPayloadProvider for RpcRequest {
    fn get_extn_payload(&self) -> ExtnPayload {
        ExtnPayload::Request(ExtnRequest::Rpc(self.clone()))
    }

    fn get_from_payload(payload: ExtnPayload) -> Option<Self> {
        if let ExtnPayload::Request(ExtnRequest::Rpc(v)) = payload {
            return Some(v);
        }
        None
    }

    fn contract() -> RippleContract {
        RippleContract::Rpc
    }
}

impl crate::Mockable for RpcRequest {
    fn mock() -> Self {
        RpcRequest {
            method: "module.method".to_owned(),
            params_json: "{}".to_owned(),
            ctx: CallContext::mock(),
        }
    }
}

#[derive(Debug)]
pub struct RequestParseError {}

impl RpcRequest {
    pub fn new(method: String, params_json: String, ctx: CallContext) -> RpcRequest {
        RpcRequest {
            method,
            params_json,
            ctx,
        }
    }
    /// Serializes a parameter so that the given ctx becomes the first list in a json array of
    /// parameters. Each rpc handler will get the call context as the first param and
    /// the actual request parameter as the second param.
    ///
    /// # Arguments
    ///
    /// * `req_params` - The request parameter that becomes the second handler parameter
    /// * `ctx` - Context around the call which becomes the first handler parameter
    pub fn prepend_ctx(req_params: Option<Value>, ctx: &CallContext) -> String {
        let mut ps = Vec::<Value>::new();
        ps.push(json!(ctx));
        if let Some(params) = req_params {
            ps.push(json!(params));
        }
        json!(ps).to_string()
    }

    /// Parses a json string into an RpcRequest
    /// Checks if jsonrpc field is present in order to determine
    /// which protocol this message is using.
    /// Parse the different protocol messages into a common request object
    /// # Arguments
    ///
    /// * `json` - The json string to parse
    /// * `app_id` - The app_id this message was from, used to populate the context
    /// * `session_id` - The session_id this message was from, used to populate the context
    pub fn parse(
        json: String,
        app_id: String,
        session_id: String,
        request_id: String,
        cid: Option<String>,
        gateway_secure: bool,
    ) -> Result<RpcRequest, RequestParseError> {
        let parsed_res = serde_json::from_str(&json);
        if parsed_res.is_err() {
            return Err(RequestParseError {});
        }
        let parsed: serde_json::Value = parsed_res.unwrap();
        let base_res = serde_json::from_value(parsed.clone());
        if base_res.is_err() {
            return Err(RequestParseError {});
        }
        let base: ApiBaseRequest = base_res.unwrap();
        if !base.is_jsonrpc() {
            return Err(RequestParseError {});
        }
        let jsonrpc_req_res = serde_json::from_value(parsed);
        if jsonrpc_req_res.is_err() {
            return Err(RequestParseError {});
        }
        let jsonrpc_req: JsonRpcApiRequest = jsonrpc_req_res.unwrap();
        let id = jsonrpc_req.id.unwrap_or(0);
        let method = FireboltOpenRpcMethod::name_with_lowercase_module(&jsonrpc_req.method);
        let ctx = CallContext::new(
            session_id,
            request_id,
            app_id,
            id,
            ApiProtocol::JsonRpc,
            method.clone(),
            cid,
            gateway_secure,
        );
        let ps = RpcRequest::prepend_ctx(jsonrpc_req.params, &ctx);
        Ok(RpcRequest::new(method, ps, ctx))
    }

    pub fn is_subscription(&self) -> bool {
        self.method.contains(".on") && self.params_json.contains("listen")
    }

    pub fn is_listening(&self) -> bool {
        if let Some(params) = self.get_params() {
            debug!("Successfully got params {:?}", params);
            if let Ok(v) = serde_json::from_value::<ListenRequest>(params) {
                debug!("Successfully got listen request {:?}", v);
                return v.listen;
            }
        }
        false
    }

    pub fn get_params(&self) -> Option<Value> {
        if let Ok(mut v) = serde_json::from_str::<Vec<Value>>(&self.params_json) {
            if v.len() > 1 {
                return v.pop();
            }
        }
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RpcGatewayCommand {
    Handle {
        req: String,
        req_id: String,
        ctx: ClientContext,
    },
    Route {
        req: RpcRequest,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClientContext {
    pub session_id: String,
    pub app_id: String,
    pub gateway_secure: bool,
}

#[derive(Debug)]
pub enum PermissionCommand {
    GateRequest {
        req: RpcRequest,
        route_tx: oneshot::Sender<bool>,
        session_tx: mpsc::Sender<ApiMessage>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::gateway::rpc_gateway_api::{ApiProtocol, CallContext};
    use crate::utils::test_utils::test_extn_payload_provider;

    #[test]
    fn test_extn_request_rpc() {
        let call_context = CallContext {
            session_id: "test_session_id".to_string(),
            request_id: "test_request_id".to_string(),
            app_id: "test_app_id".to_string(),
            call_id: 123,
            protocol: ApiProtocol::Bridge,
            method: "some_method".to_string(),
            cid: Some("some_cid".to_string()),
            gateway_secure: true,
        };

        let rpc_request = RpcRequest {
            method: "some_method".to_string(),
            params_json: r#"{"key": "value"}"#.to_string(),
            ctx: call_context,
        };
        let contract_type: RippleContract = RippleContract::Rpc;
        test_extn_payload_provider(rpc_request, contract_type);
    }
}
