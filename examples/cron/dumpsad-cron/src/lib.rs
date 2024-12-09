use astromesh::{
    FISInstruction, MsgAstroTransfer, PoolManager, QueryDenomLinkResponse, ACTION_COSMOS_INVOKE, ACTION_VM_INVOKE, PLANE_COSMOS, PLANE_EVM, PLANE_SVM, PLANE_WASM
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, to_json_vec, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128
};
use std::vec::Vec;
use svm::{raydium::Raydium, AccountLink};
use wasm::astroport::Astroport;
mod astromesh;
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
    pub vm: String,
    pub pool_address: String,
}

#[entry_point]
pub fn query(_deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let cron_msg = from_json::<CronMsg>(msg.msg)?;
    let vm = cron_msg.vm;
    let pool_address = cron_msg.pool_address;

    // get current liquidity source
    let quote_coin = from_json::<Coin>(msg.fis_input.get(0).unwrap().data.get(0).unwrap())?; // SOL
    let meme_coin = from_json::<Coin>(msg.fis_input.get(0).unwrap().data.get(1).unwrap())?;
    // TODO: pool graduate condition (phuc)
    let graduated = quote_coin.amount.gt(&Uint128::one());
    let mut instructions = vec![];
    if graduated {
        let contract_sequence = msg.fis_input.get(1).unwrap().data.get(0).unwrap();

        // account link and denom link
        let account_link =
            from_json::<AccountLink>(msg.fis_input.get(2).unwrap().data.get(0).unwrap())?;
        let linked_svm = account_link.link.svm_addr;

        let (mut denom_0, mut denom_1) = (quote_coin.denom.clone(), meme_coin.denom.clone());
        match vm.as_str() {
            PLANE_EVM => {
                // do something with the denom
            }
            PLANE_SVM => {
                let svm_coin =
                    from_json::<Coin>(msg.fis_input.get(3).unwrap().data.get(2).unwrap())?;
                denom_0 = "CPozhCGVaGAcPVkxERsUYat4b7NKT9QeAR9KjNH4JpDG".to_string();
                denom_1 = svm_coin.denom;

                // move all funds to SVM
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
                                denom: quote_coin.denom,
                                amount: quote_coin.amount,
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
                                denom: meme_coin.denom,
                                amount: meme_coin.amount,
                            },
                        ))?,
                    }
                ]);
            }
            _ => {}
        }

        let (mut amount_0, mut amount_1) = (quote_coin.amount, meme_coin.amount);
        if denom_0 > denom_1 {
            (denom_0, denom_1) = (denom_1, denom_0);
            (amount_0, amount_1) = (amount_1, amount_0);
        }

        // 1. pay creator 0.5 SOL, get 1.5 SOL as fee
        // TODO(Minh): Generate fee transfers here
        // 2. create pool in target vm
        _deps.api.debug(
            format!(
                "graduate: vm: {}, coin0:{}{}, coin1:{}{}",
                vm,
                amount_0.u128(),
                denom_0.to_string(),
                amount_1.u128(),
                denom_1.to_string(),
            )
            .as_str(),
        );
        let pool: Box<dyn PoolManager> = match vm.as_str() {
            "SVM" => Box::new(Raydium {
                svm_creator: linked_svm,
                open_time: env.block.time.seconds(),
            }),
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
    }

    Ok(to_json_binary(&StrategyOutput { instructions }).unwrap())
}
