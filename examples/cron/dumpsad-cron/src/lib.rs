use astromesh::{FISInstruction, OracleEntries, PoolManager, QueryDenomLinkResponse};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, Binary, Coin, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Uint128,
};
use events::{GraduateEvent, StrategyEvent};
use std::vec::Vec;
use svm::raydium::Raydium;
use wasm::astroport::Astroport;
mod astromesh;
mod events;
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
    pub solver_id: String,
}

#[entry_point]
pub fn query(_deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let cron_msg = from_json::<CronMsg>(msg.msg)?;

    let event_inputs = &msg.fis_input.get(0).unwrap().data;
    let mut instructions = vec![];
    for e in event_inputs {
        let parsed_event = from_json::<StrategyEvent>(e)?;
        if parsed_event.strategy_id != cron_msg.solver_id {
            continue;
        }

        if parsed_event.topic != "graduate" {
            continue;
        }

        let graduate_event = from_json::<GraduateEvent>(parsed_event.data)?;
        let sol_coin = Coin {
            denom: "sol".to_string(),
            amount: graduate_event.sol_amount,
        };

        let meme_coin = Coin {
            denom: graduate_event.meme_denom,
            amount: graduate_event.meme_amount,
        };

        // TODO: Get denom link here for EVM, SVM
        // handle graduate
        let contract_sequence = msg.fis_input.get(1).unwrap().data.get(0).unwrap();
        let (mut denom_0, mut denom_1) = (sol_coin.denom, meme_coin.denom);
        let (mut amount_0, mut amount_1) = (sol_coin.amount, meme_coin.amount);
        let vm = graduate_event.vm;

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
        let pool: Box<dyn PoolManager> = match vm.to_uppercase().as_str() {
            "SVM" => Box::new(Raydium {}),
            "WASM" => Box::new(Astroport {
                contract_sequence: contract_sequence.clone(),
            }),
            _ => unreachable!(),
        };

        let create_pool_ixs = pool.create_pool_with_initial_liquidity(
            graduate_event.pool_address.clone(),
            denom_0.clone(),
            amount_0,
            denom_1.clone(),
            amount_1,
        );
        instructions.extend(create_pool_ixs);
    }
    Ok(to_json_binary(&StrategyOutput { instructions }).unwrap())
}
