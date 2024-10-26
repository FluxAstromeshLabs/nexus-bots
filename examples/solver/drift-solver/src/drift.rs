use cosmwasm_std::{Binary, StdError, StdResult};

use crate::svm::{
    InstructionAccountMeta, InstructionMeta, Pubkey, SPL_TOKEN2022_PROGRAM_ID, SYSTEM_PROGRAM_ID,
    SYS_VAR_RENT_ID,
};
use borsh::{BorshDeserialize, BorshSerialize};

pub const DRIFT_PROGRAM_ID: &str = "FLR3mfYrMZUnhqEadNJVwjUhjX8ky9vE9qTtDmkK4vwC";
pub const ORACLE_BTC: &str = "3HRnxmtHQrHkooPdFZn5ZQbPTKGvBSyoTi4VVkkoT6u6";
pub const ORACLE_ETH: &str = "2S8JS8K4E7EYnXaoVABFWG3wkxKKaVWEVKZ8GiyinBuS";
pub const ORACLE_SOL: &str = "362SGYeXLRddaacjbyRuXPc1iewF1FrZpRpkyw72LHAM";
pub const DRIFT_STATE: &str = "HYEM9xMiSVsGzwEVRhX3WHH9CB2sFeHnWhyZUR4KVr8c";

pub const ALL_MARKETS: &[&str] = &[
    "GbMqWisskNfP9ZY53cy8eZNK16sg89FKCo4yzpRhFZ2",
    "EUCAzwBhsNnK9BRK6SW4aYbn9eT4foMVHXjpyUP9WuH4",
    "7WrZxBiKCMGuzLCW2VwKK7sQjhTZLbDe5sKfJsEcARpF",
    "E4DJDZwcSWzujRLjoWQXqq4KQVuzbvBiHSR35BPbK7BX",
    "2GKUdmaBJNjfCucDT14HrsWchVrm3yvj4QY2jjnUEg3v",
];

pub fn create_initialize_user_ixs(
    sender_svm: String,
    drift_state: String,
) -> StdResult<Vec<InstructionMeta>> {
    let sender_pubkey = Pubkey::from_string(&sender_svm)?;
    let drift_program_id = Pubkey::from_string(&DRIFT_PROGRAM_ID.to_string())?;
    let subacc_index = 0u16.to_le_bytes();
    let (user, _) = Pubkey::find_program_address(
        &["user".as_bytes(), sender_pubkey.0.as_slice(), &subacc_index],
        &drift_program_id,
    )
    .ok_or_else(|| StdError::generic_err("failed to find user PDA"))?;

    let (userstats, _) = Pubkey::find_program_address(
        &["user_stats".as_bytes(), sender_pubkey.0.as_slice()],
        &drift_program_id,
    )
    .ok_or_else(|| StdError::generic_err("failed to find userstats PDA"))?;

    let initialize_user_stat_data = &[254, 243, 72, 98, 251, 130, 168, 213];
    let initialize_user_data = [
        [111, 17, 185, 250, 60, 122, 38, 254].as_slice(),
        subacc_index.as_slice(),
        [0u8; 32].as_slice(),
    ]
    .concat();
    Ok(vec![
        InstructionMeta {
            program_id: DRIFT_PROGRAM_ID.to_string(),
            account_meta: vec![
                InstructionAccountMeta {
                    pubkey: userstats.to_string(),
                    is_signer: false,
                    is_writable: true,
                },
                InstructionAccountMeta {
                    pubkey: drift_state.clone(),
                    is_signer: false,
                    is_writable: true,
                },
                InstructionAccountMeta {
                    pubkey: sender_svm.clone(),
                    is_signer: true,
                    is_writable: false,
                },
                InstructionAccountMeta {
                    pubkey: sender_svm.clone(),
                    is_signer: true,
                    is_writable: true,
                },
                InstructionAccountMeta {
                    pubkey: SYS_VAR_RENT_ID.to_string(),
                    is_signer: false,
                    is_writable: false,
                },
                InstructionAccountMeta {
                    pubkey: SYSTEM_PROGRAM_ID.to_string(),
                    is_signer: false,
                    is_writable: false,
                },
            ],
            data: Binary::new(initialize_user_stat_data.to_vec()),
        },
        InstructionMeta {
            program_id: DRIFT_PROGRAM_ID.to_string(),
            account_meta: vec![
                InstructionAccountMeta {
                    pubkey: user.to_string(),
                    is_signer: false,
                    is_writable: true,
                },
                InstructionAccountMeta {
                    pubkey: userstats.to_string(),
                    is_signer: false,
                    is_writable: true,
                },
                InstructionAccountMeta {
                    pubkey: drift_state,
                    is_signer: false,
                    is_writable: true,
                },
                InstructionAccountMeta {
                    pubkey: sender_svm.clone(),
                    is_signer: true,
                    is_writable: false,
                },
                InstructionAccountMeta {
                    pubkey: sender_svm,
                    is_signer: true,
                    is_writable: true,
                },
                InstructionAccountMeta {
                    pubkey: SYS_VAR_RENT_ID.to_string(),
                    is_signer: false,
                    is_writable: false,
                },
                InstructionAccountMeta {
                    pubkey: SYSTEM_PROGRAM_ID.to_string(),
                    is_signer: false,
                    is_writable: false,
                },
            ],
            data: Binary::new(initialize_user_data),
        },
    ])
}

pub fn create_deposit_usdt_ix(
    sender_svm: String,
    drift_state: String,
    amount: u64,
) -> StdResult<Vec<InstructionMeta>> {
    let sender_pubkey = Pubkey::from_string(&sender_svm)?;
    let drift_program_id = Pubkey::from_string(&DRIFT_PROGRAM_ID.to_string())?;
    let (user, _) = Pubkey::find_program_address(
        &["user".as_bytes(), sender_pubkey.0.as_slice()],
        &drift_program_id,
    )
    .ok_or_else(|| StdError::generic_err("failed to find user PDA"))?;

    let (user_stats, _) = Pubkey::find_program_address(
        &["user_stats".as_bytes(), sender_pubkey.0.as_slice()],
        &drift_program_id,
    )
    .ok_or_else(|| StdError::generic_err("failed to find userstats PDA"))?;

    let market_index = 0u16;
    let deposit_data = &[
        [242, 35, 198, 137, 82, 225, 242, 182].as_slice(),
        market_index.to_le_bytes().as_slice(),
        amount.to_le_bytes().as_slice(),
        &[0],
    ]
    .concat();

    Ok(vec![InstructionMeta {
        program_id: DRIFT_PROGRAM_ID.to_string(),
        account_meta: vec![
            InstructionAccountMeta {
                pubkey: drift_state.clone(),
                is_signer: false,
                is_writable: true,
            },
            InstructionAccountMeta {
                pubkey: user.to_string(),
                is_signer: false,
                is_writable: true,
            },
            InstructionAccountMeta {
                pubkey: user_stats.to_string(),
                is_signer: false,
                is_writable: true,
            },
            InstructionAccountMeta {
                pubkey: sender_svm.clone(),
                is_signer: true,
                is_writable: false,
            },
            InstructionAccountMeta {
                pubkey: sender_svm.clone(),
                is_signer: true,
                is_writable: true,
            },
            InstructionAccountMeta {
                pubkey: SYSTEM_PROGRAM_ID.to_string(),
                is_signer: false,
                is_writable: false,
            },
            InstructionAccountMeta {
                pubkey: SPL_TOKEN2022_PROGRAM_ID.to_string(),
                is_signer: false,
                is_writable: false,
            },
        ],
        data: Binary::new(deposit_data.to_vec()),
    }])
}

fn get_all_oracles_and_markets() -> Vec<InstructionAccountMeta> {
    let mut all_oracles: Vec<InstructionAccountMeta> = [ORACLE_BTC, ORACLE_ETH, ORACLE_SOL]
        .iter()
        .map(|oracle_id| InstructionAccountMeta {
            pubkey: oracle_id.to_string(),
            is_signer: false,
            is_writable: false,
        })
        .collect();

    let all_markets: Vec<InstructionAccountMeta> = ALL_MARKETS
        .iter()
        .map(|id| InstructionAccountMeta {
            pubkey: id.to_string(),
            is_signer: false,
            is_writable: true,
        })
        .collect();

    all_oracles.extend(all_markets);
    all_oracles
}

pub fn create_place_order_ix(
    sender_svm: String,
    taker_svm: String,
    order_params: OrderParams,
) -> StdResult<Vec<InstructionMeta>> {
    let sender_pubkey = Pubkey::from_string(&sender_svm)?;
    let taker_pubkey = Pubkey::from_string(&taker_svm)?;

    let drift_program_id = Pubkey::from_string(&DRIFT_PROGRAM_ID.to_string())?;
    let (user, _) = Pubkey::find_program_address(
        &["user".as_bytes(), sender_pubkey.0.as_slice()],
        &drift_program_id,
    )
    .ok_or_else(|| StdError::generic_err("failed to find user PDA"))?;

    let order_param_bz = borsh::to_vec(&order_params).or_else(|e| {
        Err(StdError::generic_err(format!(
            "serialize order param err: {}",
            e.to_string()
        )))
    })?;
    let place_order_data = &[
        [69, 161, 93, 202, 120, 126, 76, 185].as_slice(),
        order_param_bz.as_slice(),
    ]
    .concat();

    // TODO: we should include only user related oracle/markets in this instruction
    // this add some more filtering logic so I skipped it for now
    let all_oracles_markets = get_all_oracles_and_markets();

    let mut account_meta = vec![
        InstructionAccountMeta {
            pubkey: DRIFT_STATE.to_string(),
            is_signer: false,
            is_writable: true,
        },
        InstructionAccountMeta {
            pubkey: user.to_string(),
            is_signer: false,
            is_writable: true,
        },
        InstructionAccountMeta {
            pubkey: sender_svm.clone(),
            is_signer: true,
            is_writable: true,
        },
        InstructionAccountMeta {
            pubkey: SYSTEM_PROGRAM_ID.to_string(),
            is_signer: false,
            is_writable: false,
        },
    ];
    account_meta.extend(all_oracles_markets);

    Ok(vec![InstructionMeta {
        program_id: DRIFT_PROGRAM_ID.to_string(),
        account_meta,
        data: Binary::new(place_order_data.to_vec()),
    }])
}

pub fn create_fill_order_jit_ix(
    sender_svm: String,
    order_params: OrderParams,
    taker_svm: String,
    taker_order_id: u32,
) -> StdResult<Vec<InstructionMeta>> {
    let sender_pubkey = Pubkey::from_string(&sender_svm)?;
    let taker_pubkey = Pubkey::from_string(&taker_svm)?;
    let drift_program_id = Pubkey::from_string(&DRIFT_PROGRAM_ID.to_string())?;
    let subaccount_id = &[0, 0];
    let (user, _) = Pubkey::find_program_address(
        &["user".as_bytes(), sender_pubkey.0.as_slice(), subaccount_id],
        &drift_program_id,
    )
    .ok_or_else(|| StdError::generic_err("failed to find user PDA"))?;

    let (user_stats, _) = Pubkey::find_program_address(
        &["user_stats".as_bytes(), sender_pubkey.0.as_slice()],
        &drift_program_id,
    )
    .ok_or_else(|| StdError::generic_err("failed to find userstats PDA"))?;

    let (taker_user, _) = Pubkey::find_program_address(
        &["user".as_bytes(), taker_pubkey.0.as_slice(), subaccount_id],
        &drift_program_id,
    )
    .ok_or_else(|| StdError::generic_err("failed to find taker user PDA"))?;

    let (taker_user_stats, _) = Pubkey::find_program_address(
        &["user_stats".as_bytes(), taker_pubkey.0.as_slice()],
        &drift_program_id,
    )
    .ok_or_else(|| StdError::generic_err("failed to find taker userstats PDA"))?;

    let order_param_bz = borsh::to_vec(&order_params).or_else(|e| {
        Err(StdError::generic_err(format!(
            "serialize order param err: {}",
            e.to_string()
        )))
    })?;
    let jit_fill_data = &[
        [149, 158, 85, 66, 239, 9, 243, 98].as_slice(),
        order_param_bz.as_slice(),
        taker_order_id.to_le_bytes().as_slice(),
    ]
    .concat();

    Ok(vec![InstructionMeta {
        program_id: DRIFT_PROGRAM_ID.to_string(),
        account_meta: vec![
            InstructionAccountMeta {
                pubkey: DRIFT_STATE.to_string(),
                is_signer: false,
                is_writable: true,
            },
            InstructionAccountMeta {
                pubkey: user.to_string(),
                is_signer: false,
                is_writable: true,
            },
            InstructionAccountMeta {
                pubkey: user_stats.to_string(),
                is_signer: false,
                is_writable: true,
            },
            InstructionAccountMeta {
                pubkey: sender_svm.clone(),
                is_signer: true,
                is_writable: true,
            },
            InstructionAccountMeta {
                pubkey: SYSTEM_PROGRAM_ID.to_string(),
                is_signer: false,
                is_writable: false,
            },
        ],
        data: Binary::new(jit_fill_data.to_vec()),
    }])
}

pub fn create_fill_order_vamm_ix(
    sender_svm: String,
    taker_svm: String,
    taker_order_id: u32,
) -> StdResult<Vec<InstructionMeta>> {
    let sender_pubkey = Pubkey::from_string(&sender_svm)?;
    let taker_pubkey = Pubkey::from_string(&taker_svm)?;
    let drift_program_id = Pubkey::from_string(&DRIFT_PROGRAM_ID.to_string())?;

    let (filler, _) = Pubkey::find_program_address(
        &["user".as_bytes(), sender_pubkey.0.as_slice()], // TODO: Verify here when we actually implement it
        &drift_program_id,
    )
    .ok_or_else(|| StdError::generic_err("failed to find filler PDA"))?;

    let (filler_stats, _) = Pubkey::find_program_address(
        &["user_stats".as_bytes(), sender_pubkey.0.as_slice(), &[0, 0]],
        &drift_program_id,
    )
    .ok_or_else(|| StdError::generic_err("failed to find fillerstats PDA"))?;

    let (taker, _) = Pubkey::find_program_address(
        &["user".as_bytes(), taker_pubkey.0.as_slice(), &[0, 0]],
        &drift_program_id,
    )
    .ok_or_else(|| StdError::generic_err("failed to find taker PDA"))?;

    let (taker_stats, _) = Pubkey::find_program_address(
        &["user_stats".as_bytes(), taker_pubkey.0.as_slice()],
        &drift_program_id,
    )
    .ok_or_else(|| StdError::generic_err("failed to find takerstats PDA"))?;

    let fill_data = &[
        &[13, 188, 248, 103, 134, 217, 106, 240],
        [1].as_slice(),
        taker_order_id.to_le_bytes().as_slice(),
    ]
    .concat();

    let mut account_meta = vec![
        InstructionAccountMeta {
            pubkey: DRIFT_STATE.to_string(),
            is_signer: false,
            is_writable: true,
        },
        InstructionAccountMeta {
            pubkey: sender_svm.to_string(),
            is_signer: true,
            is_writable: false,
        },
        InstructionAccountMeta {
            pubkey: filler.to_string(),
            is_signer: false,
            is_writable: true,
        },
        InstructionAccountMeta {
            pubkey: filler_stats.to_string(),
            is_signer: false,
            is_writable: true,
        },
        InstructionAccountMeta {
            pubkey: taker.to_string(),
            is_signer: false,
            is_writable: true,
        },
        InstructionAccountMeta {
            pubkey: taker_stats.to_string(),
            is_signer: false,
            is_writable: true,
        },
    ];

    let all_oracles_markets = get_all_oracles_and_markets();
    account_meta.extend(all_oracles_markets);

    let instruction_meta = InstructionMeta {
        program_id: DRIFT_PROGRAM_ID.to_string(),
        account_meta,
        data: Binary::new(fill_data.to_vec()),
    };

    Ok(vec![instruction_meta])
}

#[derive(Default, Clone, Copy, BorshSerialize, BorshDeserialize, PartialEq, Debug, Eq)]
pub enum MarketType {
    #[default]
    Spot,
    Perp,
}

#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, PartialEq, Debug, Eq, Default)]
pub enum OrderType {
    Market,
    #[default]
    Limit,
    TriggerMarket,
    TriggerLimit,
    /// Market order where the auction prices are oracle offsets
    Oracle,
}

#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, PartialEq, Debug, Eq, Default)]
pub enum PositionDirection {
    #[default]
    Long,
    Short,
}

#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, PartialEq, Debug, Eq, Default)]
pub enum PostOnlyParam {
    #[default]
    None,
    MustPostOnly, // Tx fails if order can't be post only
    TryPostOnly,  // Tx succeeds and order not placed if can't be post only
    Slide,        // Modify price to be post only if can't be post only
}

#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, PartialEq, Debug, Eq, Default)]
pub enum OrderTriggerCondition {
    #[default]
    Above,
    Below,
    TriggeredAbove, // above condition has been triggered
    TriggeredBelow, // below condition has been triggered
}

#[derive(Clone, Default, Copy, Eq, PartialEq, Debug, BorshSerialize, BorshDeserialize)]
pub struct OrderParams {
    pub order_type: OrderType,
    pub market_type: MarketType,
    pub direction: PositionDirection,
    pub user_order_id: u8,
    pub base_asset_amount: u64,
    pub price: u64,
    pub market_index: u16,
    pub reduce_only: bool,
    pub post_only: PostOnlyParam,
    pub immediate_or_cancel: bool,
    pub max_ts: Option<i64>,
    pub trigger_price: Option<u64>,
    pub trigger_condition: OrderTriggerCondition,
    pub oracle_price_offset: Option<i32>, // price offset from oracle for order (~ +/- 2147 max)
    pub auction_duration: Option<u8>,     // specified in slots
    pub auction_start_price: Option<i64>, // specified in price or oracle_price_offset
    pub auction_end_price: Option<i64>,   // specified in price or oracle_price_offset
}

#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Default)]
pub enum OrderStatus {
    /// The order is not in use
    #[default]
    Init,
    /// Order is open
    Open,
    /// Order has been filled
    Filled,
    /// Order has been canceled
    Canceled,
}

#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, PartialEq, Debug, Eq, Default)]
pub struct Order {
    /// The slot the order was placed
    pub slot: u64,
    /// The limit price for the order (can be 0 for market orders)
    /// For orders with an auction, this price isn't used until the auction is complete
    /// precision: PRICE_PRECISION
    pub price: u64,
    /// The size of the order
    /// precision for perps: BASE_PRECISION
    /// precision for spot: token mint precision
    pub base_asset_amount: u64,
    /// The amount of the order filled
    /// precision for perps: BASE_PRECISION
    /// precision for spot: token mint precision
    pub base_asset_amount_filled: u64,
    /// The amount of quote filled for the order
    /// precision: QUOTE_PRECISION
    pub quote_asset_amount_filled: u64,
    /// At what price the order will be triggered. Only relevant for trigger orders
    /// precision: PRICE_PRECISION
    pub trigger_price: u64,
    /// The start price for the auction. Only relevant for market/oracle orders
    /// precision: PRICE_PRECISION
    pub auction_start_price: i64,
    /// The end price for the auction. Only relevant for market/oracle orders
    /// precision: PRICE_PRECISION
    pub auction_end_price: i64,
    /// The time when the order will expire
    pub max_ts: i64,
    /// If set, the order limit price is the oracle price + this offset
    /// precision: PRICE_PRECISION
    pub oracle_price_offset: i32,
    /// The id for the order. Each users has their own order id space
    pub order_id: u32,
    /// The perp/spot market index
    pub market_index: u16,
    /// Whether the order is open or unused
    pub status: OrderStatus,
    /// The type of order
    pub order_type: OrderType,
    /// Whether market is spot or perp
    pub market_type: MarketType,
    /// User generated order id. Can make it easier to place/cancel orders
    pub user_order_id: u8,
    /// What the users position was when the order was placed
    pub existing_position_direction: PositionDirection,
    /// Whether the user is going long or short. LONG = bid, SHORT = ask
    pub direction: PositionDirection,
    /// Whether the order is allowed to only reduce position size
    pub reduce_only: bool,
    /// Whether the order must be a maker
    pub post_only: bool,
    /// Whether the order must be canceled the same slot it is placed
    pub immediate_or_cancel: bool,
    /// Whether the order is triggered above or below the trigger price. Only relevant for trigger orders
    pub trigger_condition: OrderTriggerCondition,
    /// How many slots the auction lasts
    pub auction_duration: u8,
    pub padding: [u8; 3],
}

#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Default)]
pub enum SpotBalanceType {
    #[default]
    Deposit,
    Borrow,
}

#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Default)]
pub struct SpotPosition {
    /// The scaled balance of the position. To get the token amount, multiply by the cumulative deposit/borrow
    /// interest of corresponding market.
    /// precision: SPOT_BALANCE_PRECISION
    pub scaled_balance: u64,
    /// How many spot bids the user has open
    /// precision: token mint precision
    pub open_bids: i64,
    /// How many spot asks the user has open
    /// precision: token mint precision
    pub open_asks: i64,
    /// The cumulative deposits/borrows a user has made into a market
    /// precision: token mint precision
    pub cumulative_deposits: i64,
    /// The market index of the corresponding spot market
    pub market_index: u16,
    /// Whether the position is deposit or borrow
    pub balance_type: SpotBalanceType,
    /// Number of open orders
    pub open_orders: u8,
    pub padding: [u8; 4],
}

#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Default)]
pub struct PerpPosition {
    /// The perp market's last cumulative funding rate. Used to calculate the funding payment owed to user
    /// precision: FUNDING_RATE_PRECISION
    pub last_cumulative_funding_rate: i64,
    /// the size of the users perp position
    /// precision: BASE_PRECISION
    pub base_asset_amount: i64,
    /// Used to calculate the users pnl. Upon entry, is equal to base_asset_amount * avg entry price - fees
    /// Updated when the user open/closes position or settles pnl. Includes fees/funding
    /// precision: QUOTE_PRECISION
    pub quote_asset_amount: i64,
    /// The amount of quote the user would need to exit their position at to break even
    /// Updated when the user open/closes position or settles pnl. Includes fees/funding
    /// precision: QUOTE_PRECISION
    pub quote_break_even_amount: i64,
    /// The amount quote the user entered the position with. Equal to base asset amount * avg entry price
    /// Updated when the user open/closes position. Excludes fees/funding
    /// precision: QUOTE_PRECISION
    pub quote_entry_amount: i64,
    /// The amount of open bids the user has in this perp market
    /// precision: BASE_PRECISION
    pub open_bids: i64,
    /// The amount of open asks the user has in this perp market
    /// precision: BASE_PRECISION
    pub open_asks: i64,
    /// The amount of pnl settled in this market since opening the position
    /// precision: QUOTE_PRECISION
    pub settled_pnl: i64,
    /// The number of lp (liquidity provider) shares the user has in this perp market
    /// LP shares allow users to provide liquidity via the AMM
    /// precision: BASE_PRECISION
    pub lp_shares: u64,
    /// The last base asset amount per lp the amm had
    /// Used to settle the users lp position
    /// precision: BASE_PRECISION
    pub last_base_asset_amount_per_lp: i64,
    /// The last quote asset amount per lp the amm had
    /// Used to settle the users lp position
    /// precision: QUOTE_PRECISION
    pub last_quote_asset_amount_per_lp: i64,
    /// Settling LP position can lead to a small amount of base asset being left over smaller than step size
    /// This records that remainder so it can be settled later on
    /// precision: BASE_PRECISION
    pub remainder_base_asset_amount: i32,
    /// The market index for the perp market
    pub market_index: u16,
    /// The number of open orders
    pub open_orders: u8,
    pub per_lp_base: i8,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Default)]
pub struct User {
    /// The owner/authority of the account
    pub authority: Pubkey,
    /// An addresses that can control the account on the authority's behalf. Has limited power, cant withdraw
    pub delegate: Pubkey,
    /// Encoded display name e.g. "toly"
    pub name: [u8; 32],
    /// The user's spot positions
    pub spot_positions: [SpotPosition; 8],
    /// The user's perp positions
    pub perp_positions: [PerpPosition; 8],
    /// The user's orders
    pub orders: [Order; 32],
    /// The last time the user added perp lp positions
    pub last_add_perp_lp_shares_ts: i64,
    /// The total values of deposits the user has made
    /// precision: QUOTE_PRECISION
    pub total_deposits: u64,
    /// The total values of withdrawals the user has made
    /// precision: QUOTE_PRECISION
    pub total_withdraws: u64,
    /// The total socialized loss the users has incurred upon the protocol
    /// precision: QUOTE_PRECISION
    pub total_social_loss: u64,
    /// Fees (taker fees, maker rebate, referrer reward, filler reward) and pnl for perps
    /// precision: QUOTE_PRECISION
    pub settled_perp_pnl: i64,
    /// Fees (taker fees, maker rebate, filler reward) for spot
    /// precision: QUOTE_PRECISION
    pub cumulative_spot_fees: i64,
    /// Cumulative funding paid/received for perps
    /// precision: QUOTE_PRECISION
    pub cumulative_perp_funding: i64,
    /// The amount of margin freed during liquidation. Used to force the liquidation to occur over a period of time
    /// Defaults to zero when not being liquidated
    /// precision: QUOTE_PRECISION
    pub liquidation_margin_freed: u64,
    /// The last slot a user was active. Used to determine if a user is idle
    pub last_active_slot: u64,
    /// Every user order has an order id. This is the next order id to be used
    pub next_order_id: u32,
    /// Custom max initial margin ratio for the user
    pub max_margin_ratio: u32,
    /// The next liquidation id to be used for user
    pub next_liquidation_id: u16,
    /// The sub account id for this user
    pub sub_account_id: u16,
    /// Whether the user is active, being liquidated or bankrupt
    pub status: u8,
    /// Whether the user has enabled margin trading
    pub is_margin_trading_enabled: bool,
    /// User is idle if they haven't interacted with the protocol in 1 week and they have no orders, perp positions or borrows
    /// Off-chain keeper bots can ignore users that are idle
    pub idle: bool,
    /// number of open orders
    pub open_orders: u8,
    /// Whether or not user has open order
    pub has_open_order: bool,
    /// number of open orders with auction
    pub open_auctions: u8,
    /// Whether or not user has open order with auction
    pub has_open_auction: bool,
    pub padding1: [u8; 5],
    pub last_fuel_bonus_update_ts: u32,
    pub padding: [u8; 12],
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Default)]
pub struct Example {
    pub a: Option<u32>,
}
