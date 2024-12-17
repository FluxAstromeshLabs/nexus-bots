use astromesh::{
    keccak256, sha256, AccountResponse, FISInput, FISInstruction, InitialMint, MsgAstroTransfer,
    MsgCreateBankDenom, NexusAction, PLANE_COSMOS, QUERY_ACTION_COSMOS_BANK_BALANCE,
    QUERY_ACTION_COSMOS_KVSTORE, QUERY_ACTION_COSMOS_QUERY,
};
use bech32::{Bech32, Bech32m, Hrp};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, to_json_string, to_json_vec, BankMsg, Binary, Coin,
    Decimal, DenomMetadata, DenomUnit, Deps, DepsMut, Env, HexBinary, Int128, MessageInfo,
    Response, StdError, StdResult, Uint128, Uint64,
};
use curve::BondingCurve;
use events::{CreateTokenEvent, GraduateEvent, TradeTokenEvent};
use interpool::{CommissionConfig, MsgCreatePool, MsgUpdatePool, QueryPoolResponse};
use std::vec::Vec;
mod astromesh;
mod curve;
mod events;
mod interpool;
mod strategy;
mod svm;
mod test;

const PERCENTAGE_BPS: u128 = 10_000;
const DEFAULT_QUOTE_DENOM: &str = "sol";
const INITIAL_AMOUNT: &Uint128 = &Uint128::new(1_000_000_000_000_000_000);
const MARKET_CAP_TO_GRADUATE: &Uint128 = &Uint128::new(20_000_000_000);

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
pub struct StrategyEvent {
    pub topic: String,
    pub data: Binary,
}

#[cw_serde]
pub struct StrategyOutput {
    instructions: Vec<FISInstruction>,
    events: Vec<StrategyEvent>,
    result: String,
}

fn handle_create_token(
    deps: Deps,
    env: Env,
    name: String,
    symbol: String,
    description: String,
    uri: String,
    target_vm: String,
    solver_id: String,
    cron_id: String,
    fis_input: &Vec<FISInput>,
) -> StdResult<StrategyOutput> {
    let creator = env.contract.address.to_string();
    let acc_info = from_json::<AccountResponse>(fis_input.get(0).unwrap().data.get(0).unwrap())?;
    let (_, creator_bz) =
        bech32::decode(&creator).map_err(|e| StdError::generic_err(e.to_string()))?;
    let pool_id_seed = &[
        "pool".as_bytes(),
        creator_bz.as_slice(),
        (acc_info.account.sequence.u64() + 1)
            .to_le_bytes()
            .as_slice(),
    ]
    .concat();

    let pool_id = &keccak256(&pool_id_seed)[12..];
    // TODO: Check cosmwasm std Addr, it needs callback/FFI
    let pool_address = bech32::encode::<Bech32>(Hrp::parse("lux").unwrap(), pool_id)
        .map_err(|e| StdError::generic_err(e.to_string()))?;

    let denom_base = format!("astromesh/{}/{}", creator.clone(), symbol);
    let denom_symbol = symbol;

    // create pool
    let create_pool_msg = MsgCreatePool::new(
        creator.clone(),
        Some(CommissionConfig::new(0i64, 0i64, 0i64)),
    );

    // create denom and mint for the pool
    let create_denom_msg = MsgCreateBankDenom::new(
        creator.clone(),
        DenomMetadata {
            description: description.clone(),
            denom_units: vec![
                DenomUnit {
                    denom: denom_base.clone(),
                    exponent: 0,
                    aliases: vec![name.clone()],
                },
                DenomUnit {
                    denom: denom_symbol.clone(),
                    exponent: 9,
                    aliases: vec![],
                },
            ],
            base: denom_base.clone(),
            display: denom_symbol.clone(),
            name: name.clone(),
            symbol: denom_symbol.clone(),
            uri: uri.clone(),
            uri_hash: "".to_string(),
        },
        "".to_string(), // only do initial mint, cannot mint more
        vec![InitialMint {
            address: pool_address.clone(),
            amount: INITIAL_AMOUNT.clone(),
        }],
    );

    let update_pool_msg = MsgUpdatePool::new(
        creator.clone(),
        HexBinary::from(pool_id).to_string(),
        target_vm.as_bytes().to_vec(),
        vec![],
        false,
        vec![],
        cron_id.to_string(),
        solver_id.clone(),
    );

    Ok(StrategyOutput {
        instructions: vec![
            FISInstruction {
                plane: "COSMOS".to_string(),
                action: "COSMOS_INVOKE".to_string(),
                address: "".to_string(),
                msg: to_json_vec(&create_pool_msg)?,
            },
            FISInstruction {
                plane: "COSMOS".to_string(),
                action: "COSMOS_INVOKE".to_string(),
                address: "".to_string(),
                msg: to_json_vec(&update_pool_msg)?,
            },
            FISInstruction {
                plane: "COSMOS".to_string(),
                action: "COSMOS_INVOKE".to_string(),
                address: "".to_string(),
                msg: to_json_vec(&create_denom_msg)?,
            },
        ],
        events: vec![StrategyEvent {
            topic: "create_token".to_string(),
            data: to_json_binary(&CreateTokenEvent {
                denom: denom_base.clone(),
                name: name.clone(),
                symbol: denom_symbol.clone(),
                description: description.clone(),
                pool_id: HexBinary::from(pool_id).to_string(),
                vm: target_vm.clone(),
                logo: uri.clone(),
                cron_id,
                solver_id,
            })?,
        }],
        result: "Token created".to_string(),
    })
}

// for now, brute-force to find target coins in pool
// better if use map / kv query
fn get_pool_sol_meme_amounts(
    pool_inventory: &Vec<Coin>,
    meme_denom: &String,
) -> StdResult<(Uint128, Uint128)> {
    let sol_coin = pool_inventory
        .into_iter()
        .find(|c| c.denom == DEFAULT_QUOTE_DENOM)
        .map(|c| c.amount)
        .or(Some(Uint128::zero()))
        .unwrap();

    let meme_coin = pool_inventory
        .into_iter()
        .find(|c| &c.denom == meme_denom)
        .ok_or_else(|| StdError::generic_err(format!("denom {} not found", meme_denom)))?
        .amount;
    Ok((sol_coin, meme_coin))
}

fn handle_buy(
    _deps: Deps,
    env: Env,
    meme_denom: String,
    amount: Uint128,
    slippage: Uint128,
    fis_input: &Vec<FISInput>,
) -> StdResult<StrategyOutput> {
    assert!(amount.gt(&Uint128::zero()), "amount must be positive");

    let trader = env.contract.address.clone();
    let pool_res = from_json::<QueryPoolResponse>(fis_input.get(0).unwrap().data.get(0).unwrap())?;
    let (sol_amount, meme_amount) =
        get_pool_sol_meme_amounts(&pool_res.pool.inventory_snapshot, &meme_denom)?;
    assert!(
        !meme_amount.is_zero(),
        "cannot trade, the curve is graduated"
    );

    // calculate the delta Y
    let mut curve = BondingCurve::default(sol_amount, INITIAL_AMOUNT - meme_amount);
    let current_price = curve.price();
    let worst_amount = amount
        * BondingCurve::PRECISION_MULTIPLIER
        * Uint128::new(PERCENTAGE_BPS - slippage.u128())
        / current_price
        / Uint128::new(PERCENTAGE_BPS);

    let received_amount = curve.buy(amount);
    assert!(received_amount.gt(&Uint128::zero()), "cannot buy 0 amount");
    assert!(
        !received_amount.lt(&worst_amount),
        "slippage exceeds. worst amount: {}, actual amount: {}",
        worst_amount,
        received_amount
    );
    let post_price = curve.price();
    let pool_id_bz = HexBinary::from_hex(&pool_res.pool.pool_id)?;
    let pool_address = bech32::encode::<Bech32>(Hrp::parse("lux").unwrap(), pool_id_bz.as_slice())
        .map_err(|e| StdError::generic_err(e.to_string()))?;

    // send quote to vault
    let trader_send_quote = FISInstruction {
        plane: PLANE_COSMOS.to_string(),
        action: "COSMOS_INVOKE".to_string(),
        address: "".to_string(),
        msg: to_json_vec(&MsgAstroTransfer::new(
            trader.to_string(),
            pool_address.clone(),
            PLANE_COSMOS.to_string(),
            PLANE_COSMOS.to_string(),
            Coin {
                denom: DEFAULT_QUOTE_DENOM.to_string(),
                amount,
            },
        ))?,
    };

    // send meme to trader
    let received_coin = Coin {
        denom: meme_denom.clone(),
        amount: received_amount,
    };
    let pool_send_meme = FISInstruction {
        plane: PLANE_COSMOS.to_string(),
        action: "COSMOS_INVOKE".to_string(),
        address: "".to_string(),
        msg: to_json_vec(&MsgAstroTransfer::new(
            pool_address.clone(),
            trader.to_string(),
            PLANE_COSMOS.to_string(),
            PLANE_COSMOS.to_string(),
            received_coin.clone(),
        ))?,
    };

    let mut instructions = vec![trader_send_quote, pool_send_meme];
    let mut events = vec![StrategyEvent {
        topic: "buy_token".to_string(),
        data: to_json_binary(&TradeTokenEvent {
            denom: meme_denom.clone(),
            price: post_price,
            trader: trader.to_string(),
            amount: received_amount,
        })?,
    }];

    let is_graduate = (post_price * INITIAL_AMOUNT / BondingCurve::PRECISION_MULTIPLIER).ge(MARKET_CAP_TO_GRADUATE);
    if is_graduate {
        let update_pool_msg = MsgUpdatePool::new(
            pool_address.clone(),
            pool_res.pool.pool_id,
            vec![],
            "1".as_bytes().to_vec(),
            false,
            vec![],
            "".to_string(),
            "".to_string(),
        );

        instructions.push(FISInstruction {
            plane: "COSMOS".to_string(),
            action: "COSMOS_INVOKE".to_string(),
            address: "".to_string(),
            msg: to_json_vec(&update_pool_msg)?,
        });
        
        events.push(StrategyEvent {
            topic: "graduate".to_string(),
            data: to_json_binary(&GraduateEvent {
                price: post_price,
                pool_address: pool_address.clone(),
                meme_denom: meme_denom.clone(),
                meme_amount: meme_amount - received_amount,
                sol_amount: sol_amount + amount,
                vm: pool_res.pool.input_blob.unwrap().to_string(),
            })?,
        });
    }

    Ok(StrategyOutput {
        instructions,
        events,
        result: to_json_string(&received_coin)?,
    })
}

fn handle_sell(
    deps: Deps,
    env: Env,
    meme_denom: String,
    amount: Uint128,
    slippage: Uint128,
    fis_input: &Vec<FISInput>,
) -> StdResult<StrategyOutput> {
    assert!(amount.gt(&Uint128::zero()), "amount must be positive");

    // Load quote and meme amounts from input
    let trader = env.contract.address.clone();
    let pool_res = from_json::<QueryPoolResponse>(fis_input.get(0).unwrap().data.get(0).unwrap())?;
    let (sol_amount, meme_amount) =
        get_pool_sol_meme_amounts(&pool_res.pool.inventory_snapshot, &meme_denom)?;
    assert!(
        !meme_amount.is_zero(),
        "cannot trade, the curve is graduated"
    );

    // Initialize bonding curve
    let mut curve = BondingCurve::default(sol_amount, INITIAL_AMOUNT - meme_amount);

    // Calculate receive amount and verify slippage
    let current_price = curve.price();
    let received_amount = curve.sell(amount);
    let worst_amount = amount * current_price * Uint128::new(PERCENTAGE_BPS - slippage.u128())
        / Uint128::new(PERCENTAGE_BPS)
        / BondingCurve::PRECISION_MULTIPLIER;
    assert!(
        received_amount.gt(&Uint128::zero()),
        "receive zero sol, try larger meme amount"
    );
    assert!(
        !received_amount.lt(&worst_amount),
        "slippage exceeds. worst amount: {}, actual amount: {}",
        worst_amount,
        received_amount
    );
    let post_price = curve.price();

    let pool_id_bz = HexBinary::from_hex(&pool_res.pool.pool_id)?;
    let pool_address = bech32::encode::<Bech32>(Hrp::parse("lux").unwrap(), pool_id_bz.as_slice())
        .map_err(|e| StdError::generic_err(e.to_string()))?;

    // Transfer instructions
    let trader_send_meme = FISInstruction {
        plane: PLANE_COSMOS.to_string(),
        action: "COSMOS_INVOKE".to_string(),
        address: "".to_string(),
        msg: to_json_vec(&MsgAstroTransfer::new(
            trader.to_string(),
            pool_address.clone(),
            PLANE_COSMOS.to_string(),
            PLANE_COSMOS.to_string(),
            Coin {
                denom: meme_denom.clone(),
                amount,
            },
        ))?,
    };

    let received_coin = Coin {
        denom: DEFAULT_QUOTE_DENOM.to_string(),
        amount: received_amount,
    };
    let pool_send_quote = FISInstruction {
        plane: PLANE_COSMOS.to_string(),
        action: "COSMOS_INVOKE".to_string(),
        address: "".to_string(),
        msg: to_json_vec(&MsgAstroTransfer::new(
            pool_address,
            trader.to_string(),
            PLANE_COSMOS.to_string(),
            PLANE_COSMOS.to_string(),
            received_coin.clone(),
        ))?,
    };

    Ok(StrategyOutput {
        instructions: vec![trader_send_meme, pool_send_quote],
        events: vec![StrategyEvent {
            topic: "sell_token".to_string(),
            data: to_json_binary(&TradeTokenEvent {
                denom: meme_denom,
                price: post_price,
                trader: trader.to_string(),
                amount: received_amount,
            })?,
        }],
        result: to_json_string(&received_coin)?,
    })
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let nexus_action: NexusAction = from_json(&msg.msg)?;
    let output = match nexus_action {
        NexusAction::CreateToken {
            name,
            symbol,
            description,
            uri,
            target_vm,
            solver_id,
            cron_id,
        } => handle_create_token(
            deps,
            env,
            name,
            symbol,
            description,
            uri,
            target_vm,
            solver_id,
            cron_id,
            &msg.fis_input,
        ),
        NexusAction::Buy {
            denom,
            amount,
            slippage,
        } => handle_buy(deps, env, denom, amount, slippage, &msg.fis_input),
        NexusAction::Sell {
            denom,
            amount,
            slippage,
        } => handle_sell(deps, env, denom, amount, slippage, &msg.fis_input),
    }?;

    Ok(to_json_binary(&output).unwrap())
}
