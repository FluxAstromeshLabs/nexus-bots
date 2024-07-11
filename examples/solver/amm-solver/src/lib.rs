pub mod astromesh;
pub mod evm;
pub mod svm;
pub mod test;
pub mod wasm;
use astromesh::{
    to_int256, to_u128, to_uint256, FISInput, FISInstruction, MsgAstroTransfer, NexusAction, Pool,
    Swap, ETH_DECIMAL_DIFF,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, to_json_binary, to_json_vec, Binary, Coin, Deps, DepsMut, Env, Int128, Int256,
    MessageInfo, Response, StdResult, Uint128,
};
use cosmwasm_std::{from_json, Isqrt, StdError};
use evm::uniswap::UniswapPool;
use std::cmp::min;
use std::vec::Vec;
use svm::get_denom;
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
    mut amount: u128,
) -> FISInstruction {
    if src_plane == "SVM" && denom == get_denom("eth") {
        amount = amount / ETH_DECIMAL_DIFF
    }

    // round up for eth decimal diff, chain could handle the conversion too
    if dst_plane == "SVM" && denom == "eth" {
        amount = (amount / ETH_DECIMAL_DIFF) * ETH_DECIMAL_DIFF
    }

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

// Arbitrage supports astroport + raydium for now
// fis_input injects all pool for now
// format: 
// [wasm btc-usdt, svm btc-usdt, wasm eth-usdt, svm btc-usdt, wasm sol-usdt, svm sol-usdt]
pub fn arbitrage(
    deps: Deps,
    env: Env,
    pair: String,
    amount: Int128,
    min_profit: Option<Int128>,
    fis_input: &Vec<FISInput>,
) -> StdResult<Binary> {
    must_support(&pair)?;
    let pool_index = match pair.as_str() {
        "btc-usdt" => 0,
        "eth-usdt" => 2,
        "sol-usdt" => 4,
        _ => unreachable!()
    };

    let raw_pools = [
        fis_input
            .get(pool_index)
            .ok_or(StdError::generic_err("astroport pool data not found"))?,
        fis_input
            .get(pool_index+1)
            .ok_or(StdError::generic_err("raydium pool data not found"))?,
    ];

    // parse pools
    let parsed_pools: Vec<Box<dyn Pool>> = vec![
        Box::new(AstroportPool::from_fis(raw_pools[0])?),
        Box::new(RaydiumPool::from_fis(raw_pools[1])?),
    ];

    deps.api.debug(
        format!(
            "parsed pools 0: {:#?}",
            AstroportPool::from_fis(raw_pools[0])?
        )
        .as_str(),
    );
    deps.api.debug(
        format!(
            "parsed pools 1: {:#?}",
            RaydiumPool::from_fis(raw_pools[1])?
        )
        .as_str(),
    );
    // detect best swap route, i.e
    // buy on low rate and sell on higher rate pool
    let mut src_pool_opt: Option<&Box<dyn Pool>> = None;
    let mut dst_pool_opt: Option<&Box<dyn Pool>> = None;
    let (mut lowest_rate, mut highest_rate) = (Int256::MAX, Int256::MIN);

    let multiplier = Int256::from_i128(1_000_000_000_000_000_000i128);
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
            "arbitrage from {} => {}, optimal x: {}, estimate first swap output: {}, estimate profit: {}",
            src_pool.dex_name(),
            dst_pool.dex_name(),
            optimal_x,
            first_swap_output,
            profit,
        )
        .as_str(),
    );

    let expected_min_profit = if let Some(mp) = min_profit {
        Int256::from_i128(mp.i128())
    } else {
        Int256::zero()
    };

    if optimal_x <= Int256::zero() || profit < expected_min_profit {
        // do nothing if there is no profit or can't reach that amount, early stopping
        return Ok(to_json_binary(&StrategyOutput {
            instructions: vec![],
        })
        .unwrap());
    }

    // compose the swaps
    let sender = env.contract.address.to_string();
    let mut src_swap = Swap {
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
    let profit = second_swap_output - execute_amount;
    // check again because input could be less than optimal_x => less profit than using optimal_x
    if profit < expected_min_profit {
        return Ok(to_json_binary(&StrategyOutput {
            instructions: vec![],
        })
        .unwrap());
    }

    let sender = env.contract.address.to_string();
    src_swap.amount =
        Int128::from_be_bytes(execute_amount.to_be_bytes()[16..32].try_into().unwrap());
    dst_swap.amount =
        Int128::from_be_bytes(first_swap_output.to_be_bytes()[16..32].try_into().unwrap());
    deps.api.debug(
        format!(
            "arbitrage from {} => {}, actual x: {}, estimate first swap output: {}, estimate second swap output: {}, estimate profit: {}",
            src_pool.dex_name(),
            dst_pool.dex_name(),
            src_swap.amount,
            first_swap_output,
            second_swap_output,
            second_swap_output - execute_amount,
        )
        .as_str(),
    );

    // actions, take usdt > btc arbitrage as example
    // 1. do swap usdt to btc src pool
    // 2. transfer the swapped btc amount to dst pool
    // 3. swap the btc amount to usdt in dst pool
    // 4. transfer usdt back to src pool
    // TODO: Discuss and decide if initial denom should come from cosmos
    let mut instructions = vec![];
    instructions.extend(src_pool.compose_swap_fis(&src_swap)?);
    instructions.push(astro_transfer(
        sender.clone(),
        &src_pool.denom_plane(),
        &dst_pool.denom_plane(),
        first_output_denom,
        to_u128(first_swap_output),
    ));
    instructions.extend(dst_pool.compose_swap_fis(&dst_swap)?);
    instructions.push(astro_transfer(
        sender.clone(),
        &dst_pool.denom_plane(),
        &src_pool.denom_plane(),
        second_output_denom,
        to_u128(second_swap_output),
    ));

    Ok(to_json_binary(&StrategyOutput { instructions }).unwrap())
}

pub fn swap(
    deps: Deps,
    env: Env,
    dex_name: String,
    src_denom: String,
    dst_denom: String,
    amount: Int128,
    _fis_input: &Vec<FISInput>,
) -> StdResult<Binary> {
    if src_denom != "usdt" && dst_denom != "usdt" {
        return Err(StdError::generic_err(format!(
            "Unsupported swap from {} to {}. Supported pairs: btc-usdt, eth-usdt, sol-usdt",
            src_denom, dst_denom
        )));
    }

    let pair = if src_denom == "usdt" {
        format!("{}-{}", dst_denom, src_denom)
    } else {
        format!("{}-{}", src_denom, dst_denom)
    };

    let swap = &Swap {
        dex_name: dex_name.clone(),
        pool_name: pair.clone(),
        sender: env.contract.address.to_string(),
        denom: src_denom,
        amount,
    };

    match str::to_lowercase(&dex_name).as_str() {
        "svm raydium" => {
            let pool = RaydiumPool::new(&pair)?;
            let instructions = pool.compose_swap_fis(swap)?;
            Ok(to_json_binary(&StrategyOutput { instructions })?)
        }

        "wasm astroport" => {
            let pool = AstroportPool::new(&pair)?;
            let instructions = pool.compose_swap_fis(swap)?;
            Ok(to_json_binary(&StrategyOutput { instructions })?)
        }

        "evm uniswap" => {
            let pool = UniswapPool::new(&pair)?;
            let instructions = pool.compose_swap_fis(swap)?;
            Ok(to_json_binary(&StrategyOutput { instructions })?)
        }

        _ => Err(StdError::generic_err(format!(
            "Unsupported: {}. Supported: 'svm raydium', 'wasm astroport', 'evm uniswap'",
            dex_name
        ))),
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let action = from_json::<NexusAction>(msg.msg)?;
    match action {
        NexusAction::Arbitrage {
            pair,
            amount,
            min_profit,
        } => arbitrage(deps, env, pair, amount, min_profit, &msg.fis_input),

        NexusAction::Swap {
            dex_name,
            src_denom,
            dst_denom,
            amount,
        } => swap(
            deps,
            env,
            dex_name,
            src_denom,
            dst_denom,
            amount,
            &msg.fis_input,
        ),
        // more actions goes here
    }
}
