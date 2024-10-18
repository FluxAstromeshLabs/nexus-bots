use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Coin, Int128, Int256, StdError, Uint256};
use serde::{Deserialize, Serialize};

pub const ETH_DECIMAL_DIFF: u128 = 1_000_000_000u128;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MsgAstroTransfer {
    #[serde(rename = "@type")]
    pub ty: String,
    pub sender: String,
    pub receiver: String,
    pub src_plane: String,
    pub dst_plane: String,
    pub coin: Coin,
}

impl MsgAstroTransfer {
    pub fn new(
        sender: String,
        receiver: String,
        src_plane: String,
        dst_plane: String,
        coin: Coin,
    ) -> Self {
        MsgAstroTransfer {
            ty: "/flux.astromesh.v1beta1.MsgAstroTransfer".to_string(),
            sender,
            receiver,
            src_plane,
            dst_plane,
            coin,
        }
    }
}

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
        market: String,
        usdt_amount: Int128,
        leverage: u8,
        auction_duration: u8,
    },
    FillPerpMarketOrder {
        // TBU
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
