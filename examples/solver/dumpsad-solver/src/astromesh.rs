use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Coin, DenomMetadata, Uint128, Uint64};
use sha2::{Digest, Sha256};
use tiny_keccak::{Hasher, Keccak};

pub const PLANE_COSMOS: &str = "COSMOS";

pub const ACTION_COSMOS_INVOKE: &str = "COSMOS_INVOKE";

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
        symbol: String,
        description: String,
        uri: String,
        target_vm: String,
        solver_id: String,
        cron_id: String,
    },
    Buy {
        denom: String,
        amount: Uint128,
        slippage: Uint128,
    },
    Sell {
        denom: String,
        amount: Uint128,
        slippage: Uint128,
    },
}

#[cw_serde]
pub struct Account {
    #[serde(rename = "@type")]
    pub ty: String,
    pub address: String,
    pub pub_key: PublicKey,
    pub account_number: Uint64,
    pub sequence: Uint64,
}

#[cw_serde]
pub struct PublicKey {
    #[serde(rename = "@type")]
    pub ty: String,
    pub key: String,
}

#[cw_serde]
pub struct AccountResponse {
    pub account: Account,
}

#[cw_serde]
pub struct QueryDenomLinkResponse {
    pub dst_addr: String,
    pub src_decimals: i32,
    pub dst_decimals: i32,
}

pub fn keccak256(input: &[u8]) -> [u8; 32] {
    let mut hash = Keccak::v256();
    hash.update(input);
    let mut output = [0u8; 32];
    hash.finalize(output.as_mut_slice());
    output
}

pub fn sha256(input: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(input);
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}
