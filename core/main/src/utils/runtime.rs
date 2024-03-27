use crate::bootstrap::boot::boot;
use crate::state::bootstrap_state::BootstrapState;
use ripple_sdk::log::{error, info};
use ripple_sdk::utils::error::RippleError;
use ripple_sdk::utils::logger::init_and_configure_logger;
pub async fn bootstrap() -> Result<BootstrapState, RippleError> {
    BootstrapState::build()
}
pub fn boostrap_logger(name: &str, version: &str) -> Result<(), fern::InitError> {
    init_and_configure_logger(version, name.into())
}

pub async fn run(bootstrap_state: BootstrapState) -> () {
    match boot(bootstrap_state).await {
        Ok(_) => {
            info!("Ripple Exited gracefully!");
            std::process::exit(exitcode::OK);
        }
        Err(e) => {
            error!("Ripple failed with Error: {:?}", e);
            std::process::exit(exitcode::SOFTWARE);
        }
    }
}
