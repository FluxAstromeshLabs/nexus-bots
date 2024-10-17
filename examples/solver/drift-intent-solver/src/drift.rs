use crate::svm::Instruction;
pub const DRIFT_PROGRAM_ID: &str = "FLR3mfYrMZUnhqEadNJVwjUhjX8ky9vE9qTtDmkK4vwC";

pub fn create_initialize_user_ix(
    sender_svm: String,
) -> Instruction {

}

pub fn create_initialize_userstats_ix(
    sender_svm: String,
) -> Instruction {

}

pub fn create_deposit_ix(
    sender_svm: String,
) -> Instruction {

}

pub fn create_place_order_ix(
    sender_svm: String,
) -> Instruction {

}

pub fn create_fill_order_ix(
    sender_svm: String,
) -> Instruction {

}


// transfer funds
pub fn deposit(
    _deps: Deps,
    _env: Env,
    svm_pub_key: PubKey,
    deposit_amount: Uint256,
    mint: PubKey,
){

    let (state, _) = PubKey::find_program_address(
        &["drift_state".as_bytes()], &DRIFT_PROGRAM_ID
    );

    let (spot_market_vault, _) = PubKey::find_program_address(
        &["spot_market_vault".as_bytes(), uint16_to_le_bytes(0)], &DRIFT_PROGRAM_ID
    );  

    let (user_token_account, _) = PubKey::find_program_address(
        &["user_token_account".as_bytes(), &SPL_TOKEN_PROGRAM_ID.0, &mint.0], &SPL_TOKEN_PROGRAM_ID
    );

    let (user, _) = PubKey::find_program_address(
        &["user".as_bytes(), &svm_pub_key.0, &[0, 0]], &DRIFT_PROGRAM_ID
    );

    let (user_stats, _) = PubKey::find_program_address(
        &["user_stats".as_bytes(), &svm_pub_key.0], &DRIFT_PROGRAM_ID
    );

    let market_index = 0;
    let (spot_market, _) = PubKey::find_program_address(
        &["spot_market".as_bytes(), uint16_to_le_bytes(market_index)], &DRIFT_PROGRAM_ID
    );

    let initialize_user_stats = InitializeUserStats {
        user_stats: user_stats,
        state: state,
        authority: svm_pub_key,
        payer: svm_pub_key,
        rent: SYS_VAR_RENT_ID,
        system_program: SYSTEM_PROGRAM_ID,
    };

    handle_initialize_user_stats(
        initialize_user_stats,
    );

    let initialize_user = InitializeUser {
        user: user,
        user_stats: user_stats,
        state: state,
        authority: svm_pub_key,
        payer: svm_pub_key,
        rent: SYS_VAR_RENT_ID,
        system_program: SYSTEM_PROGRAM_ID,
    };
    
    handle_initialize_user(
        initialize_user,
        sub_account_id: 0,
        name: [0u8; 32],
    );

    let deposit = Deposit {
        state: state,
        user: user,
        user_stats: user_stats,
        authority: svm_pub_key,   
        spot_market_vault: spot_market_vault,
        user_token_account: user_token_account,
        token_account: SPL_TOKEN_PROGRAM_ID,
    };

    let deposit_ix_builder = {
        deposit,
        market_index: market_index,
        amount: deposit_amount,
        reduce_only: false,
    };


    // struggle with deposit_ix_builder
    deposit_ix_builder.append(
        AccountMeta {
            pubkey: spot_market,
            is_writable: true,
            is_signer: false,
        },
    );

}

pub fn get_drift_perp_market_info(
    deps: Deps,
    env: Env,
    perp_market_index: u16,
    svm_link_input: &FISInput,
) -> StdResult<PerpMarket> {
    let (perp_market, _) = PubKey::find_program_address(
        &["perp_market".as_bytes(), uint16_to_le_bytes(perp_market_index)], &DRIFT_PROGRAM_ID
    );

    let acc_link = from_json::<AccountLink>(svm_link_input.data.get(0).unwrap())?;

    Ok(perp_market_info)
}

pub fn get_drift_user_info(
    deps: Deps,
    env: Env,
    svm_pub_key: PubKey,
    svm_link_input: &FISInput,
) -> StdResult<User> {
    let (user, _) = PubKey::find_program_address(
        &["user".as_bytes(), &svm_pub_key.0, &[0, 0]], &DRIFT_PROGRAM_ID
    );

    let acc_link = from_json::<AccountLink>(svm_link_input.data.get(0).unwrap())?;
    let user_decoder = base64::decode(acc_link.account.data).unwrap();
    // let user_info = 

    Ok(user_info)
}

pub fn place_perp_market_order(
    deps: Deps,
    env: Env,
    market: String,
    usdt_amount: Uint256,
    leverage: Uint256,
    auction_duration: Uint256,
    fis_input: &Vec<FISInput>,
) -> StdResult<Binary> {

    let svm_link_input = fis_input
        .get(0)
        .ok_or(StdError::generic_err("svm account not found"))?;


    let svm_link_input = from_json::<AccountLink>(svm_link_input)?;
    
    let mut all_market = Vec::new();
    for spot_market_index in [0u16, 1].iter() {
        let seed: &[&[u8]] = &[
            b"spot_market",
            &uint16_to_le_bytes(*spot_market_index),
        ];
        let (market, _) = PubKey::find_program_address(seeds, DRIFT_PROGRAM_ID);
        all_markets.push(market);
    };

    let mut all_market = Vec::new();
    for perp_market_index in [0u16, 1].iter() {
        let seed: &[&[u8]] = &[
            b"perp_market",
            &uint16_to_le_bytes(*perp_market_index),
        ];
        let (market, _) = PubKey::find_program_address(seeds, DRIFT_PROGRAM_ID);
        all_markets.push(market);
    };

    let denom = {
        src_plane: "0".to_String(), // plan cosmos
        dst_plane: "3".to_String(), // plan svm
        src_addr: "usdt".to_string(),
    }

    let usdt_minx_hex = denom.dst_plane;
    let usdt_mint_bz = usdt_mint_hex.as_bytes();
    let usdt_mint = PubKey::from_slice(usdt_mint_bz).unwrap();

    deposit(
        deps,
        env,
        svm_pub_key,
        1000000000,
        usdt_mint,
    )

    // get perp market info
    let market_map = HashMap::new();
    let all_oracles = Vec::new();
    for market_index in [0u16, 1, 2].iter() {
        let market_info = get_drift_perp_market_info(
            deps,
            env,
            market_index,
        )?;
        market_map.insert(market_index, market_info);
        all_oracles.push(market_info.amm.oracle);
    }
    
    // get drift user info
    let drift_user = get_drift_user_info(
        deps,
        env,
        svm_pub_key,
    )?;

    let order_id = drift_user.next_order_id;
    let market_orders = vec![
        Order {
            market_index: 0,
            auction_start_price: 65_020_000_000,
            auction_end_price: 65_033_000_000,
            direction: PositionDirection::Long,
            quantity: 500_000,
        },
        Order {
            market_index: 1,
            auction_start_price: 3_001_000_000,
            auction_end_price: 3_004_000_000,
            direction: PositionDirection::Long,
            quantity: 500_000,
        },
        Order {
            market_index: 2,
            auction_start_price: 151_000_000,
            auction_end_price: 151_100_000,
            direction: PositionDirection::Long,
            quantity: 500_000,
        },
    ];

    let unix_expire_time = (Utc::now() + Duration::minutes(10)).timestamp();

    for order in market_orders.iter() {
        let order_params = OrderParams {
            order_type: OrderType::Market,
            market_type: MarketType::Perp,
            direction: OrderDirection::Long,
            user_order_id: order_id,
            base_asset_amount: order.quantity, 
            price: order.auction_end_price,
            market_index: order.market_index,
            reduce_only: false,
            post_only: PostOnlyParam::None,
            immediate_or_cancel: false,
            max_ts: unix_expire_time,
            trigger_price: 0,
            trigger_condition: OrderTriggerCondition::Above,
            oracle_price_offset: 0,
            auction_duration: auction_duration,
            auction_start_price: order.auction_start_price,
            auction_end_price: order.auction_end_price,
        };

        ++order_id;

        let (state, _) = PubKey::find_program_address(
            &["drift_state".as_bytes()], &DRIFT_PROGRAM_ID
        );

        let (user, _) = PubKey::find_program_address(
            &["user".as_bytes(), &svm_pub_key.0, &[0, 0]], &DRIFT_PROGRAM_ID
        );

        let place_order_ix = handle_place_perp_order(
            order_params,
            state,
            user,
            svm_pub_key,
        );
        
        // continue;
        // append part 

    }

    drift_user = get_drift_user_info(
        deps,
        env,
        svm_pub_key,
    )?;
    for order in drift_user.iter() {
        if (order.Status == "Open") {
            let order_id = order.user_order_id;
            let market_index = order.market_index;
            let cancel_order_ix = handle_cancel_order(
                order_id,
                market_index,
                state,
                user,
                svm_pub_key,
            );
        }
    }

    
    Ok(to_json_binary(&StrategyOutput { instructions }).unwrap())
}
