use cosmwasm_std::{
    coins, entry_point, to_binary, Addr, Api, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo,
    Response, StdError, StdResult, Uint128,
};

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, PayeesResponse, QueryMsg, ReleasedResponse, SharesResponse,
};
use crate::state::{PAYEES, RELEASED, SHARES, TOTAL_RELEASED, TOTAL_SHARES};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let length = msg.payees.len();

    if msg.shares.len() != length {
        return Err(ContractError::InvalidLength {});
    }

    let payees = map_validate(deps.api, &msg.payees)?;

    for index in 0..length {
        if let Some(payee) = payees.get(index) {
            if msg.shares[index] < 1 {
                return Err(ContractError::InvalidShares {});
            }

            SHARES.save(deps.storage, payee, &msg.shares[index])?;
            RELEASED.save(deps.storage, payee, &Uint128::zero())?;
        }
    }

    let total_shares = msg.shares.iter().sum();

    PAYEES.save(deps.storage, &payees)?;
    TOTAL_SHARES.save(deps.storage, &total_shares)?;
    TOTAL_RELEASED.save(deps.storage, &Uint128::zero())?;

    Ok(Response::default())
}

pub fn map_validate(api: &dyn Api, addrs: &[String]) -> StdResult<Vec<Addr>> {
    addrs.iter().map(|addr| api.addr_validate(&addr)).collect()
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Release { address } => execute_release(deps, env, info, address), // v1
    }
}

fn execute_release(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    let account = deps.api.addr_validate(&address)?;

    if !can_release(deps.as_ref(), account.as_str())? {
        Err(ContractError::Unauthorized {})
    } else {
        let shares = SHARES.load(deps.storage, &account)?;
        let released = RELEASED.load(deps.storage, &account)?;

        let total_shares = TOTAL_SHARES.load(deps.storage)?;
        let total_released = TOTAL_RELEASED.load(deps.storage)?;

        let balance = deps.querier.query_all_balances(&env.contract.address)?;
        if let Some(native_balance) = balance.first() {
            let total_received = native_balance
                .amount
                .checked_add(total_released)
                .map_err(StdError::overflow)?;
            let denom = native_balance.denom.clone();

            let amount = total_received
                .checked_mul(Uint128::from(shares))
                .map_err(StdError::overflow)?
                .checked_div(Uint128::from(total_shares))
                .map_err(StdError::divide_by_zero)?
                .checked_sub(released)
                .map_err(StdError::overflow)?;

            if amount.is_zero() {
                return Err(ContractError::NoPaymentDue {});
            }

            RELEASED.update(deps.storage, &account, |_| -> StdResult<_> {
                released.checked_add(amount).map_err(StdError::overflow)
            })?;
            TOTAL_RELEASED.update(deps.storage, |_| -> StdResult<_> {
                total_released
                    .checked_add(amount)
                    .map_err(StdError::overflow)
            })?;

            let message = BankMsg::Send {
                to_address: account.to_string(),
                amount: coins(amount.u128(), denom),
            };

            let mut res = Response::new();
            res.add_attribute("action", "approve");
            res.add_message(message);
            Ok(res)
        } else {
            Err(ContractError::InvalidBalance {})
        }
    }
}

fn can_release(deps: Deps, addr: &str) -> StdResult<bool> {
    let payees = PAYEES.load(deps.storage)?;
    let can = payees.iter().any(|s| s.as_ref() == addr);
    Ok(can)
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetTotalShares {} => to_binary(&query_total_shares(deps)?),
        QueryMsg::GetTotalReleased {} => to_binary(&query_total_released(deps)?),
        QueryMsg::GetShares { address } => to_binary(&query_shares(deps, address)?),
        QueryMsg::GetReleased { address } => to_binary(&query_released(deps, address)?),
        QueryMsg::GetPayees {} => to_binary(&query_payees(deps)?),
    }
}

fn query_payees(deps: Deps) -> StdResult<PayeesResponse> {
    let payees = PAYEES.load(deps.storage)?;
    Ok(PayeesResponse {
        payees: payees.into_iter().map(|a| a.into()).collect(),
    })
}

fn query_total_shares(deps: Deps) -> StdResult<SharesResponse> {
    let shares = TOTAL_SHARES.load(deps.storage)?;
    Ok(SharesResponse { shares })
}

fn query_total_released(deps: Deps) -> StdResult<ReleasedResponse> {
    let released = TOTAL_RELEASED.load(deps.storage)?;
    Ok(ReleasedResponse { released })
}

fn query_shares(deps: Deps, address: String) -> StdResult<SharesResponse> {
    let addr = deps.api.addr_validate(&address)?;
    let shares = SHARES.load(deps.storage, &addr)?;
    Ok(SharesResponse { shares })
}

fn query_released(deps: Deps, address: String) -> StdResult<ReleasedResponse> {
    let addr = deps.api.addr_validate(&address)?;
    let released = RELEASED.load(deps.storage, &addr)?;
    Ok(ReleasedResponse { released })
}

#[cfg(test)]
mod tests {
    use crate::msg::PayeesResponse;

    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MOCK_CONTRACT_ADDR};
    use cosmwasm_std::{coins, from_binary, CosmosMsg};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);
        let one = String::from("one");

        let msg = InstantiateMsg {
            payees: vec![String::from(&one)],
            shares: vec![1],
        };

        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetPayees {}).unwrap();
        let value: PayeesResponse = from_binary(&res).unwrap();
        assert_eq!(1, value.payees.len());
        let query_one = value.payees.get(0).unwrap();
        assert_eq!(query_one.as_str(), one);
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetShares { address: one },
        )
        .unwrap();
        let value: SharesResponse = from_binary(&res).unwrap();
        assert_eq!(1, value.shares);
    }

    #[test]
    fn invalid_shares() {
        let mut deps = mock_dependencies(&[]);
        let one = String::from("one");

        let msg = InstantiateMsg {
            payees: vec![String::from(&one)],
            shares: vec![0],
        };

        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert!(matches!(res, ContractError::InvalidShares {}));
    }

    #[test]
    fn send_one() {
        let mut deps = mock_dependencies(&coins(1000, "token"));

        let one = String::from("asdf");
        let shares = 1;

        let msg = InstantiateMsg {
            payees: vec![one.clone()],
            shares: vec![shares],
        };
        let info = mock_info("creator", &coins(1000, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info(&one, &coins(1000, "token"));
        let msg = ExecuteMsg::Release {
            address: one.clone(),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(1, res.messages.len());
        let msg: &CosmosMsg = res.messages.get(0).expect("no message");
        assert_eq!(
            msg,
            &CosmosMsg::Bank(BankMsg::Send {
                to_address: one,
                amount: coins(1000, "token"),
            })
        );
    }

    #[test]
    fn block_nonsplit_address() {
        let mut deps = mock_dependencies(&coins(1000, "token"));

        let one = String::from("asdf");
        let blocked = String::from("blocked");

        let msg = InstantiateMsg {
            payees: vec![one.clone()],
            shares: vec![1],
        };
        let info = mock_info("creator", &coins(1000, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release itlet env = mock_env();
        let env = mock_env();
        let info = mock_info(&one, &coins(1000, "token"));
        let msg = ExecuteMsg::Release { address: blocked };
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized {}));
    }

    #[test]
    fn send_two() {
        let mut deps = mock_dependencies(&coins(1000, "token"));

        let one = String::from("asdf");
        let one_shares = 1;

        let two = String::from("jkl");
        let two_shares = 3;

        let msg = InstantiateMsg {
            payees: vec![one.clone(), two.clone()],
            shares: vec![one_shares, two_shares],
        };
        let info = mock_info("creator", &coins(1000, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert_eq!(
            query_total_shares(deps.as_ref()).unwrap(),
            SharesResponse { shares: 4 }
        );
        assert_eq!(
            query_total_released(deps.as_ref()).unwrap(),
            ReleasedResponse {
                released: Uint128::zero()
            }
        );

        // beneficiary can release it
        let info = mock_info(&two, &[]);
        let msg = ExecuteMsg::Release {
            address: two.clone(),
        };
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        assert_eq!(1, res.messages.len());
        let msg: &CosmosMsg = res.messages.get(0).expect("no message");
        assert_eq!(
            msg,
            &CosmosMsg::Bank(BankMsg::Send {
                to_address: two.clone(),
                amount: coins(750, "token"),
            })
        );
        deps.querier
            .update_balance(MOCK_CONTRACT_ADDR, coins(250, "token"));

        assert_eq!(
            query_released(deps.as_ref(), two.clone()).unwrap(),
            ReleasedResponse {
                released: Uint128::new(750)
            }
        );
        assert_eq!(
            query_shares(deps.as_ref(), two.clone()).unwrap(),
            SharesResponse { shares: 3 }
        );

        assert_eq!(
            query_total_shares(deps.as_ref()).unwrap(),
            SharesResponse { shares: 4 }
        );
        assert_eq!(
            query_total_released(deps.as_ref()).unwrap(),
            ReleasedResponse {
                released: Uint128::new(750)
            }
        );

        // check no more can be released
        let msg = ExecuteMsg::Release {
            address: two.clone(),
        };
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert!(matches!(err, ContractError::NoPaymentDue {}));

        // other beneficiary can release theirs
        let info = mock_info(&one, &coins(1000, "token"));
        let msg = ExecuteMsg::Release {
            address: one.clone(),
        };
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        assert_eq!(1, res.messages.len());
        let msg: &CosmosMsg = res.messages.get(0).expect("no message");
        assert_eq!(
            msg,
            &CosmosMsg::Bank(BankMsg::Send {
                to_address: one.clone(),
                amount: coins(250, "token"),
            })
        );
        deps.querier
            .update_balance(MOCK_CONTRACT_ADDR, coins(0, "token"));

        assert_eq!(
            query_released(deps.as_ref(), one.clone()).unwrap(),
            ReleasedResponse {
                released: Uint128::new(250)
            }
        );
        assert_eq!(
            query_shares(deps.as_ref(), one.clone()).unwrap(),
            SharesResponse { shares: 1 }
        );

        // check no more can be released
        let msg = ExecuteMsg::Release { address: one };
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert!(matches!(err, ContractError::NoPaymentDue {}));

        // todo - add more balance and test another payment
    }
}
