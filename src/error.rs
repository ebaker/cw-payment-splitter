use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
    #[error("Invalid Length")]
    InvalidLength {},
    #[error("Invalid Shares")]
    InvalidShares {},
    #[error("Invalid Payees")]
    InvalidPayees {},
    #[error("No payment due for account")]
    NoPaymentDue {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
