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

use ripple_sdk::{
    api::mock_server::{
        AddRequestResponseResponse, EmitEventResponse, MockServerRequest, MockServerResponse,
        RemoveRequestResponse,
    },
    async_trait::async_trait,
    extn::{
        client::{
            extn_client::ExtnClient,
            extn_processor::{
                DefaultExtnStreamer, ExtnRequestProcessor, ExtnStreamProcessor, ExtnStreamer,
            },
        },
        extn_client_message::{ExtnMessage, ExtnResponse},
    },
    log::{debug, error},
    tokio::sync::mpsc::{Receiver, Sender},
};

use crate::{mock_data::MockDataMessage, mock_web_socket_server::MockWebSocketServer};

#[derive(Debug, Clone)]
pub struct MockDeviceState {
    client: ExtnClient,
    server: Arc<MockWebSocketServer>,
}

impl MockDeviceState {
    fn new(client: ExtnClient, server: Arc<MockWebSocketServer>) -> Self {
        Self { client, server }
    }
}

pub struct MockDeviceProcessor {
    state: MockDeviceState,
    streamer: DefaultExtnStreamer,
}

impl MockDeviceProcessor {
    pub fn new(client: ExtnClient, server: Arc<MockWebSocketServer>) -> MockDeviceProcessor {
        MockDeviceProcessor {
            state: MockDeviceState::new(client, server),
            streamer: DefaultExtnStreamer::new(),
        }
    }

    async fn respond(client: ExtnClient, req: ExtnMessage, resp: MockServerResponse) -> bool {
        let resp = client
            .clone()
            .respond(req, ExtnResponse::MockServer(resp))
            .await;

        match resp {
            Ok(_) => true,
            Err(err) => {
                error!("{err:?}");
                false
            }
        }
    }
}

impl ExtnStreamProcessor for MockDeviceProcessor {
    type STATE = MockDeviceState;
    type VALUE = MockServerRequest;

    fn get_state(&self) -> Self::STATE {
        self.state.clone()
    }

    fn receiver(&mut self) -> Receiver<ExtnMessage> {
        self.streamer.receiver()
    }

    fn sender(&self) -> Sender<ExtnMessage> {
        self.streamer.sender()
    }
}

#[async_trait]
impl ExtnRequestProcessor for MockDeviceProcessor {
    fn get_client(&self) -> ExtnClient {
        self.state.client.clone()
    }

    async fn process_request(
        state: Self::STATE,
        extn_request: ExtnMessage,
        extracted_message: Self::VALUE,
    ) -> bool {
        debug!("extn_request={extn_request:?}, extracted_message={extracted_message:?}");
        match extracted_message {
            MockServerRequest::AddRequestResponse(params) => {
                let result = state
                    .server
                    .add_request_response(
                        MockDataMessage::from(params.request),
                        params
                            .responses
                            .into_iter()
                            .map(MockDataMessage::from)
                            .collect(),
                    )
                    .await;

                let resp = match result {
                    Ok(_) => AddRequestResponseResponse {
                        success: true,
                        error: None,
                    },
                    Err(err) => AddRequestResponseResponse {
                        success: false,
                        error: Some(err.to_string()),
                    },
                };

                Self::respond(
                    state.client.clone(),
                    extn_request,
                    MockServerResponse::AddRequestResponse(resp),
                )
                .await
            }
            MockServerRequest::RemoveRequest(params) => {
                let result = state
                    .server
                    .remove_request(&MockDataMessage::from(params.request))
                    .await;

                let resp = match result {
                    Ok(_) => RemoveRequestResponse {
                        success: true,
                        error: None,
                    },
                    Err(err) => RemoveRequestResponse {
                        success: false,
                        error: Some(err.to_string()),
                    },
                };

                Self::respond(
                    state.client.clone(),
                    extn_request,
                    MockServerResponse::RemoveRequestResponse(resp),
                )
                .await
            }
            MockServerRequest::EmitEvent(params) => {
                state
                    .server
                    .emit_event(&params.event.body, params.event.delay)
                    .await;

                Self::respond(
                    state.client.clone(),
                    extn_request,
                    MockServerResponse::EmitEvent(EmitEventResponse { success: true }),
                )
                .await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[should_panic]
    fn test_add_request_response() {
        todo!(
            "currently unable to test this without a testing solution so ExtnClient interactions"
        );
    }
}
