use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Coin, DenomMetadata, Int128, Uint128, Uint64};
use serde::{Deserialize, Serialize};

pub const PLANE_COSMOS: &str = "COSMOS";

#[cw_serde]
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
pub struct InitialMint {
    pub address: String,
    pub amount: Uint128,
}

#[cw_serde]
pub struct MsgCreateBankDenom {
    #[serde(rename = "@type")]
    pub ty: String,
    pub sender: String,
    pub metadata: DenomMetadata,
    pub minter: String,
    pub initial_mints: Vec<InitialMint>,
}

impl MsgCreateBankDenom {
    pub fn new(
        sender: String,
        metadata: DenomMetadata,
        minter: String,
        initial_mints: Vec<InitialMint>,
    ) -> Self {
        MsgCreateBankDenom {
            ty: "/flux.astromesh.v1beta1.MsgCreateBankDenom".to_string(),
            sender,
            metadata,
            minter,
            initial_mints,
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
    CreateToken {
        name: String,
        description: String,
        uri: String,
        target_vm: String,
        solver_address: String,
    },
    Buy {
        denom: String,
        amount: Uint128,
        slippage: Uint128,
        solver_address: String,
    },
    Sell {
        denom: String,
        amount: Uint128,
        slippage: Uint128,
        solver_address: String,
    },
}