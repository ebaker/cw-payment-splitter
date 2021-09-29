use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub total_released: Uint128,
    pub total_shares: u64,
    pub payees: Vec<Addr>,
}

pub const STATE: Item<State> = Item::new("state");
pub const SHARES: Map<&Addr, Uint128> = Map::new("shares");
pub const RELEASED: Map<&Addr, Uint128> = Map::new("released");
