pub mod evm;
pub mod svm;
pub mod test;
pub mod wasm;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Isqrt;
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128, Uint256,
};
use std::cmp::min;
use std::vec::Vec;

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
pub struct ArbitrageMsg {
    pub source_plane: String,
    pub amount: Uint128,
    pub denom: String,
}

pub struct Pool {
    pub plane: String,
    pub id: String,
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

// TODO: Add contract
pub fn swap(plane: String, pool_id: String, x: u128, zeroForOne: bool) -> Option<FISInstruction> {
    // TODO: implement
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

#[entry_point]
pub fn query(_deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    // _deps.querier
    let instructions = vec![];
    // calculate optimal value, check decimals
    // input 3 planes [wasm] [evm] [svm]
    let msg = ArbitrageMsg {
        source_plane: "EVM".to_string(),
        amount: Uint128::from(100_000u128),
        denom: "1a2b3c".to_string(),
    };

    let pools = vec![
        Pool {
            plane: "WASM".to_string(),
            id: "1".to_string(),
            a: Uint128::from(100u128),
            b: Uint128::from(6_400_000u128),
        },
        Pool {
            plane: "EVM".to_string(),
            id: "1".to_string(),
            a: Uint128::from(100u128),
            b: Uint128::from(6_500_000u128),
        },
        Pool {
            plane: "SVM".to_string(),
            id: "1".to_string(),
            a: Uint128::from(100u128),
            b: Uint128::from(6_500_000u128),
        },
    ];

    let src_pool = pools.iter().find(|p| p.plane == msg.source_plane).unwrap();

    // calculate profit for target pool
    let mut selected_pool: &Pool = &pools[0];
    let (mut optimal_x, mut optimal_y) = (0i128, 0i128);
    pools.iter().for_each(|p| {
        if p.plane == msg.source_plane {
            return;
        }

        let (x, y) =
            get_max_profit_point(src_pool.a.u128(), src_pool.b.u128(), p.a.u128(), p.b.u128());
        if y > optimal_y {
            optimal_y = y;
            optimal_x = x;
            selected_pool = p;
        }
    });

    if optimal_x <= 0 || optimal_y < 0 {
        // do nothing if there is no profit or can't reach that amount
        return Ok(to_json_binary(&StrategyOutput { instructions }).unwrap());
    }

    let execute_amount = min(optimal_x as u128, msg.amount.u128());
    let expected_output = if execute_amount == optimal_x as u128 {
        optimal_y
    } else {
        get_profit_at_x(
            src_pool.a.u128(),
            src_pool.b.u128(),
            selected_pool.a.u128(),
            selected_pool.b.u128(),
            execute_amount,
        )
    };

    // use the amount to buy
    let instructions = vec![
        swap(
            msg.clone().source_plane,
            src_pool.id.clone(),
            execute_amount,
            true,
        )
        .unwrap(),
        astro_transfer(
            msg.source_plane.clone(),
            selected_pool.plane.clone(),
            msg.denom,
            expected_output as u128,
        )
        .unwrap(),
        swap(
            selected_pool.plane.to_string(),
            selected_pool.id.clone(),
            expected_output as u128,
            false,
        )
        .unwrap(), // zero for one? we don't know
    ];

    Ok(to_json_binary(&StrategyOutput { instructions }).unwrap())
}
