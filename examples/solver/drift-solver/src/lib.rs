use astromesh::{FISInput, FISInstruction, MsgAstroTransfer, NexusAction};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, to_json_vec, Binary, Coin, Deps, DepsMut, Env, Int128,
    MessageInfo, Response, StdError, StdResult, Uint128,
};
use drift::{
    create_deposit_usdt_ix, create_fill_order_jit_ixs, create_fill_order_vamm_ix,
    create_initialize_user_ixs, create_place_order_ix, oracle_price_from_perp_market, MarketType,
    OrderParams, OrderTriggerCondition, OrderType, PositionDirection, PostOnlyParam, User,
    DRIFT_DEFAULT_PERCISION, PERP_MARKET_DISCRIMINATOR,
};
use std::{collections::HashMap, vec::Vec};
use svm::{Account, AccountLink, Pubkey, TransactionBuilder};
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

pub fn get_all_market_indexes(drift_program_id: Pubkey) -> StdResult<HashMap<String, u16>> {
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

pub fn is_in_auction_time(height: u64, order_creation_slot: u64, auction_period: u8) -> bool {
    if height < order_creation_slot + (auction_period as u64) {
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
        let init_account_ixs = create_initialize_user_ixs(svm_addr.clone())?;
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
    let (start_price, end_price) = (market_price * 998 / 1000, market_price);
    // base_asset_amount = usdt_amount * leverage / price

    let user_info = from_json::<Account>(user_info_bz)?;
    const USER_DISCRIMINATOR: &[u8] = &[159, 117, 95, 227, 239, 151, 58, 236];
    let user_info_bz = user_info.data;
    if !user_info_bz[..8].starts_with(USER_DISCRIMINATOR) {
        return Err(StdError::generic_err(format!(
            "invalid user discriminator, expected: {:?}",
            USER_DISCRIMINATOR
        )));
    }
    deps.api
        .debug(format!("user descriminator: {:?}", user_info_bz[..8].to_vec()).as_str());
    
    let user_info =
        borsh::from_slice::<User>(&user_info_bz[8..]).expect("must be parsed as drift::User");
    let user_order_id = user_info.next_order_id.try_into().unwrap();

    let order_params = OrderParams {
        order_type: OrderType::Market,
        market_type: MarketType::Perp,
        direction: PositionDirection::Long,
        user_order_id,
        base_asset_amount: quote_asset_amount * (leverage as u64) * DRIFT_DEFAULT_PERCISION
            / (market_price as u64),
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
    deps: Deps,
    env: Env,
    taker_svm: String,
    taker_order_id: u32,
    percent: u8,
    // fis[0]: cosmos: acc link
    // fis[1]: svm: accounts [maker_user, taker_user]
    fis_input: &Vec<FISInput>,
) -> StdResult<Binary> {
    let sender = env.contract.address.to_string();
    if percent <= 0 || percent > 100 {
        return Err(StdError::generic_err(format!("fill percent must be an integer within range 1..100, actual: {}", percent)))
    } 
    let sender_svm_link = from_json::<AccountLink>(fis_input.get(0).unwrap().data.get(0).unwrap())?; // sender svm
    let svm_addr = sender_svm_link.link.svm_addr;
    let taker_info_bz = fis_input.get(1).unwrap().data.get(1).unwrap();
    if taker_info_bz.eq(&"null".as_bytes()) {
        return Err(StdError::generic_err("taker subaccount is not initialized"));
    }

    let taker_info = from_json::<Account>(taker_info_bz)?;
    if taker_info.lamports.is_zero() {
        return Err(StdError::generic_err("taker subaccount is not initialized"));
    }

    let taker_info_bz = taker_info.data;
    const USER_DISCRIMINATOR: &[u8] = &[159, 117, 95, 227, 239, 151, 58, 236];
    if !taker_info_bz[..8].starts_with(USER_DISCRIMINATOR) {
        return Err(StdError::generic_err(format!(
            "invalid user discriminator, expected: {:?}",
            USER_DISCRIMINATOR
        )));
    }

    let sender_info = fis_input.get(1).unwrap().data.get(0).unwrap();
    let mut fis_instructions = vec![];
    let mut tx_builder = TransactionBuilder::new();

    // if subaccount is not created, create it
    if sender_info.starts_with("null".as_bytes()) {
        let initialize_ixs = create_initialize_user_ixs(svm_addr.clone())?;
        tx_builder.add_instructions(initialize_ixs);
    }

    deps.api
        .debug(format!("user descriminator: {:?}", taker_info_bz[..8].to_vec()).as_str());
    let taker_info =
        borsh::from_slice::<User>(&taker_info_bz[8..]).expect("must be parsed as drift::User");
    let order = taker_info
        .orders
        .iter()
        .find(|x| x.order_id == taker_order_id)
        .expect(format!("taker order id {} must exist", taker_order_id).as_str());

    // fallback: fill against vAMM
    if !is_in_auction_time(env.block.height, order.slot, order.auction_duration) {
        let fill_vamm = create_fill_order_vamm_ix(svm_addr, taker_svm, taker_order_id)?;

        tx_builder.add_instructions(fill_vamm);
        let msg = tx_builder.build(vec![sender.clone()], 10_000_000);

        let instruction = FISInstruction {
            plane: "SVM".to_string(),
            action: "VM_INVOKE".to_string(),
            address: "".to_string(),
            msg: to_json_vec(&msg)?,
        };
        return to_json_binary(&StrategyOutput {
            instructions: vec![instruction],
        });
    }
    let amount_to_fill =
        (order.base_asset_amount - order.base_asset_amount_filled) * (percent as u64) / 100;
    let usdt_to_deposit = amount_to_fill * order.price / DRIFT_DEFAULT_PERCISION;
    let deposit_ixs = create_deposit_usdt_ix(deps, svm_addr.clone(), usdt_to_deposit)?;
    tx_builder.add_instructions(deposit_ixs);

    let direction = match order.direction {
        PositionDirection::Long => PositionDirection::Short,
        PositionDirection::Short => PositionDirection::Long,
    };

    let order_params = OrderParams {
        order_type: OrderType::Limit,
        market_type: MarketType::Perp,
        direction,
        user_order_id: 0,
        base_asset_amount: amount_to_fill,
        price: order.auction_start_price as u64,
        market_index: order.market_index,
        reduce_only: false,
        post_only: PostOnlyParam::MustPostOnly,
        immediate_or_cancel: true,
        max_ts: Some(env.block.time.plus_minutes(5).seconds() as i64),
        trigger_price: Some(0),
        trigger_condition: OrderTriggerCondition::Above, // TriggerCondition is 0, which we'll assume is None
        oracle_price_offset: Some(0),
        auction_duration: Some(100),
        auction_start_price: None,
        auction_end_price: None,
    };

    let fill_jit_ixs =
        create_fill_order_jit_ixs(svm_addr, order_params, taker_svm, taker_order_id)?;

    tx_builder.add_instructions(fill_jit_ixs);
    tx_builder.build(vec![sender.clone()], 10_000_000);

    // transfer usdt from cosmos > svm
    let cosmos_transfer = MsgAstroTransfer::new(
        sender.clone(),
        sender,
        "COSMOS".to_string(),
        "SVM".to_string(),
        Coin {
            denom: "usdt".to_string(),
            amount: Uint128::from(usdt_to_deposit),
        },
    );

    fis_instructions.push(FISInstruction {
        plane: "COSMOS".to_string(),
        action: "COSMOS_INVOKE".to_string(),
        address: "".to_string(),
        msg: to_json_vec(&cosmos_transfer)?,
    });

    // do drift instructions
    let msg = tx_builder.build(vec![env.contract.address.to_string()], 10_000_000);
    let instruction = FISInstruction {
        plane: "SVM".to_string(),
        action: "VM_INVOKE".to_string(),
        address: "".to_string(),
        msg: to_json_vec(&msg)?,
    };
    fis_instructions.push(instruction);

    return to_json_binary(&StrategyOutput {
        instructions: fis_instructions,
    });
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
        } => fill_perp_market_order(
            deps,
            env,
            taker_svm_address,
            taker_order_id,
            percent,
            &msg.fis_input,
        ),
    }
}
