use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;
use serde::Serialize;

pub mod astroport {
    use std::{default, io::Read, str::FromStr};

    use bech32::{Bech32, Hrp};
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{to_json_vec, Addr, Binary, Coin, Decimal, Uint128};

    use crate::{
        astromesh::{
            self, module_address, sha256, FISInstruction, PoolManager, ACTION_VM_INVOKE,
            PLANE_COSMOS, PLANE_WASM,
        },
        wasm::MsgExecuteContract,
    };

    pub struct Astroport {
        pub contract_sequence: Binary,
    }

    pub const FACTORY_CONTRACT: &str =
        "lux14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9sm3tpfk";
    pub const PAIR_CODE_ID: u64 = 2;

    #[cw_serde]
    #[derive(Hash)]
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

    #[cw_serde]
    pub enum PairType {
        /// XYK pair type
        Xyk {},
        /// Stable pair type
        Stable {},
        /// Custom pair type
        Custom(String),
    }

    #[cw_serde]
    pub enum AstroportMsg {
        CreatePair {
            /// The pair type (exposed in [`PairType`])
            pair_type: PairType,
            /// The assets to create the pool for
            asset_infos: Vec<AssetInfo>,
            /// Optional binary serialised parameters for custom pool types
            init_params: Option<Binary>,
        },

        ProvideLiquidity {
            /// The assets available in the pool
            assets: Vec<Asset>,
            /// The slippage tolerance that allows liquidity provision only if the price in the pool doesn't move too much
            slippage_tolerance: Option<Decimal>,
            /// Determines whether the LP tokens minted for the user is auto_staked in the Generator contract
            auto_stake: Option<bool>,
            /// The receiver of LP tokens
            receiver: Option<String>,
        },
    }

    impl PoolManager for Astroport {
        fn create_pool_with_initial_liquidity(
            &self,
            sender: String,
            denom_0: String,
            amount_0: Uint128,
            denom_1: String,
            amount_1: Uint128,
        ) -> Vec<FISInstruction> {
            let sequence_number =
                u64::from_be_bytes(self.contract_sequence.as_slice().try_into().unwrap());
            let contract_id = &[
                "wasm".as_bytes(),
                &[0],
                PAIR_CODE_ID.to_be_bytes().as_slice(),
                sequence_number.to_be_bytes().as_slice(),
            ]
            .concat();
            let pair_address_bz = module_address("module", &contract_id);
            let pair_address_str =
                bech32::encode::<Bech32>(Hrp::parse("lux").unwrap(), &pair_address_bz).unwrap();

            vec![
                FISInstruction {
                    plane: PLANE_WASM.to_string(),
                    action: ACTION_VM_INVOKE.to_string(),
                    address: "".to_string(),
                    msg: to_json_vec(&MsgExecuteContract::new(
                        sender.clone(),
                        FACTORY_CONTRACT.to_string(),
                        &AstroportMsg::CreatePair {
                            pair_type: PairType::Xyk {},
                            asset_infos: vec![
                                AssetInfo::NativeToken {
                                    denom: denom_0.clone(),
                                },
                                AssetInfo::NativeToken {
                                    denom: denom_1.clone(),
                                },
                            ],
                            init_params: None,
                        },
                        vec![],
                    ))
                    .unwrap(),
                },
                FISInstruction {
                    plane: PLANE_WASM.to_string(),
                    action: ACTION_VM_INVOKE.to_string(),
                    address: "".to_string(),
                    msg: to_json_vec(&MsgExecuteContract::new(
                        sender,
                        pair_address_str,
                        &AstroportMsg::ProvideLiquidity {
                            assets: vec![
                                Asset {
                                    info: AssetInfo::NativeToken {
                                        denom: denom_0.clone(),
                                    },
                                    amount: amount_0,
                                },
                                Asset {
                                    info: AssetInfo::NativeToken {
                                        denom: denom_1.clone(),
                                    },
                                    amount: amount_1,
                                },
                            ],
                            slippage_tolerance: Some(Decimal::from_str("0.5").unwrap()),
                            auto_stake: Some(false),
                            receiver: None, // don't receive LP => no liquidity withdrawal
                        },
                        vec![
                            Coin {
                                denom: denom_0.clone(),
                                amount: amount_0,
                            },
                            Coin {
                                denom: denom_1.clone(),
                                amount: amount_1,
                            },
                        ],
                    ))
                    .unwrap(),
                },
            ]
        }
    }
}

#[cw_serde]
pub struct MsgExecuteContract<T>
where
    T: Serialize,
{
    /// Sender is the actor that signed the messages
    pub sender: String,
    /// Contract is the address of the smart contract
    pub contract: String,
    /// Msg is a JSON encoded message to be passed to the contract
    pub msg: T,
    /// Funds are coins that are transferred to the contract on execution
    pub funds: Vec<Coin>,
}

impl<T> MsgExecuteContract<T>
where
    T: Serialize,
{
    pub fn new(sender: String, contract: String, msg: T, funds: Vec<Coin>) -> Self {
        MsgExecuteContract {
            sender,
            contract,
            msg,
            funds,
        }
    }
}
