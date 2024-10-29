use cosmwasm_std::{Binary, Deps, StdError, StdResult};

use crate::svm::{
    InstructionAccountMeta, InstructionMeta, Pubkey, ASSOCIATED_TOKEN_PROGRAM_ID, MINT,
    SPL_TOKEN2022_PROGRAM_ID, SYSTEM_PROGRAM_ID, SYS_VAR_RENT_ID,
};
use borsh::{BorshDeserialize, BorshSerialize};

pub const DRIFT_PROGRAM_ID: &str = "FLR3mfYrMZUnhqEadNJVwjUhjX8ky9vE9qTtDmkK4vwC";
pub const ORACLE_BTC: &str = "3HRnxmtHQrHkooPdFZn5ZQbPTKGvBSyoTi4VVkkoT6u6";
pub const ORACLE_ETH: &str = "2S8JS8K4E7EYnXaoVABFWG3wkxKKaVWEVKZ8GiyinBuS";
pub const ORACLE_SOL: &str = "362SGYeXLRddaacjbyRuXPc1iewF1FrZpRpkyw72LHAM";
pub const DRIFT_STATE: &str = "HYEM9xMiSVsGzwEVRhX3WHH9CB2sFeHnWhyZUR4KVr8c";
pub const DRIFT_DEFAULT_PERCISION: u64 = 1000_000;

pub const ALL_MARKETS: &[&str] = &[
    "GbMqWisskNfP9ZY53cy8eZNK16sg89FKCo4yzpRhFZ2",
    "EUCAzwBhsNnK9BRK6SW4aYbn9eT4foMVHXjpyUP9WuH4",
    "7WrZxBiKCMGuzLCW2VwKK7sQjhTZLbDe5sKfJsEcARpF",
    "E4DJDZwcSWzujRLjoWQXqq4KQVuzbvBiHSR35BPbK7BX",
    "2GKUdmaBJNjfCucDT14HrsWchVrm3yvj4QY2jjnUEg3v",
];

pub const DISCRIMINATOR_OFFSET: usize = 8;
pub const PERP_MARKET_DISCRIMINATOR: &[u8] = &[10, 223, 12, 44, 107, 245, 55, 247];

pub fn create_initialize_user_ixs(
    _deps: Deps,
    sender_svm: String,
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

    // deps.api.debug(&format!("user: {}, userstats: {}", user.to_string(), userstats.to_string()));

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
                    pubkey: DRIFT_STATE.to_string(),
                    is_signer: false,
                    is_writable: true,
                },
                InstructionAccountMeta {
                    pubkey: sender_svm.clone(),
                    is_signer: true,
                    is_writable: true,
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
                    pubkey: DRIFT_STATE.to_string(),
                    is_signer: false,
                    is_writable: true,
                },
                InstructionAccountMeta {
                    pubkey: sender_svm.clone(),
                    is_signer: true,
                    is_writable: true,
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
    _deps: Deps,
    sender_svm: String,
    amount: u64,
) -> StdResult<Vec<InstructionMeta>> {
    let sender_pubkey = Pubkey::from_string(&sender_svm)?;
    let spl_token2022_pubkey = Pubkey::from_string(&SPL_TOKEN2022_PROGRAM_ID.to_string())?;
    let mint = Pubkey::from_string(&MINT.to_string())?;
    let drift_program_id = Pubkey::from_string(&DRIFT_PROGRAM_ID.to_string())?;
    let subacc_index = 0u16.to_le_bytes();
    let (user, _) = Pubkey::find_program_address(
        &["user".as_bytes(), sender_pubkey.0.as_slice(), &subacc_index],
        &drift_program_id,
    )
    .ok_or_else(|| StdError::generic_err("failed to find user PDA"))?;

    let (user_stats, _) = Pubkey::find_program_address(
        &["user_stats".as_bytes(), sender_pubkey.0.as_slice()],
        &drift_program_id,
    )
    .ok_or_else(|| StdError::generic_err("failed to find userstats PDA"))?;

    let market_index = 0u16;
    let (spot_market_vault, _) = Pubkey::find_program_address(
        &[
            "spot_market_vault".as_bytes(),
            &market_index.to_le_bytes().as_slice(),
        ],
        &drift_program_id,
    )
    .ok_or_else(|| StdError::generic_err("failed to find spot market vault PDA"))?;

    let (spot_market, _) = Pubkey::find_program_address(
        &[
            "spot_market".as_bytes(),
            &market_index.to_le_bytes().as_slice(),
        ],
        &drift_program_id,
    )
    .ok_or_else(|| StdError::generic_err("failed to find spot market PDA"))?;

    let associated_token_program_id =
        Pubkey::from_string(&ASSOCIATED_TOKEN_PROGRAM_ID.to_string())?;
    let (user_token_account, _) = Pubkey::find_program_address(
        &[
            sender_pubkey.0.as_slice(),
            spl_token2022_pubkey.0.as_slice(),
            mint.0.as_slice(),
        ],
        &associated_token_program_id,
    )
    .ok_or_else(|| StdError::generic_err("failed to find user token account PDA"))?;

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
                pubkey: spot_market_vault.to_string(),
                is_signer: false,
                is_writable: true,
            },
            InstructionAccountMeta {
                pubkey: user_token_account.to_string(),
                is_signer: false,
                is_writable: true,
            },
            InstructionAccountMeta {
                pubkey: SPL_TOKEN2022_PROGRAM_ID.to_string(),
                is_signer: false,
                is_writable: false,
            },
            InstructionAccountMeta {
                pubkey: spot_market.to_string(),
                is_signer: false,
                is_writable: true,
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
    order_params: OrderParams,
) -> StdResult<Vec<InstructionMeta>> {
    let sender_pubkey = Pubkey::from_string(&sender_svm)?;
    let drift_program_id = Pubkey::from_string(&DRIFT_PROGRAM_ID.to_string())?;
    let subacc_index = 0u16.to_le_bytes();
    let (user, _) = Pubkey::find_program_address(
        &["user".as_bytes(), sender_pubkey.0.as_slice(), &subacc_index],
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
    drift_state: String,
    order_params: OrderParams,
    taker_order_id: u32,
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
    drift_state: String,
    order_params: OrderParams,
) -> StdResult<Vec<InstructionMeta>> {
    let sender_pubkey = Pubkey::from_string(&sender_svm)?;
    let drift_program_id = Pubkey::from_string(&DRIFT_PROGRAM_ID.to_string())?;

    let (filler, _) = Pubkey::find_program_address(
        &["user".as_bytes(), sender_pubkey.0.as_slice()], // TODO: Verify here when we actually implement it
        &drift_program_id,
    )
    .ok_or_else(|| StdError::generic_err("failed to find filler PDA"))?;

    let (filler_stats, _) = Pubkey::find_program_address(
        &["user_stats".as_bytes(), sender_pubkey.0.as_slice()],
        &drift_program_id,
    )
    .ok_or_else(|| StdError::generic_err("failed to find fillerstats PDA"))?;

    // let order_param_bz = borsh::to_vec(&order_params).or_else(|e| Err(StdError::generic_err(format!("serialize order param err: {}", e.to_string()))))?;
    let vamm_fill_data = &[
        // TODO: Compose instruction to fill vAMM order here
    ];

    Ok(vec![InstructionMeta {
        program_id: DRIFT_PROGRAM_ID.to_string(),
        account_meta: vec![
            InstructionAccountMeta {
                pubkey: drift_state.clone(),
                is_signer: false,
                is_writable: true,
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
        data: Binary::new(vamm_fill_data.to_vec()),
    }])
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

pub fn oracle_price_from_perp_market(market_bz: &Binary) -> StdResult<i64> {
    const AMM_OFFSET: usize = 32;
    const HISTORICAL_PRICE_OFFSET: usize = 32;

    let price_bz = market_bz
        .as_slice()
        .get(
            DISCRIMINATOR_OFFSET + AMM_OFFSET + HISTORICAL_PRICE_OFFSET
                ..DISCRIMINATOR_OFFSET + AMM_OFFSET + HISTORICAL_PRICE_OFFSET + 8,
        )
        .ok_or_else(|| {
            StdError::generic_err("read oracle price failed: must have valid data within range")
        })?;
    Ok(i64::from_le_bytes(price_bz.try_into().unwrap()))
}
