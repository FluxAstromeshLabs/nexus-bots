use astromesh::{FISInstruction, PoolManager, QueryDenomLinkResponse};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, Binary, Coin, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Uint128,
};
use std::vec::Vec;
use svm::raydium::Raydium;
mod astromesh;
mod svm;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
pub struct QueryMsg {
    msg: Binary,
    fis_input: Vec<FisInput>,
}

#[cw_serde]
pub struct FisInput {
    data: Vec<Binary>,
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

#[cw_serde]
pub struct OracleResponse {
    pub entries: Vec<SimpleEntry>,
}

#[cw_serde]
pub struct SimpleEntry {
    pub symbol: String,
    pub decimal: i64,
    pub value: Uint128, // Assuming cosmossdk_io_math.Int maps to Uint128
    pub timestamp: u64,
}

#[cw_serde]
pub struct CronMsg {
    pub vm: String,
}

#[entry_point]
pub fn query(_deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let vm = &from_json::<CronMsg>(msg.msg)?.vm;
    let oracle_bz = msg.fis_input.get(0).unwrap().data.get(0).unwrap();
    // get oracle price
    let oracle_data = from_json::<OracleResponse>(oracle_bz)?;
    let price = oracle_data.entries.get(0).unwrap().value;
    // get current liquidity source
    let quote_amount = from_json::<Coin>(msg.fis_input.get(1).unwrap().data.get(0).unwrap())?;
    let meme_amount = from_json::<Coin>(msg.fis_input.get(1).unwrap().data.get(1).unwrap())?;

    let quote_link =
        from_json::<QueryDenomLinkResponse>(msg.fis_input.get(2).unwrap().data.get(0).unwrap())?;
    let meme_link =
        from_json::<QueryDenomLinkResponse>(msg.fis_input.get(2).unwrap().data.get(0).unwrap())?;
    // check if the pool is graduated
    let graduated = true; // TODO: Check if the pool is graduated

    let mut instructions = vec![];
    if graduated {
        let (denom_0, denom_1) = if quote_link.dst_addr < meme_link.dst_addr {
            (quote_link.dst_addr, meme_link.dst_addr)
        } else {
            (meme_link.dst_addr, quote_link.dst_addr)
        };
        // TODO: get actual amount as well
        let (amount_0, amount_1) = (Uint128::zero(), Uint128::zero());

        // 1. pay creator 0.5 SOL, get 1.5 SOL as fee
        // TODO: Code here
        // 2. create pool in target vm
        let pool = match vm.as_str() {
            "SVM" => Raydium {},
            _ => unreachable!(),
        };

        let create_pool_ixs = pool.create_pool(denom_0, denom_1);
        instructions.extend(create_pool_ixs);

        // 3. denom0, denom1 for liquidity
        let proivde_liquidity_ixs =
            pool.provide_liquidity_no_lp("".to_string(), amount_0, amount_1);
        instructions.extend(proivde_liquidity_ixs);
    }

    Ok(to_json_binary(&StrategyOutput { instructions }).unwrap())
}
