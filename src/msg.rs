use cosmwasm_std::Uint128;
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
    Release { address: String },
    // v0.1.0
    // - [x] use Map to check if address in split
    // - [x] message to release funds for account

    // for v0.1.1
    // - [ ] remaining query messages below

    // for v0.1.2
    // - [ ] account can remove themselves

    // for v0.1.3
    // - [ ] support cw20
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetTotalShares{},
    // GetTotalReleased{},
    GetShares { address: String },
    GetReleased { address: String },
    GetPayees {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PayeesResponse {
    pub payees: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ReleasedResponse {
    pub released: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SharesResponse {
    pub shares: u64,
}
