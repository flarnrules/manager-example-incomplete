#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult, SubMsg, WasmMsg, Reply, StdError, ReplyOn, Empty,
};

use cw2::set_contract_version;

use cw_utils::{parse_reply_instantiate_data};

use counter;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetContractsResponse, InstantiateMsg, QueryMsg};
use crate::state::{State, CONTRACTS};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:counter_manager";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const MAP_KEY:&str = "0";

const INSTANTIATE_REPLY_ID:u64 = 1;
const EXECUTE_INCREMENT_REPLY_ID:u64 = 2;
const EXECUTE_RESET_REPLY_ID:u64 = 3;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::InstantiateNewCounter { code_id } => instantiate_new_counter(deps, info, code_id),
        ExecuteMsg::Increment { contract } => try_increment(deps, contract),
        ExecuteMsg::Reset { contract, count } => try_reset(deps, info, contract, count),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    match msg.id {
        INSTANTIATE_REPLY_ID => handle_instantiate_reply(deps, msg),
        EXECUTE_INCREMENT_REPLY_ID => handle_increment_reply(deps, msg),
        EXECUTE_RESET_REPLY_ID => handle_reset_reply(deps, msg),
        id => Err(StdError::generic_err(format!("Unknown reply id: {}", id))),
    }
}



fn handle_instantiate_reply(deps: DepsMut, msg: Reply) -> StdResult<Response> {
    //println!("{:?}", msg.clone());
    let res = parse_reply_instantiate_data(msg).unwrap();
    let state = State {
        address: res.contract_address.clone(),
        count:0,
    };
    CONTRACTS.save(deps.storage, (&MAP_KEY, &res.contract_address), &state)?;
    Ok(Response::default())
}

fn handle_increment_reply(deps: DepsMut, msg: Reply) -> StdResult<Response> {
    //println!("{:?}", msg.clone());
    let contract_address = get_contract_address(&msg);
    
    CONTRACTS.update(deps.storage, (&MAP_KEY, &contract_address), |state| -> Result<_, ContractError> {
        let mut i_state = state.unwrap();
        i_state.count += 1;
        Ok(i_state)
    }).unwrap();

    Ok(Response::default())
}

fn handle_reset_reply(deps: DepsMut, msg: Reply) -> StdResult<Response> {
    //println!("{:?}", msg.clone());
    let contract_address = get_contract_address(&msg);
    let count = get_reset_count(&msg);

    CONTRACTS.update(deps.storage, (&MAP_KEY, &contract_address), |state| -> Result<_, ContractError> {
        let mut i_state = state.unwrap();
        i_state.count = count;
        Ok(i_state)
    }).unwrap();

    Ok(Response::default())
}

pub fn instantiate_new_counter(
    _deps: DepsMut,
    _info: MessageInfo,
    code_id: u64,
) -> Result<Response, ContractError> {
    let instantiate_message = WasmMsg::Instantiate {
        admin: None,
        code_id,
        msg: to_binary(&counter::msg::InstantiateMsg { count: 0 })?,
        funds: vec![],
        label: "counter".to_string(),
    };

    /*let submessage:SubMsg = SubMsg {
        gas_limit: None,
        id: INSTANTIATE_REPLY_ID,
        reply_on: ReplyOn::Success,
        msg: instantiate_message.into()
    };*/

    let submessage:SubMsg<Empty> = SubMsg::reply_on_success(instantiate_message, INSTANTIATE_REPLY_ID);

    Ok(Response::new().add_submessage(submessage))
}


pub fn try_increment(deps: DepsMut, contract: String) -> Result<Response, ContractError> {
    unimplemented!()
}

pub fn try_reset(
    deps: DepsMut,
    _info: MessageInfo,
    contract: String,
    count: i32,
) -> Result<Response, ContractError> {
    if CONTRACTS.has(deps.storage, (&MAP_KEY, &contract)) == false {
        return Err(ContractError::NotFound {});
    }

    let execute_message = WasmMsg::Execute {
        contract_addr: contract,
        funds: vec![],
        msg: to_binary(&counter::msg::ExecuteMsg::Reset { count })?,
    };

    /*let submessage:SubMsg = SubMsg {
        gas_limit: None,
        id: EXECUTE_RESET_REPLY_ID,
        reply_on: ReplyOn::Success,
        msg: execute_message.into()
    };*/

    let submessage:SubMsg<Empty> = SubMsg::reply_on_success(execute_message, EXECUTE_RESET_REPLY_ID);

    Ok(Response::new().add_submessage(submessage))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetContracts {} => to_binary(&query_get_contracts(deps)?),
    }
}

fn query_get_contracts(deps: Deps) -> StdResult<GetContractsResponse> {
    let res: StdResult<Vec<_>> = CONTRACTS
        .prefix(MAP_KEY)
        .range(deps.storage, None, None, Order::Ascending)
        .collect();
    let contracts = res?;
    Ok(GetContractsResponse { contracts })
}

////////////////////////////
//helper functions for parsing reply data
fn get_contract_address(msg: &Reply) -> String {
    let result:String = msg.result.clone().unwrap().events.iter().filter(|event| event.ty == "wasm" && event.attributes[0].key == "_contract_addr").map(|p| p.attributes[0].value.clone()).collect();
    //println!("{:?}", result);
    result
}

fn get_reset_count(msg: &Reply) -> i32 {
    let result :String = msg.result.clone().unwrap().events.iter().filter(|event| event.ty == "wasm" && event.attributes.len() == 3 && event.attributes[1].value == "reset").map(|p| p.attributes[2].value.clone()).collect();
    //println!("TEST {:?}", result);
    result.parse().unwrap()
}