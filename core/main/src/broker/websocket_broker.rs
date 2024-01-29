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

use std::time::Duration;
use futures_util::{SinkExt, StreamExt};
use ripple_sdk::{tokio::{self, sync::mpsc, net::TcpStream}, api::manifest::extn_manifest::PassthroughEndpoint, log::error};
use tokio_tungstenite::client_async;

use super::endpoint_broker::{BrokerSender, EndpointBroker, BrokerCallback};

pub struct WebsocketBroker{
    sender: BrokerSender,
}

impl EndpointBroker for WebsocketBroker {
    
    fn get_broker(endpoint:PassthroughEndpoint, callback:BrokerCallback) -> Self {
        let (tx,mut tr) = mpsc::channel(10);
        let broker = BrokerSender {
            sender: tx.clone()
        };
        tokio::spawn(async move {
            let tcp = loop {
                if let Ok(v) = TcpStream::connect(&endpoint.url).await {
                    break v;
                } else {
                    error!("Broker Wait for a sec and retry");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            };
            let url = url::Url::parse(&endpoint.url).unwrap();
            let (stream, _) = client_async(url, tcp)
            .await
            .unwrap();
            let (mut ws_tx, mut ws_rx) = stream.split();

            tokio::pin! {
                let read = ws_rx.next();
            }
            loop {
                tokio::select! {
                    Some(value) = &mut read => {
                        match value {
                            Ok(v) => {
                                match v {
                                    tokio_tungstenite::tungstenite::Message::Text(t) => {
                                        // send the incoming text without context back to the sender
                                        Self::handle_response(&t,callback.clone())
                                    }
                                    _ => {}
                                }
                            },
                            Err(e) => {
                                error!("Broker Websocket error on read {:?}", e);
                                break false
                            }
                        }
    
                    },
                    Some(request) = tr.recv() => {
                        if let Ok(request) = Self::update_request(&request) {
                             let _feed = ws_tx.feed(tokio_tungstenite::tungstenite::Message::Text(request)).await;
                            let _flush = ws_tx.flush().await;
                        }
                    
                    }
                }
            }
        });
        Self {
            sender: broker
        }
    }

    fn get_sender(&self) -> BrokerSender {
        self.sender.clone()
    }
}


