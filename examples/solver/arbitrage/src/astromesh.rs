use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Coin, Int128, Int256};
use serde::{Deserialize, Serialize};

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
pub struct Swap {
    pub dex_name: String,
    pub pool_name: String,
    pub denom: String,
    pub amount: Int128,
}

#[cw_serde]
#[derive(Default)]
pub struct Pool {
    pub dex_name: String,
    pub denom_plane: String,
    pub a: Int256,
    pub b: Int256,
    pub fee_rate: Int256,
}

#[cw_serde]
pub struct FISInstruction {
    plane: String,
    action: String,
    address: String,
    msg: Vec<u8>,
}

#[cw_serde]
pub enum NexusAction {
    Arbitrage {
        pair: String, // 3 pools needed and only need usdt amount, usdt => X => usdt
        usdt_amount: Int256,
        min_profit: Option<Int256>,
    },
}
