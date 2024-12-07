use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Coin, DenomMetadata, Int128, Uint128, Uint64};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
// use tiny_keccak::{Hasher, Keccak};

pub const PLANE_COSMOS: &str = "COSMOS";
pub const PLANE_EVM: &str = "EVM";
pub const PLANE_SVM: &str = "SVM";
pub const PLANE_WASM: &str = "WASM";

pub const ACTION_VM_INVOKE: &str = "VM_INVOKE";
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
pub struct QueryDenomLinkResponse {
    pub dst_addr: String,
    pub src_decimals: i32,
    pub dst_decimals: i32,
}

pub trait PoolManager {
    fn create_pool_with_initial_liquidity(
        &self,
        sender: String,
        denom_0: String,
        amount_0: Uint128,
        denom_1: String,
        amount_1: Uint128,
    ) -> Vec<FISInstruction>;
}

// pub fn keccak256(input: &[u8]) -> [u8; 32] {
//     let mut hash = Keccak::v256();
//     hash.update(input);
//     let mut output = [0u8; 32];
//     hash.finalize(output.as_mut_slice());
//     output
// }

pub fn sha256(input: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(input);
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

pub fn module_address(typ: &str, key: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    // Hash the type
    hasher.update(typ.as_bytes());
    let th = hasher.finalize(); // Finalize and reset for the first hash
    let mut hasher2 = Sha256::new();

    // Hash the intermediate hash and the key
    hasher2.update(th);
    hasher2.update(key);

    hasher2.finalize().to_vec() // Finalize and return the final hash as Vec<u8>
}
