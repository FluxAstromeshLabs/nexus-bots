pub mod astromesh;
pub mod evm;
pub mod svm;
pub mod test;
pub mod wasm;
use astromesh::{FISInput, MsgAstroTransfer, NexusAction, Pool, Swap};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, to_json_binary, to_json_vec, Binary, Coin, Deps, DepsMut, Env, Int128, Int256,
    MessageInfo, Response, StdResult, Uint128, Uint256,
};
use cosmwasm_std::{from_json, Isqrt, StdError};
use evm::uniswap;
use std::cmp::min;
use std::vec::Vec;
use svm::{raydium, Account, TokenAccount};
use wasm::astroport::{self, AssetInfo};
const UNISWAP: &str = "uniswap"; // to be supported

const BPS: i128 = 1000000i128;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {}

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

pub fn get_output_amount(pool: &Pool, x: Int256, a_for_b: bool) -> Int256 {
    // (a+x)(b-y) = ab
    // y = b - a*b/(a+x) = xb/(a + x)
    let bps = Int256::from_i128(BPS);
    match pool.dex_name.as_str() {
        // astroport applies fee rate after swap => a1*x / (b1 + x) * (1-fee_rate)
        ASTROPORT => {
            if a_for_b {
                (pool.b * x) * (bps - pool.fee_rate) / ((pool.a + x) * bps)
            } else {
                (pool.a * x) * (bps - pool.fee_rate) / ((pool.b + x) * bps)
            }
        }

        // raydium applies fee rate before swap
        RAYDIUM => {
            let x = x * (bps - pool.fee_rate) / bps;

            if a_for_b {
                (pool.b * x) / (pool.a + x)
            } else {
                (pool.a * x) / (pool.b + x)
            }
        }

        UNISWAP => {
            if a_for_b {
                (pool.b * x) / (pool.a + x)
            } else {
                (pool.a * x) / (pool.b + x)
            }
        }

        _ => panic!("unsupported dex"),
    }
}

// swap x from a to b in src_pool, use same b amount to swap b to a in dst_pool
// this function returns output amount of each swap with input x
pub fn calculate_pools_output(src_pool: &Pool, dst_pool: &Pool, x: Int256) -> (Int256, Int256) {
    // swap a for b in src_pool
    let first_swap_output = get_output_amount(src_pool, x, true);
    // swap b for a in dst_pool
    let second_swap_output = get_output_amount(dst_pool, first_swap_output, false);
    (first_swap_output, second_swap_output)
}

pub fn to_uint256(i: Int256) -> Uint256 {
    Uint256::from_be_bytes(i.to_be_bytes())
}

pub fn to_int256(i: Uint256) -> Int256 {
    Int256::from_be_bytes(i.to_be_bytes())
}

pub fn to_u128(i: Int256) -> u128 {
    u128::from_be_bytes(i.to_be_bytes()[16..32].try_into().expect("must be u128"))
}

// contract: use x a1 to get b1
// use a1 as a2 to get b2 => profit = output (b1) - input (b1)
pub fn get_max_profit_point(a1: Int256, b1: Int256, a2: Int256, b2: Int256) -> Int256 {
    let optimal_x =
        (to_int256(Isqrt::isqrt(to_uint256(a1 * b1)) * Isqrt::isqrt(to_uint256(a2 * b2)))
            - a1 * b2)
            / (b1 + b2);
    optimal_x
}

pub fn swap(sender: String, swap: &Swap) -> Result<FISInstruction, StdError> {
    let cloned_swap = swap.to_owned();
    match swap.dex_name.as_str() {
        RAYDIUM => raydium::compose_swap_fis(sender, cloned_swap),
        ASTROPORT => astroport::compose_swap_fis(sender, cloned_swap),
        _ => Err(StdError::generic_err("unsupported dex")),
    }
}

pub fn astro_transfer(
    sender: String,
    src_plane: &String,
    dst_plane: &String,
    mut denom: String,
    amount: u128,
) -> FISInstruction {
    if src_plane == "EVM" || src_plane == "SVM" {
        denom = String::from("astro/") + &denom;
    }

    FISInstruction {
        plane: "COSMOS".to_string(),
        action: "COSMOS_INVOKE".to_string(),
        address: "".to_string(),
        msg: to_json_vec(&MsgAstroTransfer::new(
            sender.clone(),
            sender,
            src_plane.clone(),
            dst_plane.clone(),
            Coin {
                denom,
                amount: Uint128::from(amount),
            },
        ))
        .unwrap(),
    }
}

// this estimates optimal_x with pool fee
pub fn adjust_optimal_x(optimal_x: Int256, src_pool: Pool, dst_pool: Pool) -> Int256 {
    // ratio = a1/a_decimal/(b1 / b_decimal) / (a2/a_decimal/(b2/b_decimal)) - 1
    let a1 = src_pool.a;
    let b1 = src_pool.b;
    let a2 = dst_pool.a;
    let b2 = dst_pool.b;
    let one = Int256::from_i128(1_000_000i128);
    let ratio = a1 * b2 * one / (b1 * a2) - one;
    // price change > -1%
    if ratio >= Int256::from_i128(-10_000i128) {
        optimal_x * Int256::from(90) / Int256::from(100)
    }
    // price change > -2%
    else if ratio >= Int256::from_i128(-20_000i128) {
        optimal_x * Int256::from(95) / Int256::from(100)
    } else {
        optimal_x * Int256::from(99) / Int256::from(100)
    }
}

pub fn arbitrage(
    deps: Deps,
    env: Env,
    pair: String,
    amount: Int128,
    min_profit: Option<Int256>,
    fis_input: &Vec<FISInput>,
) -> StdResult<Binary> {
    // TODO: asserts with meaningful messages
    // wasm, svm
    // pool queries
    let raw_pools = [
        fis_input
            .get(0)
            .ok_or(StdError::not_found("astroport pool data"))?,
        fis_input
            .get(1)
            .ok_or(StdError::not_found("raydium pool data"))?,
    ];

    // parse pools
    let parsed_pools = [
        &astroport::parse_pool(raw_pools[0])?,
        &raydium::parse_pool(raw_pools[1])?,
    ];

    let mut src_pool: &Pool;
    let mut dst_pool: &Pool;

    let (mut lowest_rate, mut highest_rate) = (Int256::MAX, Int256::MIN);
    // detect best swap rate
    // buy on low rate and sell on higher rate
    for i in 0..parsed_pools.len() {
        let rate = parsed_pools[i].a / parsed_pools[i].b;
        if lowest_rate > rate {
            src_pool = parsed_pools[i];
        }

        if highest_rate < rate {
            dst_pool = parsed_pools[i];
        }
    }

    // 2 pools are the same, no way but defense check
    if src_pool.dex_name == dst_pool.dex_name {
        return Err(StdError::generic_err("two detected swap pool are same, nothing executed"))
    }

    // compose the swap
    let src_swap = Swap { 
        dex_name: src_pool.dex_name, 
        pool_name: pair, 
        denom: "usdt".to_string(), 
        amount: amount,
    };

    let mut dst_swap = Swap {
        dex_name: dst_pool.dex_name,
        pool_name: pair,
        denom: "".to_string(),
        amount: Int128::zero(), // to be updated later
    };

    // // TODO: detect source and dst swap
    // let src_swap = swaps.get(0).unwrap().clone();
    // let mut dst_swap = swaps.get(1).unwrap().clone();
    // calculate profit for target pool with ideal scenario (no pool fees)
    let optimal_x = get_max_profit_point(src_pool.a, src_pool.b, dst_pool.a, dst_pool.b);
    let optimal_x = adjust_optimal_x(optimal_x, src_pool.clone(), dst_pool.clone());
    let (_, second_optimal_swap_output) = calculate_pools_output(&src_pool, &dst_pool, optimal_x);
    let profit = second_optimal_swap_output - optimal_x;
    deps.api.debug(
        format!(
            "pool s {:#?}, pool d: {:#?}, optimal x: {}, optimal profit: {}",
            src_pool, dst_pool, optimal_x, profit
        )
        .as_str(),
    );

    if optimal_x <= Int256::zero() || profit <= Int256::zero() {
        // do nothing if there is no profit or can't reach that amount
        return Ok(to_json_binary(&StrategyOutput {
            instructions: vec![],
        })
        .unwrap());
    }

    let execute_amount = min(
        optimal_x,
        Int256::from(src_swap.amount.i128()),
    );
    let (first_swap_output, second_swap_output) =
        calculate_pools_output(&src_pool, &dst_pool, execute_amount);
    let sender = env.contract.address.to_string();
    dst_swap.amount = Int128::from_be_bytes(
        first_swap_output.to_be_bytes()[16..32].try_into().unwrap(),
    );

    // actions, take usdt > btc arbitrage as example
    // 1. do swap usdt to btc src pool
    // 2. transfer the swapped btc amount to dst pool
    // 3. swap the btc amount to usdt in dst pool
    // 4. transfer usdt back to src pool
    let instructions = vec![
        swap(sender.clone(), &src_swap)?,
        astro_transfer(
            sender.clone(),
            &src_pool.denom_plane,
            &dst_pool.denom_plane,
            src_swap.output_denom,
            to_u128(first_swap_output),
        ),
        swap(sender.clone(), &dst_swap)?,
        astro_transfer(
            sender.clone(),
            &dst_pool.denom_plane,
            &src_pool.denom_plane,
            dst_swap.clone().output_denom,
            to_u128(second_swap_output),
        ),
    ];

    Ok(to_json_binary(&StrategyOutput { instructions }).unwrap())
}

// Let's do astroport + raydium for now
#[entry_point]
pub fn query(_deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let action = from_json::<Vec<NexusAction>>(msg.msg)?;
    match action {
        NexusAction::Arbitrage {
            pair,
            usdt_amount,
            min_profit,
        } => arbitrage(deps, env, pair, amount, min_profit),
        // more actions goes here
    }
}
