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

use crate::bootstrap::boot::boot;
use crate::state::bootstrap_state::BootstrapState;
use ripple_sdk::{
    log::{error, info},
    tokio,
    utils::error::RippleError,
    utils::logger::init_and_configure_logger,
};
pub mod bootstrap;
pub mod firebolt;
pub mod processor;
pub mod service;
pub mod state;
pub mod utils;
use utils::runtime::*;
include!(concat!(env!("OUT_DIR"), "/version.rs"));

// pub async fn bootstrap() -> Result<BootstrapState, RippleError>  {
//     BootstrapState::build()
// }
// pub fn boostrap_logger(name: Option<&str>, version: Option<&str>) -> Result<(), fern::InitError> {

//   //  init_and_configure_logger(version.unwrap_or(SEMVER_LIGHTWEIGHT), name.unwrap_or("gateway").into())
//     Ok(())
// }

// pub async fn run(bootstrap_state:BootstrapState) -> () {

//     match boot(bootstrap_state).await {
//         Ok(_) => {
//             info!("Ripple Exited gracefully!");
//             std::process::exit(exitcode::OK);
//         }
//         Err(e) => {
//             error!("Ripple failed with Error: {:?}", e);
//             std::process::exit(exitcode::SOFTWARE);
//         }
//     }

//}
pub fn identity() -> () {
    info!("Ripple OSS version {}", SEMVER_LIGHTWEIGHT);
}

#[tokio::main(worker_threads = 2)]
async fn main() {
    boostrap_logger("gatewway", SEMVER_LIGHTWEIGHT).expect("could not setup logging, exiting");
    identity();
    run(bootstrap().await.expect("Ripple OSS bootstrap failed"));
}

// #[tokio::main(worker_threads = 2)]
// async fn main() {
//     // Init logger
//     if let Err(e) = init_and_configure_logger(SEMVER_LIGHTWEIGHT, "gateway".into()) {
//         println!("{:?} logger init error", e);
//         return;
//     }
//     info!("version {}", SEMVER_LIGHTWEIGHT);
//     let bootstate = BootstrapState::build().expect("Failure to init state for bootstrap");

//     // bootstrap
//     match boot(bootstate).await {
//         Ok(_) => {
//             info!("Ripple Exited gracefully!");
//             std::process::exit(exitcode::OK);
//         }
//         Err(e) => {
//             error!("Ripple failed with Error: {:?}", e);
//             std::process::exit(exitcode::SOFTWARE);
//         }
//     }
// }
