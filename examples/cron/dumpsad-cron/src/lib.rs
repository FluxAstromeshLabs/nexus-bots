use astromesh::{
    FISInstruction, MsgAstroTransfer, PoolManager, ACTION_COSMOS_INVOKE, ACTION_VM_INVOKE,
    PLANE_COSMOS, PLANE_SVM, PLANE_EVM,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, to_json_vec, Binary, Coin, Deps, DepsMut, Env,
    MessageInfo, Response, StdResult, Uint128,
};
use events::{GraduateEvent, StrategyEvent};
use std::vec::Vec;
use svm::raydium::Raydium;
use wasm::astroport::Astroport;
use evm::uniswap::Uniswap;
mod astromesh;
mod events;
mod evm;
mod svm;
mod test;
mod wasm;

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
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
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
        let vm = graduate_event.vm;
        let pool_address = graduate_event.pool_address;

        let vm_str = vm.to_uppercase();
        if vm_str.as_str() != "EVM" && vm_str.as_str() != "SVM" && vm_str.as_str() != "WASM" {
            deps
                .api
                .debug(format!("unsupported plane: {}", vm_str).as_str());
            continue;
        }

        let sol_coin = Coin {
            denom: "sol".to_string(),
            amount: graduate_event.sol_amount,
        };

        let meme_coin = Coin {
            denom: graduate_event.meme_denom.clone(),
            amount: graduate_event.meme_amount,
        };

        // handle graduate
        let contract_sequence = msg.fis_input.get(1).unwrap().data.get(0).unwrap();
        let (mut denom_0, mut denom_1) = (sol_coin.denom, meme_coin.denom);
        let (mut amount_0, mut amount_1) = (sol_coin.amount, meme_coin.amount);

        if vm_str.as_str() == "SVM" {
            instructions.extend(vec![
                FISInstruction {
                    plane: PLANE_COSMOS.to_string(),
                    action: ACTION_COSMOS_INVOKE.to_string(),
                    address: "".to_string(),
                    msg: to_json_vec(&MsgAstroTransfer::new(
                        pool_address.to_string(),
                        pool_address.to_string(),
                        PLANE_COSMOS.to_string(),
                        PLANE_SVM.to_string(),
                        Coin {
                            denom: denom_0.clone(),
                            amount: amount_0,
                        },
                    ))?,
                },
                FISInstruction {
                    plane: PLANE_COSMOS.to_string(),
                    action: ACTION_COSMOS_INVOKE.to_string(),
                    address: "".to_string(),
                    msg: to_json_vec(&MsgAstroTransfer::new(
                        pool_address.to_string(),
                        pool_address.to_string(),
                        PLANE_COSMOS.to_string(),
                        PLANE_SVM.to_string(),
                        Coin {
                            denom: denom_1.clone(),
                            amount: amount_1,
                        },
                    ))?,
                },
            ]);

            denom_0 = "CPozhCGVaGAcPVkxERsUYat4b7NKT9QeAR9KjNH4JpDG".to_string();
            denom_1 = graduate_event.meme_denom_link.clone();
        }

        if vm_str.as_str() == "EVM" {
            instructions.extend(vec![
                FISInstruction {
                    plane: PLANE_COSMOS.to_string(),
                    action: ACTION_COSMOS_INVOKE.to_string(),
                    address: "".to_string(),
                    msg: to_json_vec(&MsgAstroTransfer::new(
                        pool_address.to_string(),
                        pool_address.to_string(),
                        PLANE_COSMOS.to_string(),
                        PLANE_EVM.to_string(),
                        Coin {
                            denom: denom_0.clone(),
                            amount: amount_0,
                        },
                    ))?,
                },
                FISInstruction {
                    plane: PLANE_COSMOS.to_string(),
                    action: ACTION_COSMOS_INVOKE.to_string(),
                    address: "".to_string(),
                    msg: to_json_vec(&MsgAstroTransfer::new(
                        pool_address.to_string(),
                        pool_address.to_string(),
                        PLANE_COSMOS.to_string(),
                        PLANE_EVM.to_string(),
                        Coin {
                            denom: denom_1.clone(),
                            amount: amount_1,
                        },
                    ))?,
                },
            ]);

            denom_0 = "eef74ab95099c8d1ad8de02ba6bdab9cbc9dbf93".to_string();
            denom_1 = graduate_event.meme_denom_link;
        }

        if denom_0 > denom_1 {
            (denom_0, denom_1) = (denom_1, denom_0);
            (amount_0, amount_1) = (amount_1, amount_0);
        }

        // 1. pay creator 0.5 SOL, get 1.5 SOL as fee
        // TODO: Generate fee transfers here
        let fee_rate = 0.003;
        let fee = (amount_1.u128() as f64 * fee_rate).round() as u32;

        let price_uin128: u128 = graduate_event.price.into();
        let price = price_uin128 as f64;

        // 2. create pool in target vm
        let pool: Box<dyn PoolManager> = match vm.to_uppercase().as_str() {
            "SVM" => Box::new(Raydium {
                svm_creator: graduate_event.pool_svm_address,
                open_time: env.block.time.seconds(),
            }),
            "WASM" => Box::new(Astroport {
                contract_sequence: contract_sequence.clone(),
            }),
            "EVM" => Box::new(Uniswap {
                fee: fee,
                price: price,
            }),
            _ => {
                deps
                    .api
                    .debug(format!("unknown vm: {}, continue", vm).as_str());
                continue;
            }
        };

        let create_pool_ixs = pool.create_pool_with_initial_liquidity(
            pool_address.clone(),
            denom_0.clone(),
            amount_0,
            denom_1.clone(),
            amount_1,
        );
        instructions.extend(create_pool_ixs);
    }
    Ok(to_json_binary(&StrategyOutput { instructions }).unwrap())
}
