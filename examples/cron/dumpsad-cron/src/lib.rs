use astromesh::{FISInstruction, OracleEntries, PoolManager, QueryDenomLinkResponse};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, Binary, Coin, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Uint128,
};
use std::vec::Vec;
use svm::raydium::Raydium;
use wasm::astroport::Astroport;
mod astromesh;
mod evm;
mod svm;
mod test;
mod wasm;

const GRADUATE_THRESHOLD_USD: &Uint128 = &Uint128::new(100_000u128);
const SOL_PRECISION_MULTIPLIER: &Uint128 = &Uint128::new(1_000_000_000u128);
const USDT_DECIMALS: u32 = 6;
const PYTH_PRICE_DECIMALS: u32 = 8;

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
    pub pool_address: String,
}

pub fn is_graduated(quote_amount: Uint128, sol_price: Uint128) -> bool {
    // 10^9 * (30+x)^2 / 32190005730 * sol_price >= 100000
    // graduate condition: market cap >= $100000
    // see bonding curve for the price formula
    // for simplicity, sol decimals = mem decimals => sol multiplier = meme multiplier
    let cap_in_sol = Uint128::new(1_000_000_000)
        * SOL_PRECISION_MULTIPLIER
        * (Uint128::new(30) * SOL_PRECISION_MULTIPLIER + quote_amount)
        / (Uint128::new(32190005730) * SOL_PRECISION_MULTIPLIER);
    let cap_in_usd = cap_in_sol * sol_price / SOL_PRECISION_MULTIPLIER / Uint128::new(100_000_000);

    return cap_in_usd.ge(GRADUATE_THRESHOLD_USD)
}

#[entry_point]
pub fn query(_deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let cron_msg = from_json::<CronMsg>(msg.msg)?;
    let vm = cron_msg.vm;
    let pool_address = cron_msg.pool_address;

    // get current liquidity source
    let quote_coin = from_json::<Coin>(msg.fis_input.get(0).unwrap().data.get(0).unwrap())?; // SOL
    let meme_coin = from_json::<Coin>(msg.fis_input.get(0).unwrap().data.get(1).unwrap())?;
    // TODO: Get denom link here for EVM, SVM
    // TODO: pool graduate condition (phuc)
    let quote_price_response =
        from_json::<OracleEntries>(msg.fis_input.get(2).unwrap().data.get(0).unwrap())?;
    let quote_price = quote_price_response.entries.get(0).unwrap().value;
    let graduated = is_graduated(quote_coin.amount, quote_price);
    if !graduated {
        return Ok(to_json_binary(&StrategyOutput { instructions: vec![] }).unwrap())
    }
    // handle graduate
    let mut instructions = vec![];
    let contract_sequence = msg.fis_input.get(1).unwrap().data.get(0).unwrap();
    let (mut denom_0, mut denom_1) = (quote_coin.denom, meme_coin.denom);
    let (mut amount_0, mut amount_1) = (quote_coin.amount, meme_coin.amount);

    if denom_0 > denom_1 {
        (denom_0, denom_1) = (denom_1, denom_0);
        (amount_0, amount_1) = (amount_1, amount_0);
    }

    // 1. pay creator 0.5 SOL, get 1.5 SOL as fee
    // TODO: Generate fee transfers here
    // 2. create pool in target vm
    _deps.api.debug(
        format!(
            "graduate: vm: {}, coin0:{}{}, coin1:{}{}",
            vm,
            denom_0.to_string(),
            amount_0.u128(),
            denom_1.to_string(),
            amount_1.u128()
        )
        .as_str(),
    );
    let pool: Box<dyn PoolManager> = match vm.as_str() {
        "SVM" => Box::new(Raydium {}),
        "WASM" => Box::new(Astroport {
            contract_sequence: contract_sequence.clone(),
        }),
        _ => unreachable!(),
    };

    let create_pool_ixs = pool.create_pool_with_initial_liquidity(
        pool_address.clone(),
        denom_0.clone(),
        amount_0,
        denom_1.clone(),
        amount_1,
    );
    instructions.extend(create_pool_ixs);
    // TODO: stop cron after graduate
    Ok(to_json_binary(&StrategyOutput { instructions }).unwrap())
}
