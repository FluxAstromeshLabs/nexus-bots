use astromesh::{
    to_int256, to_u128, to_uint256, uint16_to_le_bytes, FISInput, FISInstruction, MsgAstroTransfer,
    NexusAction,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, from_binary, from_json, to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env,
    Int128, MessageInfo, Response, StdError, StdResult, Uint256, Uint64, WasmMsg,
};
use std::collections::HashMap;
use std::vec::Vec;
mod astromesh;
mod drift;
mod svm;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
pub struct Link {
    pub cosmos_addr: String,
    pub svm_addr: String,
    pub height: Uint64,
}

#[cw_serde]
pub struct AccountLink {
    pub link: Link,
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

#[cw_serde]
pub struct QueryMsg {
    msg: Binary,
    fis_input: Vec<FISInput>,
}

pub struct StrategyOutput {
    instructions: Vec<FISInstruction>,
}

pub fn place_perp_market_order(
    deps: Deps,
    env: Env,
    market: String,
    usdt_amount: Int128,
    leverage: u8,
    auction_duration: u8,
    fis_input: &Vec<FISInput>,
) -> StdResult<Binary> {
    // 1. initialize account
    // 2. deposit usdt
    // 3. place order
    Ok(Binary::new(vec![]))
}

pub fn fill_perp_market_order(
    deps: Deps,
    env: Env,
    market: String,
    order_id: u32,
    // more params if necessary
    fis_input: &Vec<FISInput>,
) -> StdResult<Binary> {
    // 1. check if order is in auction time or not
    // 2. if it's in auction time, use JIT
    // 3. otherwise, fill with vAMM
    Ok(Binary::new(vec![]))
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let action = from_json::<NexusAction>(msg.msg)?;
    match action {
        NexusAction::PlacePerpMarketOrder {
            market,
            usdt_amount,
            leverage,
            auction_duration,
        } => place_perp_market_order(
            deps,
            env,
            market,
            usdt_amount,
            leverage,
            auction_duration,
            &msg.fis_input,
        ),
        NexusAction::FillPerpMarketOrder {} => unreachable!(),
    }
}
