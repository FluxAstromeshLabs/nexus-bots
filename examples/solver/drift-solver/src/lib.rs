use astromesh::{FISInput, FISInstruction, MsgAstroTransfer, NexusAction};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, to_json_vec, Binary, Coin, Deps, DepsMut, Env, Int128,
    MessageInfo, Response, StdError, StdResult, Uint64,
};
use drift::{
    create_deposit_usdt_ix, create_fill_order_jit_ix, create_fill_order_vamm_ix, create_initialize_user_ixs, create_place_order_ix, oracle_price_from_perp_market, MarketType, OrderParams, OrderTriggerCondition, OrderType, PositionDirection, PostOnlyParam, DRIFT_DEFAULT_PERCISION, DRIFT_PROGRAM_ID, ORACLE_BTC, PERP_MARKET_DISCRIMINATOR
};
use std::collections::HashMap;
use std::vec::Vec;
use svm::{Account, AccountLink, Link, Pubkey, TransactionBuilder};
mod astromesh;
mod drift;
mod svm;

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

pub fn is_in_auction_time(height: u64, order_creation_slot: u64, auction_period: u64) -> bool {
    if height < order_creation_slot + auction_period {
        return true;
    }
    false
}

pub fn astro_transfer(cosmos_addr: String, amount: u64) -> Vec<FISInstruction> {
    let mut instructions = vec![];

    let msg = MsgAstroTransfer::new(
        cosmos_addr.clone(),
        cosmos_addr.clone(),
        "COSMOS".to_string(),
        "SVM".to_string(),
        Coin {
            denom: "usdt".to_string(),
            amount: amount.into(),
        },
    );

    instructions.push(FISInstruction {
        plane: "COSMOS".to_string(),
        action: "COSMOS_INVOKE".to_string(),
        address: "".to_string(),
        msg: to_json_vec(&msg).unwrap(),
    });

    instructions
}

pub fn place_perp_market_order(
    deps: Deps,
    env: Env,
    market: String,
    usdt_amount: Int128,
    leverage: u8,
    auction_duration: u8,
    // fis[0]: cosmos: acc link
    // fis[1]: svm: accounts [user, market 0, market 1, market 2]
    fis_input: &Vec<FISInput>,
) -> StdResult<Binary> {
    let mut instructions = vec![];
    // parse fis
    let fis = &fis_input[0];
    let acc_link = from_json::<AccountLink>(fis.data.first().unwrap())?;
    let svm_addr = acc_link.link.svm_addr;
    let user_info_bz = fis_input[1]
        .data
        .get(0)
        .ok_or_else(|| StdError::generic_err("user info must exist"))?;

    let market_index: u16 = match market.as_str() {
        "btc-usdt" => 0,
        "eth-usdt" => 1,
        "sol-usdt" => 2,
        unknown_market => {
            return Err(StdError::generic_err(format!(
                "market '{}' is not supported",
                unknown_market
            )))
        }
    };
    let market_bz = fis_input[1]
        .data
        .get((market_index as usize) + 1)
        .ok_or_else(|| StdError::generic_err("requested market must exist"))?;
    let market_account = from_json::<Account>(market_bz)?;
    if !market_account.data[..8].starts_with(PERP_MARKET_DISCRIMINATOR) {
        return Err(StdError::generic_err(format!(
            "market account data must begin with {:?}",
            PERP_MARKET_DISCRIMINATOR
        )));
    }

    // compose instructions
    // 1. create accounts if not exist
    let mut tx = TransactionBuilder::new();
    if user_info_bz.eq(&"null".as_bytes()) {
        let init_account_ixs = create_initialize_user_ixs(deps, svm_addr.clone())?;
        tx.add_instructions(init_account_ixs);
    };

    // 2. deposit usdt
    let quote_asset_amount = usdt_amount.i128() as u64;
    let cosmos_addr = env.contract.address.to_string();
    let astro_transfer_ix = astro_transfer(cosmos_addr.clone(), 1_000_000_000);
    instructions.extend(astro_transfer_ix);

    let deposit_ixs = create_deposit_usdt_ix(deps, svm_addr.clone(), quote_asset_amount)?;
    tx.add_instructions(deposit_ixs);

    // 3. place order
    let market_price = oracle_price_from_perp_market(&market_account.data)?;
    let expire_time = env.block.time.seconds() as i64 + 30;
    let (start_price, end_price) = (market_price*998/1000, market_price);
    // base_asset_amount = usdt_amount * leverage / price
    let user_order_id = 1;
    let order_params = OrderParams {
        order_type: OrderType::Market,
        market_type: MarketType::Perp,
        direction: PositionDirection::Long,
        user_order_id,
        base_asset_amount: quote_asset_amount * (leverage as u64) * DRIFT_DEFAULT_PERCISION / (market_price as u64),
        price: market_price as u64, // oralce price
        market_index,
        reduce_only: false,
        post_only: PostOnlyParam::None,
        immediate_or_cancel: false,
        max_ts: Some(expire_time),
        trigger_price: Some(0),
        trigger_condition: OrderTriggerCondition::Above,
        oracle_price_offset: Some(0),
        auction_duration: Some(auction_duration),
        auction_start_price: Some(start_price),
        auction_end_price: Some(end_price),
    };

    let place_order_ixs = create_place_order_ix(svm_addr.clone(), order_params)?;
    let compute_budget = 5_000_000u64;
    tx.add_instructions(place_order_ixs);

    let msg = tx.build(vec![cosmos_addr], compute_budget.into());
    deps.api.debug(&format!("msg {:?}", msg));

    instructions.push(FISInstruction {
        plane: "SVM".to_string(),
        action: "VM_INVOKE".to_string(),
        address: "".to_string(),
        msg: to_json_vec(&msg)?,
    });

    Ok(to_json_binary(&StrategyOutput { instructions })?)
}

pub fn fill_perp_market_order(
    _deps: Deps,
    _env: Env,
    _taker_svm_address: String,
    _taker_order_id: u32,
    _percent: u8,
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

    // let market_index: u16;
    // match market.as_str() {
    //     "btc" => market_index = 0,
    //     "eth" => market_index = 1,
    //     "sol" => market_index = 2,
    //     default => {
    //         return Err(StdError::generic_err(format!(
    //             "market {} is not supported",
    //             default
    //         )))
    //     }
    // }
    // let expire_time = ;
    // let asset_amount = total * percent as u64;

    // let order_params = OrderParams {
    //     order_type: OrderType::Market,
    //     market_type: MarketType::Perp,
    //     direction: PositionDirection::Long,
    //     user_order_id: user_order_id,
    //     base_asset_amount: asset_amount,
    //     price: 0u64,
    //     market_index: market_index,
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
    //         svm_addr.clone(),
    //         drift_state.to_string(),
    //         order_params,
    //         taker_order_id,
    //     )?;

    //     let mut tx = TransactionBuilder::new();

    //     for idx in 0..jit_ix.len() {
    //         tx.add_instruction(jit_ix[idx].clone());
    //     }

    //     let msg = tx.build(vec![svm_addr], 5000000u64.into());

    //     instructions.push(FISInstruction {
    //         plane: "COSMOS".to_string(),
    //         action: "COSMOS_INVOKE".to_string(),
    //         address: "".to_string(),
    //         msg: to_json_vec(&msg)?,
    //     });
    // } else {
    //     // 3. otherwise, fill with vAMM
    //     let fill_ix = create_fill_order_vamm_ix(
    //         svm_addr.clone(),
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
            usdt_amount,
            leverage,
            market,
            auction_duration,
        } => place_perp_market_order(
            deps,
            env,
            market,
            usdt_amount,
            leverage,
            auction_duration,
            &msg.fis_input,
        ),
        NexusAction::FillPerpMarketOrder {
            taker_svm_address,
            taker_order_id,
            percent,
        } => fill_perp_market_order(deps, env, taker_svm_address, taker_order_id, percent),
    }
}
