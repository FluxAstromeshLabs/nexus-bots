pub mod astromesh;
pub mod evm;
pub mod svm;
pub mod test;
pub mod wasm;
use astromesh::{MsgAstroTransfer, Swap};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, to_json_binary, to_json_vec, Binary, Coin, Deps, DepsMut, Env, Int128, Int256,
    MessageInfo, Response, StdResult, Uint128, Uint256,
};
use cosmwasm_std::{from_json, Isqrt, StdError};
use evm::uniswap;
use std::cmp::min;
use std::collections::btree_set::Union;
use std::vec::Vec;
use svm::{raydium, Account, TokenAccount};
use wasm::astroport::{self, AssetInfo};

const RAYDIUM: &str = "raydium";
const ASTROPORT: &str = "astroport";
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
    pub dex_name: String,
    pub denom_plane: String,
    pub a: Int256,
    pub b: Int256,
    pub fee_rate: Int256,
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

// always make sure the denom aligns across swap
// means pool1.a_denom == pool2.a_denom otherwise the calculation will go wrong
pub fn parse_pool(swap: &Swap, input: &FisInput, reverse: bool) -> Result<Pool, StdError> {
    match swap.dex_name.as_str() {
        RAYDIUM => {
            let token_0_vault_account = Account::from_json_bytes(input.data.get(0).unwrap())?;
            let token_1_vault_account = Account::from_json_bytes(input.data.get(1).unwrap())?;
            let token_0_info = TokenAccount::unpack(token_0_vault_account.data.as_slice())?;
            let token_1_info = TokenAccount::unpack(token_1_vault_account.data.as_slice())?;
            // TODO: more constraint as validate basic
            let (mut a, mut b) = (token_0_info.amount, token_1_info.amount);
            if token_0_info.mint.to_string() != swap.input_denom {
                (a, b) = (b, a);
            }

            if reverse {
                (a, b) = (b, a);
            }

            Ok(Pool {
                dex_name: RAYDIUM.to_string(),
                denom_plane: "SVM".to_string(),
                a: Int256::from_i128(a as i128),
                b: Int256::from_i128(b as i128),
                fee_rate: Int256::from(1000i128),
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
                dex_name: ASTROPORT.to_string(),
                denom_plane: "COSMOS".to_string(),
                a: Int256::from(a.u128()),
                b: Int256::from(b.u128()),
                fee_rate: Int256::from(10000i128),
            })
        }
        UNISWAP => {
            let pool_info = uniswap::parse_pool_info(input.data.get(0).unwrap().as_slice())?;
            let liquidity =
                Uint256::from_be_bytes(input.data.get(1).unwrap().as_slice().try_into().unwrap());

            let uniswap_params = swap.uniswap_input.clone().unwrap();
            let (mut a, mut b) = pool_info.calculate_liquidity_amounts(
                liquidity,
                uniswap_params.lower_tick,
                uniswap_params.upper_tick,
            );
            if !uniswap_params.zero_for_one {
                (a, b) = (b, a)
            }

            if reverse {
                (a, b) = (b, a)
            }
            Ok(Pool {
                dex_name: UNISWAP.to_string(),
                denom_plane: "EVM".to_string(),
                a,
                b,
                fee_rate: Int256::from(3000i128),
            })
        }
        _ => Err(StdError::generic_err("unsupported dex")),
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

// Let's do astroport + raydium for now
#[entry_point]
pub fn query(_deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let swaps = from_json::<Vec<Swap>>(msg.msg)?;
    // TODO: asserts with meaningful messages
    let src_pool_raw = msg.fis_input.get(0).unwrap();
    let dst_pool_raw = msg.fis_input.get(1).unwrap();
    let src_swap = swaps.get(0).unwrap().clone();
    let mut dst_swap = swaps.get(1).unwrap().clone();

    let src_pool = parse_pool(&src_swap, src_pool_raw, false)?;
    // reverse = true because on the 2nd swap, we do reverse denom swap
    // i.e 1st swap: pool 1 usdt => btc
    // 2nd swap: pool 2: btc => usdt
    // but we need to make (a, b) coefficient aligned (means pool1 a's denom = pool2 a's denom)
    let dst_pool = parse_pool(&dst_swap, dst_pool_raw, true)?;

    _deps.api.debug("parsed pools");
    // calculate profit for target pool with ideal scenario (no pool fees)
    let optimal_x = get_max_profit_point(src_pool.a, src_pool.b, dst_pool.a, dst_pool.b);
    let optimal_x = adjust_optimal_x(optimal_x, src_pool.clone(), dst_pool.clone());
    let (_, second_optimal_swap_output) = calculate_pools_output(&src_pool, &dst_pool, optimal_x);
    let profit = second_optimal_swap_output - optimal_x;
    _deps.api.debug(
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
        Int256::from(src_swap.input_amount.unwrap().i128()),
    );
    let (first_swap_output, second_swap_output) =
        calculate_pools_output(&src_pool, &dst_pool, execute_amount);
    let sender = env.contract.address.to_string();
    dst_swap.input_amount = Some(Int128::from_be_bytes(
        first_swap_output.to_be_bytes()[16..32].try_into().unwrap(),
    ));

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
