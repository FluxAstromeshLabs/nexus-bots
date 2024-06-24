use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, Int128, Int256};
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
pub struct RaydiumAccounts {
    pub sender_svm_account: String,
    pub authority_account: String,
    pub amm_config_account: String,
    pub pool_state_account: String,
    pub input_token_account: String,
    pub output_token_account: String,
    pub input_vault: String,
    pub output_vault: String,
    pub observer_state: String,
}

#[cw_serde]
pub struct Swap {
    pub dex_name: String,
    pub pool_id: String,
    pub input_denom: String,
    pub output_denom: String,
    pub denom_plane: String,
    pub input_amount: Option<Int128>,

    // only availble on astroport
    pub max_spread: Option<f32>,
    // only available on raydium/svm
    pub raydium_accounts: Option<RaydiumAccounts>,
    // only availble on uniswap
    pub zero_for_one: Option<bool>,
    pub sqrt_price_limit: Option<Int256>,
}

#[cw_serde]
pub struct FISInstruction {
    plane: String,
    action: String,
    address: String,
    msg: Vec<u8>,
}
