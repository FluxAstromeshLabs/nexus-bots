use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Coin, Int128, Int256, StdError, Uint256};
use serde::{Deserialize, Serialize};

pub const ETH_DECIMAL_DIFF: u128 = 1_000_000_000u128;

#[cw_serde]
pub struct FISInput {
    pub data: Vec<Binary>,
}

#[cw_serde]
pub struct FISInstruction {
    pub plane: String,
    pub action: String,
    pub address: String,
    pub msg: Vec<u8>,
}

#[cw_serde]
pub enum NexusAction {
    PlacePerpMarketOrder {
        usdt_amount: Int128,
        leverage: u8,
        market: String,
        auction_duration: u8,
    },
    FillPerpMarketOrder {
        taker_svm_address: String,
        taker_order_id: u32,
        percent: u8,
    },
}

pub fn uint16_to_le_bytes(x: u16) -> [u8; 2] {
    x.to_le_bytes()
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
