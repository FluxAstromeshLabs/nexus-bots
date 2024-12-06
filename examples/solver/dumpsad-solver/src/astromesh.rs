use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Coin, DenomMetadata, Int128, Uint128, Uint64};
use sha2::{Digest, Sha256};
use tiny_keccak::{Hasher, Keccak};

pub const PLANE_COSMOS: &str = "COSMOS";

pub const QUERY_ACTION_COSMOS_QUERY: &str = "COSMOS_QUERY";
pub const QUERY_ACTION_COSMOS_BANK_BALANCE: &str = "COSMOS_BANK_BALANCE";
pub const QUERY_ACTION_COSMOS_KVSTORE: &str = "COSMOS_KVSTORE";

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
        bot_id: String,
    },
    Buy {
        denom: String,
        amount: Uint128,
        slippage: Uint128,
        pool_address: String,
    },
    Sell {
        denom: String,
        amount: Uint128,
        slippage: Uint128,
        pool_address: String,
    },
}

#[cw_serde]
pub struct CommissionConfig {
    pub management_fee_rate: i64,
    pub management_fee_interval: i64,
    pub trading_fee_rate: i64,
}

impl CommissionConfig {
    pub fn new(
        management_fee_rate: i64,
        management_fee_interval: i64,
        trading_fee_rate: i64,
    ) -> Self {
        CommissionConfig {
            management_fee_rate,
            management_fee_interval,
            trading_fee_rate,
        }
    }
}

#[cw_serde]
pub struct MsgCreatePool {
    #[serde(rename = "@type")]
    pub ty: String,
    pub sender: String,
    pub operator_commission_config: Option<CommissionConfig>,
}

impl MsgCreatePool {
    pub fn new(sender: String, operator_commission_config: Option<CommissionConfig>) -> Self {
        MsgCreatePool {
            ty: "/flux.interpool.v1beta1.MsgCreatePool".to_string(),
            sender,
            operator_commission_config,
        }
    }
}

#[cw_serde]
pub struct MsgUpdatePool {
    #[serde(rename = "@type")]
    pub ty: String,
    pub sender: String,
    pub pool_id: String,
    pub input_blob: Vec<u8>,
    pub output_blob: Vec<u8>,
    pub charge_management_fee: bool,
    pub trading_fee: Vec<Coin>,
    pub cron_id: String,
    pub drivers: Vec<String>,
}

impl MsgUpdatePool {
    pub fn new(
        sender: String,
        pool_id: String,
        input_blob: Vec<u8>,
        output_blob: Vec<u8>,
        charge_management_fee: bool,
        trading_fee: Vec<Coin>,
        cron_id: String,
        drivers: Vec<String>,
    ) -> Self {
        MsgUpdatePool {
            ty: "/flux.interpool.v1beta1.MsgUpdatePool".to_string(),
            sender,
            pool_id,
            input_blob,
            output_blob,
            charge_management_fee,
            trading_fee,
            cron_id,
            drivers,
        }
    }
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
