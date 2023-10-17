use serde::{Deserialize, Serialize};

use crate::framework::ripple_contract::{ContractAdjective, RippleContract};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PlayerAdjective {
    Base,
    Broadcast,
    Streaming,
}

impl ContractAdjective for PlayerAdjective {
    fn get_contract(&self) -> RippleContract {
        RippleContract::Player(self.clone())
    }
}
