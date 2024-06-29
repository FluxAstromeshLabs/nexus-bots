pub mod astromesh;
pub mod svm;
pub mod test;
pub mod wasm;
use astromesh::{to_int256, to_u128, to_uint256, FISInput, FISInstruction, MsgAstroTransfer, NexusAction, Pool, Swap};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, to_json_binary, to_json_vec, Binary, Coin, Deps, DepsMut, Env, Int128, Int256,
    MessageInfo, Response, StdResult, Uint128, Uint256,
};
use cosmwasm_std::{from_json, Isqrt, StdError};
use std::cmp::min;
use std::vec::Vec;
use svm::raydium::RaydiumPool;
use wasm::astroport::AstroportPool;

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
pub struct StrategyOutput {
    instructions: Vec<FISInstruction>,
}

// swap x from a to b in src_pool, use same b amount to swap b to a in dst_pool
// this function returns output amount of each swap with input x
pub fn calculate_pools_output(
    src_pool: &Box<dyn Pool>,
    dst_pool: &Box<dyn Pool>,
    x: Int256,
) -> (String, Int256, String, Int256) {
    // swap a for b in src_pool
    let (first_output_denom, first_swap_output) = src_pool.swap_output(x, true);
    // swap b for a in dst_pool
    let (second_output_denom, second_swap_output) = dst_pool.swap_output(first_swap_output, false);
    (
        first_output_denom,
        first_swap_output,
        second_output_denom,
        second_swap_output,
    )
}

// contract: use x a1 to get b1
// use a1 as a2 to get b2 => profit = output (b1) - input (b1)
pub fn get_max_profit_point(a1: Int256, b1: Int256, a2: Int256, b2: Int256) -> Int256 {
    (to_int256(Isqrt::isqrt(to_uint256(a1 * b1)) * Isqrt::isqrt(to_uint256(a2 * b2))) - a1 * b2)
        / (b1 + b2)
}

pub fn get_pair_output_denom(input_denom: &str, pair: &String) -> String {
    match pair.as_str() {
        "btc-usdt" => {
            if input_denom == "btc" {
                "usdt".to_string()
            } else {
                "btc".to_string()
            }
        }
        "eth-usdt" => {
            if input_denom == "eth" {
                "usdt".to_string()
            } else {
                "eth".to_string()
            }
        }
        "sol-usdt" => {
            if input_denom == "sol" {
                "usdt".to_string()
            } else {
                "sol".to_string()
            }
        }
        _ => "".to_string(),
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
pub fn adjust_optimal_x(
    optimal_x: Int256,
    src_pool: &Box<dyn Pool>,
    dst_pool: &Box<dyn Pool>,
) -> Int256 {
    // ratio = a1/a_decimal/(b1 / b_decimal) / (a2/a_decimal/(b2/b_decimal)) - 1
    let a1 = src_pool.a();
    let b1 = src_pool.b();
    let a2 = dst_pool.a();
    let b2 = dst_pool.b();
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

fn must_support(pair: &String) -> Result<(), StdError> {
    if get_pair_output_denom("usdt", pair) == "" {
        return Err(StdError::generic_err(
            format!("unsupported pair: {}", pair).as_str(),
        ));
    };

    Ok(())
}

pub fn arbitrage(
    deps: Deps,
    env: Env,
    pair: String,
    amount: Int128,
    min_profit: Option<Int128>,
    fis_input: &Vec<FISInput>,
) -> StdResult<Binary> {
    must_support(&pair)?;

    let raw_pools = [
        fis_input
            .first()
            .ok_or(StdError::generic_err("astroport pool data not found"))?,
        fis_input
            .get(1)
            .ok_or(StdError::generic_err("raydium pool data not found"))?,
    ];

    // parse pools
    let parsed_pools: Vec<Box<dyn Pool>> = vec![
        Box::new(AstroportPool::from_fis(raw_pools[0])?),
        Box::new(RaydiumPool::from_fis(raw_pools[1])?),
    ];

    // detect best swap route, i.e
    // buy on low rate and sell on higher rate pool
    let mut src_pool_opt: Option<&Box<dyn Pool>> = None;
    let mut dst_pool_opt: Option<&Box<dyn Pool>> = None;
    let (mut lowest_rate, mut highest_rate) = (Int256::MAX, Int256::MIN);

    let multiplier = Int256::from_i128(1_000_000_000_000_000_000_000_000_000i128);
    for i in 0..parsed_pools.len() {
        // trick: use multiplier to get over usdt and other denom's decimal
        // it's fine to compare the ratios with same multiplier 
        let rate = parsed_pools[i].a() * multiplier / parsed_pools[i].b();
        if lowest_rate > rate {
            src_pool_opt = Some(&parsed_pools[i]);
            lowest_rate = rate
        }

        if highest_rate < rate {
            dst_pool_opt = Some(&parsed_pools[i]);
            highest_rate = rate
        }
    }

    let (src_pool, dst_pool) = (src_pool_opt.unwrap(), dst_pool_opt.unwrap());

    // calculate profit for target pool with ideal scenario (no pool fees)
    let optimal_x = get_max_profit_point(src_pool.a(), src_pool.b(), dst_pool.a(), dst_pool.b());
    let optimal_x = adjust_optimal_x(optimal_x, src_pool, dst_pool);
    let (_, first_swap_output, _, second_optimal_swap_output) =
        calculate_pools_output(src_pool, dst_pool, optimal_x);
    let profit = second_optimal_swap_output - optimal_x;
    deps.api.debug(
        format!(
            "optimal x: {}, estimate first swap output: {}, estimate profit: {}, swap route: {} => {}",
            optimal_x,
            first_swap_output,
            profit,
            src_pool.dex_name(),
            dst_pool.dex_name(),
        )
        .as_str(),
    );

    let expected_min_profit = if let Some(mp) = min_profit {
        Int256::from_i128(mp.i128())
    } else {
        Int256::zero()
    };

    if optimal_x <= Int256::zero() || profit < expected_min_profit {
        // do nothing if there is no profit or can't reach that amount
        return Ok(to_json_binary(&StrategyOutput {
            instructions: vec![],
        })
        .unwrap());
    }

    // compose the swaps
    let sender = env.contract.address.to_string();
    let src_swap = Swap {
        sender: sender.clone(),
        dex_name: src_pool.dex_name(),
        pool_name: pair.clone(),
        denom: "usdt".to_string(),
        amount,
    };

    let mut dst_swap = Swap {
        sender,
        dex_name: dst_pool.dex_name(),
        pool_name: pair.clone(),
        denom: get_pair_output_denom("usdt", &pair),
        amount: Int128::zero(), // to be updated after calculations
    };

    let execute_amount = min(optimal_x, Int256::from(src_swap.amount.i128()));
    let (first_output_denom, first_swap_output, second_output_denom, second_swap_output) =
        calculate_pools_output(src_pool, dst_pool, execute_amount);
    let sender = env.contract.address.to_string();
    dst_swap.amount =
        Int128::from_be_bytes(first_swap_output.to_be_bytes()[16..32].try_into().unwrap());

    // actions, take usdt > btc arbitrage as example
    // 1. do swap usdt to btc src pool
    // 2. transfer the swapped btc amount to dst pool
    // 3. swap the btc amount to usdt in dst pool
    // 4. transfer usdt back to src pool
    // TODO: Discuss and decide if initial denom should come from cosmos
    let instructions = vec![
        src_pool.compose_swap_fis(&src_swap)?,
        astro_transfer(
            sender.clone(),
            &src_pool.denom_plane(),
            &dst_pool.denom_plane(),
            first_output_denom,
            to_u128(first_swap_output),
        ),
        dst_pool.compose_swap_fis(&dst_swap)?,
        astro_transfer(
            sender.clone(),
            &dst_pool.denom_plane(),
            &src_pool.denom_plane(),
            second_output_denom,
            to_u128(second_swap_output),
        ),
    ];

    Ok(to_json_binary(&StrategyOutput { instructions }).unwrap())
}

// Let's do astroport + raydium for now
#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let action = from_json::<NexusAction>(msg.msg)?;
    match action {
        NexusAction::Arbitrage {
            pair,
            amount,
            min_profit,
        } => arbitrage(deps, env, pair, amount, min_profit, &msg.fis_input),
        // more actions goes here
    }
}
