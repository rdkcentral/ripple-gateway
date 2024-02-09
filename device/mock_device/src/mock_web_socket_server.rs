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
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
    time::Duration,
};

use http::{HeaderMap, StatusCode};
use ripple_sdk::{
    api::gateway::rpc_gateway_api::JsonRpcApiRequest,
    futures::{stream::SplitSink, SinkExt, StreamExt},
    log::{debug, error, warn},
    tokio::{
        self,
        net::{TcpListener, TcpStream},
        sync::Mutex,
    },
};
use serde_json::{json, Value};
use tokio_tungstenite::{
    accept_hdr_async,
    tungstenite::{handshake, Error, Message, Result},
    WebSocketStream,
};

use crate::{
    errors::MockServerWebSocketError,
    mock_config::MockConfig,
    mock_data::{MockData, MockDataError, ParamResponse, ResponseSink},
    utils::is_value_jsonrpc,
};

#[derive(Clone, Debug, PartialEq)]
pub struct WsServerParameters {
    path: Option<String>,

    headers: Option<HeaderMap>,

    query_params: Option<HashMap<String, String>>,

    port: Option<u16>,
}

impl WsServerParameters {
    pub fn new() -> Self {
        Self {
            path: None,
            headers: None,
            query_params: None,
            port: None,
        }
    }
    pub fn path(&mut self, path: &str) -> &mut Self {
        self.path = Some(path.into());

        self
    }
    pub fn headers(&mut self, headers: HeaderMap) -> &mut Self {
        self.headers = Some(headers);

        self
    }
    pub fn query_params(&mut self, query_params: HashMap<String, String>) -> &mut Self {
        self.query_params = Some(query_params);

        self
    }
    pub fn port(&mut self, port: u16) -> &mut Self {
        self.port = Some(port);

        self
    }
}

impl Default for WsServerParameters {
    fn default() -> Self {
        Self::new()
    }
}

type WSConnection = Arc<Mutex<HashMap<String, SplitSink<WebSocketStream<TcpStream>, Message>>>>;

#[derive(Debug)]
pub struct MockWebSocketServer {
    mock_data_v2: Arc<RwLock<MockData>>,

    listener: TcpListener,

    conn_path: String,

    conn_headers: HeaderMap,

    conn_query_params: HashMap<String, String>,

    port: u16,

    connected_peer_sinks: WSConnection,

    config: MockConfig,
}

impl MockWebSocketServer {
    pub async fn new(
        mock_data_v2: MockData,
        server_config: WsServerParameters,
        config: MockConfig,
    ) -> Result<Self, MockServerWebSocketError> {
        let listener = Self::create_listener(server_config.port.unwrap_or(0)).await?;
        let port = listener
            .local_addr()
            .map_err(|_| MockServerWebSocketError::CantListen)?
            .port();

        Ok(Self {
            listener,
            port,
            conn_path: server_config.path.unwrap_or_else(|| "/".to_string()),
            conn_headers: server_config.headers.unwrap_or_else(HeaderMap::new),
            conn_query_params: server_config.query_params.unwrap_or_default(),
            connected_peer_sinks: Arc::new(Mutex::new(HashMap::new())),
            config,
            mock_data_v2: Arc::new(RwLock::new(mock_data_v2)),
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    async fn create_listener(port: u16) -> Result<TcpListener, MockServerWebSocketError> {
        let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|_| MockServerWebSocketError::CantListen)?;
        debug!("Listening on: {:?}", listener.local_addr().unwrap());

        Ok(listener)
    }

    pub fn into_arc(self) -> Arc<Self> {
        Arc::new(self)
    }

    pub async fn start_server(self: Arc<Self>) {
        debug!("Waiting for connections");

        while let Ok((stream, peer_addr)) = self.listener.accept().await {
            let server = self.clone();
            tokio::spawn(async move {
                server.accept_connection(peer_addr, stream).await;
            });
        }

        debug!("Shutting down");
    }

    async fn accept_connection(&self, peer: SocketAddr, stream: TcpStream) {
        debug!("Peer address: {}", peer);
        let connection = self.handle_connection(peer, stream).await;

        if let Err(e) = connection {
            match e {
                Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
                err => error!("Error processing connection: {:?}", err),
            }
        }
    }

    async fn handle_connection(&self, peer: SocketAddr, stream: TcpStream) -> Result<()> {
        let callback = |request: &handshake::client::Request,
                        mut response: handshake::server::Response| {
            let path = request.uri().path();
            if path != self.conn_path {
                *response.status_mut() = StatusCode::NOT_FOUND;
                debug!("Connection response {:?}", response);
            }

            if !self.conn_headers.iter().all(|(header_name, header_value)| {
                request.headers().get(header_name) == Some(header_value)
            }) {
                *response.status_mut() = StatusCode::BAD_REQUEST;
                error!("Incompatible headers. Headers required by server: {:?}. Headers sent in request: {:?}", self.conn_headers, request.headers());
                debug!("Connection response {:?}", response);
            }

            let request_query =
                url::form_urlencoded::parse(request.uri().query().unwrap_or("").as_bytes())
                    .into_owned()
                    .collect::<HashMap<String, String>>();

            let eq_num_params = self.conn_query_params.len() == request_query.len();
            let all_params_match =
                self.conn_query_params
                    .iter()
                    .all(|(param_name, param_value)| {
                        request_query.get(param_name) == Some(param_value)
                    });

            if !(eq_num_params && all_params_match) {
                *response.status_mut() = StatusCode::BAD_REQUEST;
                error!("Incompatible query params. Params required by server: {:?}. Params sent in request: {:?}", self.conn_query_params, request.uri().query());
                debug!("Connection response {:?}", response);
            }

            Ok(response)
        };
        let ws_stream = accept_hdr_async(stream, callback)
            .await
            .expect("Failed to accept");

        let (send, mut recv) = ws_stream.split();

        debug!("New WebSocket connection: {peer}");

        self.add_connected_peer(&peer, send).await;

        while let Some(msg) = recv.next().await {
            debug!("incoming message");
            let msg = msg?;
            debug!("Message: {:?}", msg);

            if msg.is_close() {
                break;
            }

            if msg.is_text() || msg.is_binary() {
                let msg = msg.to_string();
                let request_message = match serde_json::from_str::<Value>(msg.as_str()).ok() {
                    Some(key) => key,
                    None => {
                        warn!("Request is not valid JSON. Request: {msg}");
                        continue;
                    }
                };

                debug!("Parsed message: {:?}", request_message);

                let responses = match self.find_responses(request_message).await {
                    Some(value) => value,
                    None => continue,
                };
                let connected_peer = self.connected_peer_sinks.clone();
                tokio::spawn(async move {
                    if let Err(e) =
                        Self::send_to_sink(connected_peer, &peer.to_string(), responses).await
                    {
                        error!("Error sending data back to sink {}", e.to_string());
                    }
                });
            }
        }

        debug!("Connection dropped peer={peer}");
        self.remove_connected_peer(&peer).await;

        Ok(())
    }

    async fn send_to_sink(
        connection: WSConnection,
        peer: &str,
        responses: Vec<ResponseSink>,
    ) -> Result<()> {
        let mut clients = connection.lock().await;
        let sink = clients.get_mut(peer);
        if let Some(sink) = sink {
            for resp in responses {
                let response = resp.data.to_string();
                if resp.delay > 0 {
                    tokio::time::sleep(Duration::from_secs(resp.delay)).await
                }
                if let Err(e) = sink.send(Message::Text(response.clone())).await {
                    error!("Error sending response. resp={e:?}");
                } else {
                    debug!("sent response. resp={response:?}");
                }
            }
        } else {
            error!("No sink found for peer={peer:?}");
        }
        Ok(())
    }

    async fn find_responses(&self, request_message: Value) -> Option<Vec<ResponseSink>> {
        debug!(
            "is value json rpc {} {}",
            request_message,
            is_value_jsonrpc(&request_message)
        );
        if let Ok(v) = serde_json::from_value::<JsonRpcApiRequest>(request_message.clone()) {
            if let Some(id) = v.id {
                if self.config.activate_all_plugins && v.method.contains("Controller.1.status") {
                    return Some(vec![ResponseSink {
                        delay: 0,
                        data: json!({"jsonrpc": "2.0", "id": id, "result": [{"state": "activated"}]}),
                    }]);
                } else if let Some(v) = self.responses_for_key_v2(&v).await {
                    return Some(v.get_all(Some(id)));
                }
                return Some(vec![ResponseSink {
                    delay: 0,
                    data: json!({"jsonrpc": "2.0", "id": id, "error": {"code": -32001, "message":"not found"}}),
                }]);
            }
        }

        None
    }

    async fn responses_for_key_v2(&self, req: &JsonRpcApiRequest) -> Option<ParamResponse> {
        let mock_data = self.mock_data_v2.read().unwrap();
        if let Some(mut v) = mock_data.get(&req.method).cloned() {
            if v.len() == 1 {
                return Some(v.remove(0));
            } else if let Some(params) = &req.params {
                for response in v {
                    if response.get_key(params).is_some() {
                        return Some(response);
                    }
                }
            }
        }
        None
    }

    async fn add_connected_peer(
        &self,
        peer: &SocketAddr,
        sink: SplitSink<WebSocketStream<TcpStream>, Message>,
    ) {
        let mut peers = self.connected_peer_sinks.lock().await;
        peers.insert(peer.to_string(), sink);
    }

    async fn remove_connected_peer(&self, peer: &SocketAddr) {
        let mut peers = self.connected_peer_sinks.lock().await;
        let _ = peers.remove(&peer.to_string());
    }

    pub async fn add_request_response_v2(&self, request: MockData) -> Result<(), MockDataError> {
        let mut mock_data = self.mock_data_v2.write().unwrap();
        mock_data.extend(request);
        Ok(())
    }

    pub async fn remove_request_response_v2(&self, request: MockData) -> Result<(), MockDataError> {
        let mut mock_data = self.mock_data_v2.write().unwrap();
        for (cleanup_key, cleanup_params) in request {
            if let Some(v) = mock_data.remove(&cleanup_key) {
                let mut new_param_response = Vec::new();
                let mut updated = false;
                for cleanup_param in cleanup_params {
                    if let Some(params) = cleanup_param.params {
                        for current_params in &v {
                            if current_params.get_key(&params).is_none() {
                                new_param_response.push(current_params.clone());
                            } else if !updated {
                                updated = true;
                            }
                        }
                    }
                }
                if updated && !new_param_response.is_empty() {
                    let _ = mock_data.insert(cleanup_key, new_param_response);
                } else {
                    let _ = mock_data.insert(cleanup_key, v);
                }
            }
        }
        Ok(())
    }

    pub async fn emit_event(self: Arc<Self>, event: &Value, delay: u32) {
        unimplemented!("Emit event functionality has not yet been implemented {event} {delay}");
        // TODO: handle Results
        // debug!("waiting to send event");

        // let payload = event.clone();

        // tokio::spawn(async move {
        //     tokio::time::sleep(tokio::time::Duration::from_millis(delay.into())).await;

        //     let mut peers = self.connected_peer_sinks.lock().await;
        //     for peer in peers.values_mut() {
        //         debug!("send event to web socket");
        //         let _ = peer.send(Message::Text(payload.to_string())).await;
        //     }
        // });
    }
}

// #[cfg(test)]
// mod tests {
//     use ripple_sdk::tokio::time::{self, error::Elapsed, Duration};

//     use crate::mock_data::MockDataMessage;

//     use super::*;

//     async fn start_server(mock_data: MockData) -> Arc<MockWebSocketServer> {
//         let mock_data = Arc::new(RwLock::new(mock_data));
//         let server = MockWebSocketServer::new(
//             HashMap::new(),
//             WsServerParameters::default(),
//             MockConfig::default(),
//         )
//         .await
//         .expect("Unable to start server")
//         .into_arc();

//         tokio::spawn(server.clone().start_server());

//         server
//     }

//     async fn request_response_with_timeout(
//         server: Arc<MockWebSocketServer>,
//         request: Message,
//     ) -> Result<Option<Result<Message, Error>>, Elapsed> {
//         let (client, _) =
//             tokio_tungstenite::connect_async(format!("ws://0.0.0.0:{}", server.port()))
//                 .await
//                 .expect("Unable to connect to WS server");

//         let (mut send, mut receive) = client.split();

//         send.send(request).await.expect("Failed to send message");

//         time::timeout(Duration::from_secs(1), receive.next()).await
//     }

//     fn mock_data_json() -> (MockData, Value, Value) {
//         let request_body = json!({"key":"value"});
//         let request = json!({"type": "json", "body": request_body});
//         let response_body = json!({"success": true, "data": "data"});
//         let response = json!({"type": "json", "body": response_body});
//         let mock_data = HashMap::from([(
//             json_key(&request_body).unwrap(),
//             (
//                 (&request).try_into().unwrap(),
//                 vec![(&response).try_into().unwrap()],
//             ),
//         )]);

//         (mock_data, request_body, response_body)
//     }

//     fn mock_data_jsonrpc() -> (MockData, Value, Value) {
//         let request_body = json!({"jsonrpc":"2.0", "id": 0, "method": "someAction", "params": {}});
//         let request = json!({"type": "jsonrpc", "body": request_body});
//         let response_body = json!({"jsonrpc": "2.0", "id": 0, "result": {"success": true}});
//         let response = json!({"type": "jsonrpc", "body": response_body});
//         let mock_data = HashMap::from([(
//             jsonrpc_key(&request_body).unwrap(),
//             (
//                 (&request).try_into().unwrap(),
//                 vec![(&response).try_into().unwrap()],
//             ),
//         )]);

//         (mock_data, request_body, response_body)
//     }

//     #[ignore]
//     #[test]
//     fn test_ws_server_parameters_new() {
//         let params = WsServerParameters::new();
//         let params_default = WsServerParameters::default();

//         assert!(params.headers.is_none());
//         assert!(params.path.is_none());
//         assert!(params.port.is_none());
//         assert!(params.query_params.is_none());
//         assert_eq!(params, params_default);
//     }

//     #[ignore]
//     #[test]
//     fn test_ws_server_parameters_props() {
//         let mut params = WsServerParameters::new();
//         let headers: HeaderMap = {
//             let hm = HashMap::from([("Sec-WebSocket-Protocol".to_owned(), "jsonrpc".to_owned())]);
//             (&hm).try_into().expect("valid headers")
//         };
//         let qp = HashMap::from([("appId".to_owned(), "test".to_owned())]);
//         params
//             .headers(headers.clone())
//             .port(16789)
//             .path("/some/path")
//             .query_params(qp.clone());

//         assert_eq!(params.headers, Some(headers));
//         assert_eq!(params.port, Some(16789));
//         assert_eq!(params.path, Some("/some/path".to_owned()));
//         assert_eq!(params.query_params, Some(qp));
//     }

//     #[ignore]
//     #[tokio::test(flavor = "multi_thread")]
//     async fn test_start_server() {
//         let mock_data = HashMap::default();
//         let server = start_server(mock_data).await;

//         let _ = tokio_tungstenite::connect_async(format!("ws://0.0.0.0:{}", server.port()))
//             .await
//             .expect("Unable to connect to WS server");
//     }

//     #[ignore]
//     #[tokio::test(flavor = "multi_thread")]
//     async fn test_startup_mock_data_json_matched_request() {
//         let (mock_data, request_body, response_body) = mock_data_json();
//         let server = start_server(mock_data).await;

//         let response =
//             request_response_with_timeout(server, Message::Text(request_body.to_string()))
//                 .await
//                 .expect("no response from server within timeout")
//                 .expect("connection to server was closed")
//                 .expect("error in server response");

//         assert_eq!(response, Message::Text(response_body.to_string()));
//     }

//     #[ignore]
//     #[tokio::test(flavor = "multi_thread")]
//     async fn test_startup_mock_data_json_mismatch_request() {
//         let (mock_data, _, _) = mock_data_json();
//         let server = start_server(mock_data).await;

//         let response = request_response_with_timeout(
//             server,
//             Message::Text(json!({"key":"value2"}).to_string()),
//         )
//         .await;

//         assert!(response.is_err());
//     }

//     #[ignore]
//     #[tokio::test(flavor = "multi_thread")]
//     async fn test_startup_mock_data_jsonrpc_matched_request() {
//         let (mock_data, mut request_body, mut response_body) = mock_data_jsonrpc();
//         let server = start_server(mock_data).await;

//         request_body
//             .as_object_mut()
//             .and_then(|req| req.insert("id".to_owned(), 327.into()));
//         response_body
//             .as_object_mut()
//             .and_then(|req| req.insert("id".to_owned(), 327.into()));

//         let response =
//             request_response_with_timeout(server, Message::Text(request_body.to_string()))
//                 .await
//                 .expect("no response from server within timeout")
//                 .expect("connection to server was closed")
//                 .expect("error in server response");

//         assert_eq!(response, Message::Text(response_body.to_string()));
//     }

//     #[ignore]
//     #[tokio::test(flavor = "multi_thread")]
//     async fn test_startup_mock_data_jsonrpc_mismatch_request() {
//         let (mock_data, _, _) = mock_data_json();
//         let server = start_server(mock_data).await;

//         let response = request_response_with_timeout(
//             server,
//             Message::Text(
//                 json!({"jsonrpc": "2.0", "id": 11, "method": "someUnknownAction"}).to_string(),
//             ),
//         )
//         .await
//         .expect("no response from server within timeout")
//         .expect("connection to server was closed")
//         .expect("error in server response");

//         assert_eq!(
//             response,
//             Message::Text(
//                 json!({"jsonrpc": "2.0", "id": 11, "error": {"message": "Invalid Request", "code": -32600}})
//                     .to_string()
//             )
//         );
//     }

//     #[ignore]
//     #[tokio::test(flavor = "multi_thread")]
//     async fn test_startup_mock_data_add_request() {
//         let mock_data = HashMap::default();
//         let request_body = json!({"key": "value"});
//         let response_body = json!({"success": true});
//         let server = start_server(mock_data).await;

//         // server
//         //     .add_request_response(
//         //         (&json!({"type": "json", "body": request_body.clone()}))
//         //             .try_into()
//         //             .unwrap(),
//         //         vec![(&json!({"type": "json", "body": response_body.clone()}))
//         //             .try_into()
//         //             .unwrap()],
//         //     )
//         //     .await
//         //     .expect("unable to add mock responses");

//         let response =
//             request_response_with_timeout(server, Message::Text(request_body.to_string()))
//                 .await
//                 .expect("no response from server within timeout")
//                 .expect("connection to server was closed")
//                 .expect("error in server response");

//         assert_eq!(response, Message::Text(response_body.to_string()));
//     }

//     #[ignore]
//     #[tokio::test(flavor = "multi_thread")]
//     async fn test_startup_mock_data_remove_request() {
//         let mock_data = HashMap::default();
//         let request_body = json!({"key": "value"});
//         let response_body = json!({"success": true});
//         let server = start_server(mock_data).await;
//         let request: MockDataMessage = (&json!({"type": "json", "body": request_body.clone()}))
//             .try_into()
//             .unwrap();

//         // server
//         //     .add_request_response(
//         //         request.clone(),
//         //         vec![(&json!({"type": "json", "body": response_body.clone()}))
//         //             .try_into()
//         //             .unwrap()],
//         //     )
//         //     .await
//         //     .expect("unable to add mock responses");

//         // server
//         //     .remove_request(&request)
//         //     .await
//         //     .expect("unable to remove request");

//         let response =
//             request_response_with_timeout(server, Message::Text(request_body.to_string())).await;

//         assert!(response.is_err());
//     }
// }
