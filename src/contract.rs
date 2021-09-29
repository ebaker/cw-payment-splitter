use cosmwasm_std::{
    attr, coins, entry_point, to_binary, Addr, Api, BankMsg, Binary, Coin, CosmosMsg, Decimal,
    Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, PayeesResponse, QueryMsg};
use crate::state::{State, RELEASED, SHARES, STATE};

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
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
        let payee = payees.get(index).unwrap();

        if msg.shares[index] < 1 {
            return Err(ContractError::InvalidShares {});
        }

        SHARES.save(deps.storage, payee, &Uint128::new(msg.shares[index].into()));
        RELEASED.save(deps.storage, payee, &Uint128::zero());
    }

    let total_shares = msg.shares.iter().fold(0, |acc, s| acc + s);

    let state = State {
        total_shares,
        total_released: Uint128::new(0),
        payees,
    };
    STATE.save(deps.storage, &state)?;

    Ok(Response::default())
}

pub fn map_validate(api: &dyn Api, addrs: &[String]) -> StdResult<Vec<Addr>> {
    addrs.iter().map(|addr| api.addr_validate(&addr)).collect()
}

// And declare a custom Error variant for the ones where you will want to make use of it
#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Release {} => execute_release(deps, env, info), // v1
    }
}

fn execute_release(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;

    if !can_execute(deps.as_ref(), info.sender.as_ref())? {
        Err(ContractError::Unauthorized {})
    } else {
        let account = info.sender;
        let shares = SHARES.load(deps.storage, &account)?;
        let released = RELEASED.load(deps.storage, &account)?;

        let total_shares = state.total_shares;
        let total_released = state.total_released;

        let balance = deps.querier.query_all_balances(&env.contract.address)?;
        let native_balance = balance.get(0).unwrap();
        let total_received = native_balance.amount + total_released;
        let denom = native_balance.denom.clone();

        let amount = shares
            .checked_mul(total_received)
            .unwrap()
            .checked_div(released.checked_sub(Uint128::from(total_shares)).unwrap())
            .unwrap();

        let send = BankMsg::Send {
            to_address: account.to_string(),
            amount: coins(amount.u128(), denom),
        };

        let mut res = Response::new();
        res.add_attribute("action", "approve");
        res.add_message(send);
        Ok(
            res, // .add_messages(vec![send])
        )
    }
}

fn can_execute(deps: Deps, addr: &str) -> StdResult<bool> {
    let state = STATE.load(deps.storage)?;
    let payees = &state.payees;
    let can = payees.iter().any(|s| s.as_ref() == addr);
    Ok(can)
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPayees {} => to_binary(&query_payees(deps)?),
    }
}

fn query_payees(deps: Deps) -> StdResult<PayeesResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(PayeesResponse {
        payees: state.payees.into_iter().map(|a| a.into()).collect(),
    })
}

#[cfg(test)]
mod tests {
    use crate::msg::PayeesResponse;

    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

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
    }
}
