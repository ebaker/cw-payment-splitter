use crate::state::Split;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub splits: Vec<Split>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Payout {},
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
    GetSplits {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SplitsResponse {
    pub splits: Vec<Split>,
}
