use cosmwasm_std::{Binary, Coin};
use serde::{Deserialize, Serialize};

pub mod astroport {
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{Addr, Decimal, Uint128};

    #[cw_serde]
    #[derive(Hash, Eq)]
    pub enum AssetInfo {
        /// Non-native Token
        Token { contract_addr: Addr },
        /// Native token
        NativeToken { denom: String },
    }

    #[cw_serde]
    pub struct Asset {
        /// Information about an asset stored in a [`AssetInfo`] struct
        pub info: AssetInfo,
        /// A token amount
        pub amount: Uint128,
    }

    pub enum AstroportMsg {
        Swap {
            offer_asset: Asset,
            ask_asset_info: Option<AssetInfo>,
            belief_price: Option<Decimal>,
            max_spread: Option<Decimal>,
            to: Option<String>,
        },
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MsgExecuteContract {
    #[serde(rename = "@type")]
    pub ty: String,
    /// Sender is the actor that signed the messages
    pub sender: String,
    /// Contract is the address of the smart contract
    pub contract: String,
    /// Msg is a JSON encoded message to be passed to the contract
    pub msg: Binary,
    /// SentFunds are coins that are transferred to the contract on execution
    pub sent_funds: Vec<Coin>,
}

impl MsgExecuteContract {
    pub fn new(sender: String, contract: String, msg: Binary, sent_funds: Vec<Coin>) -> Self {
        MsgExecuteContract {
            ty: "cosmwasm.wasm.v1beta1.MsgExecuteContract".to_string(),
            sender,
            contract,
            msg,
            sent_funds,
        }
    }
}
