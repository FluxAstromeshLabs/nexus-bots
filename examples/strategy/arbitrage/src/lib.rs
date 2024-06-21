pub mod astromesh;
pub mod evm;
pub mod svm;
pub mod test;
pub mod wasm;
use astromesh::Swap;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{from_json, Isqrt};
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128, Uint256,
};
use std::cmp::min;
use std::vec::Vec;

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


pub fn swap(swap_input: Swap) -> Option<FISInstruction> {
    None
}

pub fn astro_transfer(
    src_plane: String,
    dst_plane: String,
    denom: String,
    amount: u128,
) -> Option<FISInstruction> {
    None
}

// always make sure the denom aligns across swap
// means pool1.a_denom == pool2.a_denom otherwise the calculation will go wrong
pub fn parse_pool(swap: &Swap, input: &FisInput, reverse: bool) -> Pool {
    Pool::default()
}

// Let's do astroport + raydium for now
#[entry_point]
pub fn query(_deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let swaps = from_json::<Vec<Swap>>(msg.msg)?;
    // TODO: asserts with meaningful messages
    let src_pool_raw = msg.fis_input.get(0).unwrap();
    let dst_pool_raw = msg.fis_input.get(1).unwrap();
    let src_swap = swaps.get(0).unwrap();
    let dst_swap = swaps.get(1).unwrap();

    let src_pool  = parse_pool(src_swap, src_pool_raw, false);
    // reverse = true because on the 2nd swap, we do reverse denom swap 
    // i.e 1st swap: pool 1 usdt => btc
    // 2nd swap: pool 2: btc => usdt
    // but we need to make (a, b) coefficient aligned (means pool1 a's denom = pool2 a's denom)
    let dst_pool  = parse_pool(src_swap, src_pool_raw, true);

    // calculate profit for target pool
    let (mut optimal_x, mut optimal_y) = (0i128, 0i128);
    
    let (x, y) =
        get_max_profit_point(src_pool.a.u128(), src_pool.b.u128(), dst_pool.a.u128(), dst_pool.b.u128());
    if y > optimal_y {
        optimal_y = y;
        optimal_x = x;
    }

    if optimal_x <= 0 || optimal_y < 0 {
        // do nothing if there is no profit or can't reach that amount
        return Ok(to_json_binary(&StrategyOutput { instructions: vec![] }).unwrap());
    }

    let execute_amount = min(optimal_x as u128, src_swap.input_amount.unwrap().i128() as u128);
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
    let instructions = vec![
        // swap().unwrap(),
        // astro_transfer(
        //     msg.source_plane.clone(),
        //     selected_pool.plane.clone(),
        //     msg.denom,
        //     expected_output as u128,
        // )
        // .unwrap(),
        // swap(
        //     selected_pool.plane.to_string(),
        //     selected_pool.id.clone(),
        //     expected_output as u128,
        //     false,
        // )
        // .unwrap(), // zero for one? we don't know
    ];

    Ok(to_json_binary(&StrategyOutput { instructions }).unwrap())
}
