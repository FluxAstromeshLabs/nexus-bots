use cosmwasm_std::{Binary, Coin};
use serde::{Deserialize, Serialize};

pub mod astroport {
    use std::str::FromStr;

    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{to_json_binary, to_json_vec, Addr, Decimal, StdError, Uint128};

    use crate::{astromesh::Swap, FISInstruction};

    use super::MsgExecuteContract;

    // TODO: Get these from astroport library
    #[cw_serde]
    #[derive(Hash, Eq)]
    pub enum AssetInfo {
        /// Non-native Token
        Token { contract_addr: Addr },
        /// Native token
        NativeToken { denom: String },
    }

    #[cw_serde]
    pub struct PoolResponse {
        /// The assets in the pool together with asset amounts
        pub assets: Vec<Asset>,
        /// The total amount of LP tokens currently issued
        pub total_share: Uint128,
    }

    #[cw_serde]
    pub struct Asset {
        /// Information about an asset stored in a [`AssetInfo`] struct
        pub info: AssetInfo,
        /// A token amount
        pub amount: Uint128,
    }

    #[cw_serde]
    pub enum AstroportMsg {
        Swap {
            offer_asset: Asset,
            ask_asset_info: Option<AssetInfo>,
            belief_price: Option<Decimal>,
            max_spread: Option<Decimal>,
            to: Option<String>,
        },
    }

    pub fn compose_swap_fis(sender: String, swap: Swap) -> Result<FISInstruction, StdError> {
        let clone_swap = swap.to_owned();
        let msg = MsgExecuteContract::new(
            sender.clone(),
            clone_swap.pool_id,
            to_json_binary(&AstroportMsg::Swap {
                offer_asset: Asset {
                    info: AssetInfo::NativeToken {
                        denom: clone_swap.input_denom,
                    },
                    amount: Uint128::new(swap.input_amount.unwrap().i128() as u128),
                },
                ask_asset_info: None,
                belief_price: None,
                max_spread: Some(Decimal::from_str("0.5").unwrap()),
                to: Some(sender),
            })?,
            vec![],
        );

        Ok(FISInstruction {
            plane: "WASM".to_string(),
            action: "VM_INVOKE".to_string(),
            address: "".to_string(),
            msg: to_json_vec(&msg)?,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MsgExecuteContract {
    // #[serde(rename = "@type")]
    // pub ty: String,
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
            // ty: "cosmwasm.wasm.v1beta1.MsgExecuteContract".to_string(),
            sender,
            contract,
            msg,
            sent_funds,
        }
    }
}
