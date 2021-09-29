use crate::state::State;
// use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub payees: Vec<String>,
    pub shares: Vec<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Release {},
    // v1
    // - anyone inside split can execute payout
    // - use map to check if address in split

    // v2 UpdateSplit
    // v2 Recieve for cw20
    // v2 "Automatically" Payout
    // - maybe change Payout to Claim:
    //   allowing users in the split to claim their portion
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    // GetTotalShares{},
    // GetTotalReleased{},
    // GetShares{},
    // GetReleased{},
    GetPayees {},
}

// // We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PayeesResponse {
    pub payees: Vec<String>,
}
