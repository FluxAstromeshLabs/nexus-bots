use cosmwasm_std::Coin;
use serde::{Deserialize, Serialize};

pub mod astroport {
    use super::MsgExecuteContract;
    use crate::{
        astromesh::{FISInput, Pool, Swap},
        FISInstruction,
    };
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{from_json, to_json_vec, Addr, Coin, Decimal, Int256, StdError, Uint128};
    use std::str::FromStr;

    pub const ASTROPORT: &str = "astroport";
    pub const BPS: i128 = 1000000i128;

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

    pub struct PoolMeta {
        contract: String,
        denom_a: String,
        denom_b: String,
    }

    pub fn get_pool_meta_by_name(pool_name: &String) -> Result<PoolMeta, StdError> {
        let contract = match pool_name.as_str() {
            "btc-usdt" => {
                "lux1nc5tatafv6eyq7llkr2gv50ff9e22mnf70qgjlv737ktmt4eswrqhywrts".to_string()
            }
            "eth-usdt" => {
                "lux1aakfpghcanxtc45gpqlx8j3rq0zcpyf49qmhm9mdjrfx036h4z5sdltq0m".to_string()
            }
            "sol-usdt" => {
                "lux18v47nqmhvejx3vc498pantg8vr435xa0rt6x0m6kzhp6yuqmcp8s3z45es".to_string()
            }
            _ => {
                return Err(StdError::generic_err(format!(
                    "astroport pair not found: {}",
                    pool_name
                )))
            }
        };

        // Split the pair to extract denom_a and denom_b
        let denoms: Vec<&str> = pool_name.split('-').collect();
        if denoms.len() != 2 {
            return Err(StdError::generic_err(format!(
                "invalid pair format: {}",
                pool_name
            )));
        }

        Ok(PoolMeta {
            contract,
            denom_a: denoms[0].to_string(),
            denom_b: denoms[1].to_string(),
        })
    }

    #[cw_serde]
    #[derive(Default)]
    pub struct AstroportPool {
        pub dex_name: String,
        pub denom_plane: String,
        pub a: Int256,
        pub b: Int256,
        pub fee_rate: Int256,
        pub denom_a: String,
        pub denom_b: String,
    }

    impl AstroportPool {
        pub fn new(pair: &str) -> Result<Self, StdError> {
            // Create and return the AstroportPool struct with amounts set to zero and denominations empty
            let pool_meta = get_pool_meta_by_name(&pair.to_string())?;
            Ok(AstroportPool {
                dex_name: ASTROPORT.to_string(),
                denom_plane: "COSMOS".to_string(),
                a: Int256::zero(),
                b: Int256::zero(),
                fee_rate: Int256::from(1000i128),
                denom_a: pool_meta.denom_a,
                denom_b: pool_meta.denom_b,
            })
        }

        pub fn from_fis(input: &FISInput) -> Result<Self, StdError> {
            let pool_info = from_json::<PoolResponse>(input.data.first().unwrap())?;
            let asset_0 = pool_info.assets.first().unwrap();
            let asset_1 = pool_info.assets.get(1).unwrap();
            let mut asset_0_denom = match &asset_0.info {
                AssetInfo::Token { contract_addr } => contract_addr.to_string(),
                AssetInfo::NativeToken { denom } => denom.clone(),
            };
            let mut asset_1_denom = match &asset_1.info {
                AssetInfo::Token { contract_addr } => contract_addr.to_string(),
                AssetInfo::NativeToken { denom } => denom.clone(),
            };

            let (mut a, mut b) = (asset_0.amount, asset_1.amount);
            if asset_0_denom != "usdt" {
                (a, b) = (b, a);
                (asset_0_denom, asset_1_denom) = (asset_1_denom, asset_0_denom);
            }

            Ok(Self {
                dex_name: ASTROPORT.to_string(),
                denom_plane: "COSMOS".to_string(),
                a: Int256::from(a.u128()),
                b: Int256::from(b.u128()),
                fee_rate: Int256::from(10000i128),
                denom_a: asset_0_denom,
                denom_b: asset_1_denom,
            })
        }
    }

    impl Pool for AstroportPool {
        fn dex_name(&self) -> String {
            self.dex_name.clone()
        }

        fn denom_plane(&self) -> String {
            self.denom_plane.clone()
        }

        fn a(&self) -> Int256 {
            self.a
        }

        fn b(&self) -> Int256 {
            self.b
        }

        fn swap_output(&self, x: Int256, a_for_b: bool) -> (String, Int256) {
            let bps = Int256::from_i128(BPS);
            if a_for_b {
                (
                    self.denom_b.clone(),
                    (self.b * x * (bps - self.fee_rate)) / ((self.a + x) * bps),
                )
            } else {
                (
                    self.denom_a.clone(),
                    (self.a * x * (bps - self.fee_rate)) / ((self.b + x) * bps),
                )
            }
        }

        fn compose_swap_fis(&self, swap: &Swap) -> Result<Vec<FISInstruction>, StdError> {
            let pool = get_pool_meta_by_name(&swap.pool_name)?;

            let msg = MsgExecuteContract::new(
                swap.sender.clone(),
                pool.contract,
                AstroportMsg::Swap {
                    offer_asset: Asset {
                        info: AssetInfo::NativeToken {
                            denom: swap.denom.clone(),
                        },
                        amount: Uint128::new(swap.amount.i128() as u128),
                    },
                    ask_asset_info: None,
                    belief_price: None,
                    max_spread: Some(Decimal::from_str("0.5").unwrap()),
                    to: Some(swap.sender.clone()),
                },
                vec![Coin {
                    amount: Uint128::new(swap.amount.i128() as u128),
                    denom: swap.denom.clone(),
                }],
            );

            Ok(vec![FISInstruction {
                plane: "WASM".to_string(),
                action: "VM_INVOKE".to_string(),
                address: "".to_string(),
                msg: to_json_vec(&msg)?,
            }])
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
