pub mod astromesh;
pub mod evm;
pub mod svm;
pub mod test;
pub mod wasm;
use astromesh::{MsgAstroTransfer, Swap};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, to_json_binary, to_json_vec, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, Uint256
};
use cosmwasm_std::{from_json, Isqrt, StdError};
use spl_token::solana_program::program_error::ProgramError;
use spl_token::solana_program::program_pack::Pack;
use spl_token::state::Account as TokenAccount;
use std::cmp::min;
use std::vec::Vec;
use svm::{raydium, Account};
use wasm::astroport::{self, AssetInfo};

const RAYDIUM: &str = "raydium";
const ASTROPORT: &str = "astroport";
const UNISWAP: &str = "uniswap";

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
pub struct ArbitrageMsg {
    pub source_plane: String,
    pub amount: Uint128,
    pub denom: String,
}

#[cw_serde]
#[derive(Default)]
pub struct Pool {
    pub denom_plane: String,
    pub a: Uint128,
    pub b: Uint128,
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

pub fn get_profit_at_x(a1: u128, b1: u128, a2: u128, b2: u128, x: u128) -> i128 {
    (b2 - (a2 * b2) / (a2 + a1 - a1 * b1 / (b1 + x)) - x) as i128
}

// contract: use x b1 to get a1
// use a1 as a2 to get b2 => profit = output (b1) - input (b1)
pub fn get_max_profit_point(a1: u128, b1: u128, a2: u128, b2: u128) -> (i128, i128) {
    // TODO: prevent overflow
    let optimal_x = (Isqrt::isqrt(a1 * b1)
        .checked_mul(Isqrt::isqrt(a2 * b2))
        .unwrap()
        - b1 * a2)
        / (a1 + a2);
    (
        optimal_x as i128,
        get_profit_at_x(a1, b1, a2, b2, optimal_x),
    )
}

pub fn swap(sender: String, swap: &Swap) -> Result<FISInstruction, StdError> {
    let cloned_swap = swap.to_owned();
    match swap.dex_name.as_str() {
        RAYDIUM => {
            raydium::compose_swap_fis(sender, cloned_swap)
        },
        ASTROPORT => {
            astroport::compose_swap_fis(sender, cloned_swap)
        }
        _ => Err(StdError::generic_err("unsupported dex"))
    }
}

pub fn astro_transfer(
    sender: String,
    src_plane: String,
    dst_plane: String,
    denom: String,
    amount: u128,
) -> FISInstruction {
    FISInstruction {
        plane: "COSMOS".to_string(),
        action: "COSMOS_INVOKE".to_string(),
        address: "".to_string(),
        msg: to_json_vec(&MsgAstroTransfer::new(sender.clone(), sender, src_plane, dst_plane, Coin {
            denom,
            amount: Uint128::from(amount),
        })).unwrap(),
    }
}

fn svm_err_to_std<T>(e: ProgramError) -> Result<T, StdError> {
    Err(StdError::generic_err(e.to_string()))
}

// always make sure the denom aligns across swap
// means pool1.a_denom == pool2.a_denom otherwise the calculation will go wrong
pub fn parse_pool(swap: &Swap, input: &FisInput, reverse: bool) -> Result<Pool, StdError> {
    match swap.dex_name.as_str() {
        RAYDIUM => {
            let token_0_vault_account = Account::from_json_bytes(input.data.get(0).unwrap())?;
            let token_1_vault_account = Account::from_json_bytes(input.data.get(1).unwrap())?;

            let token_0_info = TokenAccount::unpack(token_0_vault_account.data.as_slice())
                .or_else(svm_err_to_std)?;
            let token_1_info = TokenAccount::unpack(token_1_vault_account.data.as_slice())
                .or_else(svm_err_to_std)?;
            // TODO: more constraint as validate basic
            let (mut a, mut b) = (token_0_info.amount, token_1_info.amount);
            if token_0_info.mint.to_string() != swap.input_denom {
                (a, b) = (b, a);
            }

            if reverse {
                (a, b) = (b, a);
            }

            Ok(Pool {
                denom_plane: "SVM".to_string(),
                a: Uint128::from(a as u128),
                b: Uint128::from(b as u128),
            })
        }
        ASTROPORT => {
            let pool_info = from_json::<astroport::PoolResponse>(input.data.get(0).unwrap())?;
            let asset_0 = pool_info.assets.get(0).unwrap();
            let asset_1 = pool_info.assets.get(1).unwrap();
            let asset_0_denom = match asset_0.clone().info {
                AssetInfo::Token { contract_addr } => contract_addr.to_string(),
                AssetInfo::NativeToken { denom } => denom,
            };
            // let asset_1_denom = match asset_1.clone().info {
            //     AssetInfo::Token { contract_addr } => contract_addr.to_string(),
            //     AssetInfo::NativeToken { denom } => denom,
            // };
            let (mut a, mut b) = (asset_0.amount, asset_1.amount);
            if asset_0_denom != swap.input_denom {
                (a, b) = (b, a);
            }

            if reverse {
                (a, b) = (b, a);
            }

            Ok(Pool { 
                denom_plane: "COSMOS".to_string(),
                a, b 
            })
        }
        _ => Err(StdError::generic_err("unsupported dex")),
    }
}

// Let's do astroport + raydium for now
#[entry_point]
pub fn query(_deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let swaps = from_json::<Vec<Swap>>(msg.msg)?;
    // TODO: asserts with meaningful messages
    let src_pool_raw = msg.fis_input.get(0).unwrap();
    let dst_pool_raw = msg.fis_input.get(1).unwrap();
    let src_swap = swaps.get(0).unwrap();
    let mut dst_swap = swaps.get(1).unwrap();

    let src_pool = parse_pool(src_swap, src_pool_raw, false)?;
    // reverse = true because on the 2nd swap, we do reverse denom swap
    // i.e 1st swap: pool 1 usdt => btc
    // 2nd swap: pool 2: btc => usdt
    // but we need to make (a, b) coefficient aligned (means pool1 a's denom = pool2 a's denom)
    let dst_pool = parse_pool(dst_swap, dst_pool_raw, true)?;

    // calculate profit for target pool
    let (mut optimal_x, mut optimal_y) = (0i128, 0i128);

    let (x, y) = get_max_profit_point(
        src_pool.a.u128(),
        src_pool.b.u128(),
        dst_pool.a.u128(),
        dst_pool.b.u128(),
    );
    if y > optimal_y {
        optimal_y = y;
        optimal_x = x;
    }

    if optimal_x <= 0 || optimal_y < 0 {
        // do nothing if there is no profit or can't reach that amount
        return Ok(to_json_binary(&StrategyOutput {
            instructions: vec![],
        })
        .unwrap());
    }

    let execute_amount = min(
        optimal_x as u128,
        src_swap.input_amount.unwrap().i128() as u128,
    );
    let expected_output = if execute_amount == optimal_x as u128 {
        optimal_y
    } else {
        get_profit_at_x(
            src_pool.a.u128(),
            src_pool.b.u128(),
            dst_pool.a.u128(),
            dst_pool.b.u128(),
            execute_amount,
        )
    };

    // use the amount to buy
    let sender = env.contract.address.to_string();
    let instructions = vec![
        swap(sender.clone(), src_swap)?,
        astro_transfer(sender.clone(), src_pool.clone().denom_plane, dst_pool.clone().denom_plane, src_swap.clone().output_denom, optimal_y as u128),
        swap(sender.clone(), dst_swap)?,
        astro_transfer(sender.clone(),  dst_pool.denom_plane, src_pool.denom_plane, dst_swap.clone().output_denom, expected_output as u128),
    ];

    Ok(to_json_binary(&StrategyOutput { instructions }).unwrap())
}
