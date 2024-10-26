use astromesh::{
    FISInput, FISInstruction, NexusAction,
};
use borsh::BorshDeserialize;
use time::{OffsetDateTime, Duration};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, to_json_vec, Binary, Deps, DepsMut, Env,
    Int128, MessageInfo, Response, StdError, StdResult, Uint64,
};
use drift::{
    create_deposit_usdt_ix, create_fill_order_jit_ix, create_fill_order_vamm_ix,
    create_initialize_user_ixs, create_place_order_ix, MarketType, OrderParams,
    OrderTriggerCondition, OrderType, PositionDirection, PostOnlyParam, DRIFT_PROGRAM_ID,
};
use std::collections::HashMap;
use std::vec::Vec;
use svm::{Pubkey, TransactionBuilder, AccountLink, Link};
mod astromesh;
mod drift;
mod svm;
mod test;

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

pub fn get_all_market_indexes(
    drift_program_id: Pubkey,
) -> StdResult<HashMap<String, u16>> {
    let mut market_indexes = HashMap::new();
    for idx in 0u16..4 {
        let (market, _) = Pubkey::find_program_address(
            &["perp_market".as_bytes(), idx.to_le_bytes().as_ref()],
            &drift_program_id,
        )
        .ok_or_else(|| StdError::generic_err("failed to find market PDA"))?;
        market_indexes.insert(market.to_string(), idx);
    }
    Ok(market_indexes)
}

pub fn is_in_auction_time(height: u64, order_creation_slot: u64, auction_period: u64) -> bool {
    if height < order_creation_slot + auction_period {
        return true;
    }
    false
}

pub fn place_perp_market_order(
    _deps: Deps,
    env: Env,
    market: String,
    usdt_amount: Int128,
    leverage: u8,
    auction_duration: u8,
    fis_input: &Vec<FISInput>,
) -> StdResult<Binary> {
    let mut instructions = vec![];

    let fis = &fis_input[0];
    let acc_link = from_json::<AccountLink>(fis.data.first().unwrap())?;
    let svm_pubkey = acc_link.link.svm_addr;
    let cosmos_signer = env.contract.address.to_string();
    
    let user_order_id = 1u8;

    let drift_program_id = Pubkey::from_string(&DRIFT_PROGRAM_ID.to_string())?;
    let (drift_state, _) =
        Pubkey::find_program_address(&["drift_state".as_bytes()], &drift_program_id)
            .ok_or_else(|| StdError::generic_err("failed to find drift state PDA"))?;

    // 1. initialize account
    let initialize_ix = create_initialize_user_ixs(svm_pubkey.clone(), drift_state.to_string())?;

    // 2. deposit usdt
    let deposit_amount: u64 = 1_000_000_000;

    let deposit_ix =
        create_deposit_usdt_ix(svm_pubkey.clone(), drift_state.to_string(), deposit_amount)?;

    // 3. place order
    let market_index = get_all_market_indexes(drift_program_id)?;

    let expire_time = (OffsetDateTime::now_utc() + Duration::minutes(1)).unix_timestamp();

    let asset_amount = usdt_amount.i128() as u64 * leverage as u64;

    let order_params = OrderParams {
        order_type: OrderType::Market,
        market_type: MarketType::Perp,
        direction: PositionDirection::Long,
        user_order_id: user_order_id,
        base_asset_amount: asset_amount,
        price: 1u64,
        market_index: market_index[&market],
        reduce_only: false,
        post_only: PostOnlyParam::None,
        immediate_or_cancel: false,
        max_ts: Some(expire_time),
        trigger_price: Some(0),
        trigger_condition: OrderTriggerCondition::Above,
        oracle_price_offset: Some(0),
        auction_duration: Some(auction_duration),
        auction_start_price: None,
        auction_end_price: None,
    };

    let place_order_ix =
        create_place_order_ix(svm_pubkey.clone(), drift_state.to_string(), order_params)?;

    let compute_budget = 5_000_000u64;

    let mut tx = TransactionBuilder::new();

    for idx in 0..initialize_ix.len() {
        tx.add_instruction(initialize_ix[idx].clone());
    }

    for idx in 0..deposit_ix.len() {
        tx.add_instruction(deposit_ix[idx].clone());
    }

    for idx in 0..place_order_ix.len() {
        tx.add_instruction(place_order_ix[idx].clone());
    }

    let msg = tx.build(vec![cosmos_signer], compute_budget.into());

    instructions.push(FISInstruction {
        plane: "COSMOS".to_string(),
        action: "COSMOS_INVOKE".to_string(),
        address: "".to_string(),
        msg: to_json_vec(&msg)?,
    });

    Ok(to_json_binary(&StrategyOutput { instructions })?)
}

pub fn fill_perp_market_order(
    deps: Deps,
    env: Env,
    taker_svm_address: String,
    taker_order_id: u32,
    percent: u8,
) -> StdResult<Binary> {
    // let mut instructions = vec![];

    // let height = env.block.height;
    // let cosmos_signer = env.contract.address.to_string();
    
    // // get from taker order id
    // let order_creation_slot = 0u64;
    // let total = 1000000000u64;
    // let market = String::from("BTC");

    // // generate order
    // let user_order_id = 0u8;

    // let drift_program_id = Pubkey::from_string(&DRIFT_PROGRAM_ID.to_string())?;
    // let (drift_state, _) =
    //     Pubkey::find_program_address(&["drift_state".as_bytes()], &drift_program_id)
    //         .ok_or_else(|| StdError::generic_err("failed to find drift state PDA"))?;

    // let market_index = get_all_market_indexes(deps, drift_program_id)?;
    // let expire_time = (Utc::now() + Duration::minutes(1)).timestamp();
    // let asset_amount = total * percent as u64;

    // let order_params = OrderParams {
    //     order_type: OrderType::Market,
    //     market_type: MarketType::Perp,
    //     direction: PositionDirection::Long,
    //     user_order_id: user_order_id,
    //     base_asset_amount: asset_amount,
    //     price: 0u64,
    //     market_index: market_index[&market],
    //     reduce_only: false,
    //     post_only: PostOnlyParam::None,
    //     immediate_or_cancel: false,
    //     max_ts: Some(expire_time),
    //     trigger_price: Some(0),
    //     trigger_condition: OrderTriggerCondition::Above,
    //     oracle_price_offset: Some(0),
    //     auction_duration: Some(0),
    //     auction_start_price: Some(asset_amount.try_into().unwrap()),
    //     auction_end_price: Some(asset_amount.try_into().unwrap()),
    // };

    // // 1. check if order is in auction time or not
    // if is_in_auction_time(
    //     height,
    //     order_creation_slot,
    //     order_params.max_ts.unwrap().try_into().unwrap(),
    // ) {
    //     // 2. if it's in auction time, use JIT
    //     let jit_ix = create_fill_order_jit_ix(
    //         svm_pubkey.clone(),
    //         drift_state.to_string(),
    //         order_params,
    //         taker_order_id,
    //     )?;

    //     let mut tx = TransactionBuilder::new();

    //     for idx in 0..jit_ix.len() {
    //         tx.add_instruction(jit_ix[idx].clone());
    //     }

    //     let msg = tx.build(vec![svm_pubkey], 5000000u64.into());

    //     instructions.push(FISInstruction {
    //         plane: "COSMOS".to_string(),
    //         action: "COSMOS_INVOKE".to_string(),
    //         address: "".to_string(),
    //         msg: to_json_vec(&msg)?,
    //     });
    // } else {
    //     // 3. otherwise, fill with vAMM
    //     let fill_ix = create_fill_order_vamm_ix(
    //         svm_pubkey.clone(),
    //         taker_svm_address.clone(),
    //         drift_state.to_string(),
    //         order_params,
    //     )?;

    //     let mut tx = TransactionBuilder::new();

    //     for idx in 0..fill_ix.len() {
    //         tx.add_instruction(fill_ix[idx].clone());
    //     }

    //     let msg = tx.build(vec![cosmos_signer], 5000000u64.into());

    //     instructions.push(FISInstruction {
    //         plane: "COSMOS".to_string(),
    //         action: "COSMOS_INVOKE".to_string(),
    //         address: "".to_string(),
    //         msg: to_json_vec(&msg)?,
    //     });
    // }

    Ok(Binary::new(vec![]))
    // Ok(to_json_binary(&StrategyOutput { instructions })?)
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let action = from_json::<NexusAction>(msg.msg)?;
    match action {
        NexusAction::PlacePerpMarketOrder {
            market,
            usdt_amount,
            leverage,
            auction_duration,
        } => place_perp_market_order(deps, env, market, usdt_amount, leverage, auction_duration, &msg.fis_input),
        NexusAction::FillPerpMarketOrder {
            taker_svm_address,
            taker_order_id,
            percent,
        } => fill_perp_market_order(deps, env, taker_svm_address, taker_order_id, percent),
    }
}
