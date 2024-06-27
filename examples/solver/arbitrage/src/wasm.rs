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

    const ASTROPORT: &str = "astroport";

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
        token0_denom: String,
        token1_denom: String,
    }

    pub fn get_pool_meta_by_name(pool_name: &String) -> Result<PoolMeta, StdError> {
        match pool_name.as_str() {
            "btc-usdt" => Ok(PoolMeta {
                contract: "lux1nc5tatafv6eyq7llkr2gv50ff9e22mnf70qgjlv737ktmt4eswrqhywrts"
                    .to_string(),
                token0_denom: "btc".to_string(),
                token1_denom: "usdt".to_string(),
            }),
            _ => Err(StdError::not_found(pool_name)),
        }
    }

    pub fn parse_pool(input: &FISInput) -> Result<Pool, StdError> {
        let pool_info = from_json::<PoolResponse>(input.data.get(0).unwrap())?;
        let asset_0 = pool_info.assets.get(0).unwrap();
        let asset_1 = pool_info.assets.get(1).unwrap();
        let asset_0_denom = match asset_0.clone().info {
            AssetInfo::Token { contract_addr } => contract_addr.to_string(),
            AssetInfo::NativeToken { denom } => denom,
        };
        let (mut a, mut b) = (asset_0.amount, asset_1.amount);
        if asset_0_denom != "usdt" {
            (a, b) = (b, a);
        }

        Ok(Pool {
            dex_name: ASTROPORT.to_string(),
            denom_plane: "COSMOS".to_string(),
            a: Int256::from(a.u128()),
            b: Int256::from(b.u128()),
            fee_rate: Int256::from(10000i128),
        })
    }

    pub fn compose_swap_fis(sender: String, swap: &Swap) -> Result<FISInstruction, StdError> {
        let cloned_swap = swap.to_owned();
        let pool = get_pool_meta_by_name(&swap.pool_name)?;

        let msg = MsgExecuteContract::new(
            sender.clone(),
            pool.contract,
            AstroportMsg::Swap {
                offer_asset: Asset {
                    info: AssetInfo::NativeToken { denom: swap.denom },
                    amount: Uint128::new(swap.amount.i128() as u128),
                },
                ask_asset_info: None,
                belief_price: None,
                max_spread: Some(Decimal::from_str("0.5").unwrap()),
                to: Some(sender),
            },
            vec![Coin {
                amount: Uint128::new(swap.amount.i128() as u128),
                denom: swap.denom,
            }],
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
