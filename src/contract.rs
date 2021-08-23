use cosmwasm_std::{
    attr, entry_point, to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut,
    Env, MessageInfo, Response, StdResult,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SplitsResponse};
use crate::state::{Split, State, STATE};

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let total_weight = msg.splits.iter().fold(0, |acc, s| acc + s.weight);

    let state = State {
        splits: msg.splits,
        total_weight,
    };
    STATE.save(deps.storage, &state)?;

    Ok(Response::default())
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
        ExecuteMsg::Payout {} => execute_payout(deps, env, info), // v1
    }
}

fn execute_payout(deps: DepsMut, env: Env, _info: MessageInfo) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;
    let splits = state.splits;
    let total_weight = state.total_weight;

    let balance = deps.querier.query_all_balances(&env.contract.address)?;
    let messages = send_tokens(&splits, balance, total_weight)?;

    let attributes = vec![
        attr("action", "approve"),
        // attr("id", "123"),
        // attr("to", split.addr.clone()),
    ];

    Ok(Response {
        submessages: vec![],
        messages,
        attributes,
        data: None,
    })
}

fn send_tokens(
    splits: &Vec<Split>,
    balance: Vec<Coin>,
    total_weight: u64,
) -> StdResult<Vec<CosmosMsg>> {
    // TODO: support more than a single coin?
    let coin = balance.get(0).unwrap();

    let msgs = splits
        .iter()
        .map(|s| {
            let percentage: Decimal = Decimal::from_ratio(s.weight, total_weight);
            let count = coin.amount * percentage;
            let amount = vec![Coin::new(count.u128(), &coin.denom)];
            let send = BankMsg::Send {
                to_address: s.addr.to_string(),
                amount,
            };
            send.into()
        })
        .collect();

    Ok(msgs)
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetSplits {} => to_binary(&query_splits(deps)?),
    }
}

fn query_splits(deps: Deps) -> StdResult<SplitsResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(SplitsResponse {
        splits: state.splits,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);
        let one = Split {
            addr: Addr::unchecked("asdf"),
            weight: 1,
        };

        let msg = InstantiateMsg {
            splits: vec![one.clone()],
        };

        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetSplits {}).unwrap();
        let value: SplitsResponse = from_binary(&res).unwrap();
        assert_eq!(1, value.splits.len());
        let query_one = value.splits.get(0).unwrap();
        assert_eq!(one.weight, query_one.weight);
        assert_eq!(one.addr.to_string(), query_one.addr.to_string());
    }

    #[test]
    fn send_one() {
        let mut deps = mock_dependencies(&coins(1000, "token"));

        let one = Split {
            addr: Addr::unchecked("asdf"),
            weight: 1,
        };

        let msg = InstantiateMsg { splits: vec![one] };
        let info = mock_info("creator", &coins(1000, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(1000, "token"));
        let msg = ExecuteMsg::Payout {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

        // complete release by verfier, before expiration
        let env = mock_env();
        let info = mock_info("verifies", &[]);
        let execute_res = execute(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(1, execute_res.messages.len());
        let msg: &CosmosMsg = execute_res.messages.get(0).expect("no message");
        assert_eq!(
            msg,
            &CosmosMsg::Bank(BankMsg::Send {
                to_address: String::from("asdf"),
                amount: coins(1000, "token"),
            })
        );
    }

    #[test]
    fn send_two() {
        let mut deps = mock_dependencies(&coins(1000, "token"));

        let one = Split {
            addr: Addr::unchecked("asdf"),
            weight: 1,
        };
        let two = Split {
            addr: Addr::unchecked("jkl"),
            weight: 1,
        };

        let msg = InstantiateMsg {
            splits: vec![one, two],
        };
        let info = mock_info("creator", &coins(1000, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(1000, "token"));
        let msg = ExecuteMsg::Payout {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

        // complete release by verfier, before expiration
        let env = mock_env();
        let info = mock_info("verifies", &[]);
        let execute_res = execute(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(2, execute_res.messages.len());
        let msg: &CosmosMsg = execute_res.messages.get(0).expect("no message");
        assert_eq!(
            msg,
            &CosmosMsg::Bank(BankMsg::Send {
                to_address: String::from("asdf"),
                amount: coins(500, "token"),
            })
        );

        let msg2: &CosmosMsg = execute_res.messages.get(1).expect("no message");
        assert_eq!(
            msg2,
            &CosmosMsg::Bank(BankMsg::Send {
                to_address: String::from("jkl"),
                amount: coins(500, "token"),
            })
        );
    }
}
