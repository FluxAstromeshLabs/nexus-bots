use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Coin, DenomMetadata, Int128, Uint128, Uint64};
use serde::{Deserialize, Serialize};

pub const PLANE_COSMOS: &str = "COSMOS";
pub const PLANE_EVM: &str = "EVM";
pub const PLANE_SVM: &str = "SVM";
pub const PLANE_WASM: &str = "WASM";

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
pub struct QueryDenomLinkResponse {
    pub dst_addr: String,
    pub src_decimals: i32,
    pub dst_decimals: i32,
}

pub trait PoolManager {
    fn create_pool(&self, denom_0: String, denom_1: String) -> Vec<FISInstruction>;

    fn provide_liquidity_no_lp(
        &self,
        pool_id: String,
        denom_0_amount: Uint128,
        denom_1_amount: Uint128,
    ) -> Vec<FISInstruction>;
}
