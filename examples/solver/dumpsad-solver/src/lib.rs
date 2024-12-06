use astromesh::{
    keccak256, sha256, AccountResponse, CommissionConfig, FISInput, FISInstruction, InitialMint, MsgAstroTransfer, MsgCreateBankDenom, MsgCreatePool, MsgUpdatePool, NexusAction, PLANE_COSMOS, QUERY_ACTION_COSMOS_BANK_BALANCE, QUERY_ACTION_COSMOS_KVSTORE, QUERY_ACTION_COSMOS_QUERY
};
use bech32::{Bech32, Bech32m, Hrp};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, to_json_vec, BankMsg, Binary, Coin, Decimal,
    DenomMetadata, DenomUnit, Deps, DepsMut, Env, HexBinary, Int128, MessageInfo, Response,
    StdError, StdResult, Uint128, Uint64,
};
use curve::BondingCurve;
use strategy::{FISQueryInstruction, FISQueryRequest, MsgConfigStrategy, PermissionConfig, StrategyMetadata};
use std::vec::Vec;
mod astromesh;
mod curve;
mod svm;
mod test;
mod strategy;

const PERCENTAGE_BPS: u128 = 10_000;
const EMBEDDED_CRON_BINARY: &[u8] = include_bytes!("../../../cron/dumpsad-cron/target/wasm32-unknown-unknown/release/dumpsad_cron.wasm");

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

fn handle_create_token(
    deps: Deps,
    env: Env,
    name: String,
    description: String,
    uri: String,
    target_vm: String,
    bot_id: String,
    fis_input: &Vec<FISInput>,
) -> StdResult<Vec<FISInstruction>> {
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
    deps.api.debug(
        format!(
            "pool id seed: {}",
            HexBinary::from(pool_id_seed.as_slice()).to_string()
        )
        .as_str(),
    );

    let pool_id = &keccak256(&pool_id_seed)[12..];
    // TODO: Check cosmwasm std Addr, it needs callback/FFI
    let pool_address = bech32::encode::<Bech32>(Hrp::parse("lux").unwrap(), pool_id)
        .map_err(|e| StdError::generic_err(e.to_string()))?;

    let initial_amount = Uint128::new(1_000_000_000);
    let denom_base = format!("astromesh/{}/{}", creator.clone(), name);
    let denom_display = name.to_uppercase();
    let denom_symbol = name.to_uppercase();

    // create pool
    let create_pool_msg = MsgCreatePool::new(
        creator.clone(),
        Some(CommissionConfig::new(0i64, 0i64, 0i64)),
    );
    // update the pool input the target vm
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
                    denom: denom_display.clone(),
                    exponent: 6,
                    aliases: vec![],
                },
            ],
            base: denom_base.clone(),
            display: denom_display.clone(),
            name: name.clone(),
            symbol: denom_symbol.clone(),
            uri: uri.clone(),
            uri_hash: "".to_string(),
        },
        "".to_string(), // only do initial mint, cannot mint more
        vec![InitialMint {
            address: pool_address.clone(),
            amount: initial_amount,
        }],
    );

    // TODO: Use static sum to save some gas
    let cron_binary_checksum = sha256(EMBEDDED_CRON_BINARY);
    let cron_id = keccak256(&[creator_bz.as_slice(), cron_binary_checksum.as_slice(), &(acc_info.account.sequence.u64()+1).to_le_bytes()].concat());
    let create_graduate_cron_msg = MsgConfigStrategy::new(
        creator.clone(), 
        strategy::Config::Deploy, 
        "".to_string(), 
        EMBEDDED_CRON_BINARY.to_vec(), 
        Some(FISQueryRequest::new(vec![
            FISQueryInstruction::new(
                PLANE_COSMOS.to_string(),
                QUERY_ACTION_COSMOS_BANK_BALANCE.to_string(), 
                vec![], 
                vec![
                    format!("{},{}", pool_address, pool_address).as_bytes().to_vec(),
                    format!("sol,{}", denom_base).as_bytes().to_vec(),
                ],
            ),
            FISQueryInstruction::new(
                PLANE_COSMOS.to_string(),
                QUERY_ACTION_COSMOS_KVSTORE.to_string(), 
                vec![], 
                vec![
                    "wasmd".as_bytes().to_vec(),
                    [&[4u8], "lastContractId".as_bytes()].concat().to_vec(),
                ],
            ),
        ])), 
        Some(PermissionConfig::new("anyone".to_string(), vec![])),
        Some(StrategyMetadata {
            name: "graduate cron".to_string(),
            logo: "".to_string(),
            description: "graduate meme coin".to_string(),
            website: "".to_string(),
            ty: "CRON".to_string(),
            tags: vec![],
            schema: "{}".to_string(),
            cron_gas_price: Uint128::from(500_000_000u128),
            aggregated_query_keys:vec![],
            cron_input: format!(r#"{{"vm":"{}","pool_address":"{}"}}"#, target_vm, pool_address),
            cron_interval: 2,
            supported_apps: vec![],
        }),
    );

    deps.api.debug(
        format!(
            "pool id to update: {}",
            HexBinary::from(pool_id).to_string()
        )
        .as_str(),
    );
    let update_pool_msg = MsgUpdatePool::new(
        creator.clone(),
        HexBinary::from(pool_id).to_string(),
        target_vm.as_bytes().to_vec(),
        vec![],
        false,
        vec![],
        "".to_string(),
        vec![bot_id, HexBinary::from(cron_id).to_string()]
    );

    Ok(vec![
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
            msg: to_json_vec(&create_graduate_cron_msg)?, // cron
        },
        FISInstruction {
            plane: "COSMOS".to_string(),
            action: "COSMOS_INVOKE".to_string(),
            address: "".to_string(),
            msg: to_json_vec(&create_denom_msg)?,
        },
        FISInstruction {
            plane: "COSMOS".to_string(),
            action: "COSMOS_INVOKE".to_string(),
            address: "".to_string(),
            msg: to_json_vec(&update_pool_msg)?,
        },
    ])
}

fn handle_buy(
    _deps: Deps,
    env: Env,
    denom: String,
    amount: Uint128,
    slippage: Uint128,
    pool_address: String, // TODO: where can frontend get this pool address?
    fis_input: &Vec<FISInput>,
) -> StdResult<Vec<FISInstruction>> {
    let trader = env.contract.address.clone();
    // load quote amount
    let quote_coin = from_json::<Coin>(fis_input.get(0).unwrap().data.get(0).unwrap())?;
    let meme_coin = from_json::<Coin>(fis_input.get(0).unwrap().data.get(1).unwrap())?;

    // calculate the delta Y
    let mut curve = BondingCurve::default(
        quote_coin.amount,
        Uint128::from(1_000_000_000u128) - meme_coin.amount,
    );
    let pre_price = curve.price();
    let worst_price = pre_price
        .checked_mul(slippage.checked_add(Uint128::new(PERCENTAGE_BPS))?)?
        .checked_div(Uint128::new(PERCENTAGE_BPS))?;

    let bought_amount = curve.buy(amount);
    let post_price = curve.price();
    assert!(
        post_price.lt(&worst_price),
        "slippage exceeds, pre price: {}, worst price: {}, post price: {}",
        pre_price,
        worst_price,
        post_price
    );

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
                denom: quote_coin.denom,
                amount,
            },
        ))?,
    };

    // send meme to trader
    let pool_send_meme = FISInstruction {
        plane: PLANE_COSMOS.to_string(),
        action: "COSMOS_INVOKE".to_string(),
        address: "".to_string(),
        msg: to_json_vec(&MsgAstroTransfer::new(
            pool_address,
            trader.to_string(),
            PLANE_COSMOS.to_string(),
            PLANE_COSMOS.to_string(),
            Coin {
                denom: meme_coin.denom,
                amount: bought_amount,
            },
        ))?,
    };

    Ok(vec![trader_send_quote, pool_send_meme])
}

fn handle_sell(
    deps: Deps,
    env: Env,
    denom: String,
    amount: Uint128,
    slippage: Uint128,
    pool_address: String, // TODO: where can frontend get this pool address?
    fis_input: &Vec<FISInput>,
) -> StdResult<Vec<FISInstruction>> {
    let trader = env.contract.address.clone();
    // Load quote and meme amounts from input
    let quote_coin = from_json::<Coin>(fis_input.get(0).unwrap().data.get(0).unwrap())?;
    let meme_coin = from_json::<Coin>(fis_input.get(0).unwrap().data.get(1).unwrap())?;

    // Initialize bonding curve
    let mut curve = BondingCurve::default(
        quote_coin.amount,
        Uint128::from(1_000_000_000u128) - meme_coin.amount,
    );
    let pre_price = curve.price();
    let worst_price = pre_price
        .checked_mul(slippage.checked_add(Uint128::new(PERCENTAGE_BPS))?)?
        .checked_div(Uint128::new(PERCENTAGE_BPS))?;

    // Calculate sold amount and verify slippage
    let sold_amount = curve.sell(amount);
    let post_price = curve.price();
    assert!(
        post_price >= worst_price,
        "slippage exceeds, pre price: {}, post price: {}",
        pre_price,
        post_price
    );

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
                denom: meme_coin.denom,
                amount,
            },
        ))?,
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
            Coin {
                denom: quote_coin.denom,
                amount: sold_amount,
            },
        ))?,
    };

    Ok(vec![trader_send_meme, pool_send_quote])
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let nexus_action: NexusAction = from_json(&msg.msg)?;
    let instructions = match nexus_action {
        NexusAction::CreateToken {
            name,
            description,
            uri,
            target_vm,
            bot_id,
        } => handle_create_token(deps, env, name, description, uri, target_vm, bot_id, &msg.fis_input),
        NexusAction::Buy {
            denom,
            amount,
            slippage,
            pool_address,
        } => handle_buy(
            deps,
            env,
            denom,
            amount,
            slippage,
            pool_address,
            &msg.fis_input,
        ),
        NexusAction::Sell {
            denom,
            amount,
            slippage,
            pool_address,
        } => handle_sell(
            deps,
            env,
            denom,
            amount,
            slippage,
            pool_address,
            &msg.fis_input,
        ),
    }?;

    Ok(to_json_binary(&StrategyOutput { instructions }).unwrap())
}
