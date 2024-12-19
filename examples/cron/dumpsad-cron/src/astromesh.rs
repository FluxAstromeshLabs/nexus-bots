use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Coin, Int64, Uint128, Uint64};
use sha2::{Digest, Sha256};
// use tiny_keccak::{Hasher, Keccak};

pub const PLANE_COSMOS: &str = "COSMOS";
pub const PLANE_EVM: &str = "EVM";
pub const PLANE_SVM: &str = "SVM";
pub const PLANE_WASM: &str = "WASM";

pub const ACTION_VM_INVOKE: &str = "VM_INVOKE";

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

#[cw_serde]
pub struct OracleEntry {
    pub symbol: String,
    pub decimal: Int64,
    pub value: Uint128,
    pub timestamp: Uint64,
}

#[cw_serde]
pub struct OracleEntries {
    pub entries: Vec<OracleEntry>,
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
