use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult,
};
use std::{vec::Vec};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
pub struct QueryMsg {
    msg: Binary,
    fis_input: Vec<FisInput>
}

#[cw_serde]
pub struct FisInput {
    data: Vec<Binary>
}

#[cw_serde]
pub struct Fund {
    receivers: Vec<String>,
}

#[cw_serde]
pub struct FISInstruction {
    plane: String,
    action: String,
    address: String,
    msg: Vec<u8>,
}

#[cw_serde]
pub struct StrategyOutput {
    instructions: Vec<FISInstruction>,
}

#[cw_serde]
pub struct MsgSend {
    from_address: String,
    to_address: String,
    amount: Vec<BankAmount>,
}

#[cw_serde]
pub struct BankAmount {
    denom: String,
    amount: String,
}

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[entry_point]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> StdResult<Response> {
    Ok(Response::new().add_attribute("method", "execute"))
}

#[entry_point]
pub fn query(_deps: Deps, env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    // parse command, we can store it as proto bytes, encrypted binary
    let mut instructions = vec![];

    // send 1 usdt
    instructions.push(FISInstruction {
        plane: "COSMOS".to_string(),
        action: "COSMOS_BANK_SEND".to_string(),
        address: "".to_string(),
        msg: to_json_binary(&MsgSend {
            from_address: env.contract.address.clone().into_string(),
            to_address: "lux158ucxjzr6ccrlpmz8z05wylu8tr5eueqcp2afu".to_string(), // wasm account
            amount: vec![BankAmount {
                denom: "usdt".to_string(),
                amount: "1".to_string(),
            }],
        })
        .unwrap()
        .to_vec(),
    });

    Ok(to_json_binary(&StrategyOutput { instructions }).unwrap())
}
