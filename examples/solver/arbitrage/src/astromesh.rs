use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Coin, Int128, Int256, StdError};
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
    pub sender: String,
    pub denom: String,
    pub amount: Int128,
}

pub trait Pool {
    fn dex_name(&self) -> String;
    fn denom_plane(&self) -> String;
    fn a(&self) -> Int256;
    fn b(&self) -> Int256;
    // returns denom (within denom_plane) and the swap amount
    fn swap_output(&self, input_amount: Int256, a_for_b: bool) -> (String, Int256);
    fn compose_swap_fis(&self, swap: &Swap) -> Result<FISInstruction, StdError>;
    // other functionalities goes here
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
    Arbitrage {
        pair: String, // supported 3 pools and only need usdt amount, usdt => X => usdt
        amount: Int128,
        min_profit: Option<Int128>,
    },
}
