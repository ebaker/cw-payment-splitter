use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;

// we cast a ballot with our chosen vote and a given weight
// stored under the key that voted
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Split {
    pub weight: u64,
    pub addr: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub total_weight: u64,
    pub splits: Vec<Split>,
}

pub const STATE: Item<State> = Item::new("state");
