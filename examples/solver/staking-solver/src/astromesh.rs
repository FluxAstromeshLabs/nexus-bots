use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, Uint128};

#[cw_serde]
pub enum NexusAction {
    StakeDefault {
        amount: Uint128,
    },
    Stake {
        amount: Uint128,
        validator_address: String,
    },
    ReDelegate {
        amount: Uint128,
        src_validator_address: String,
        new_validator_address: String,
    },
    ClaimAllRewards {},
    UnstakeAll {},
    ClaimRewardsAndRestake {},
}

#[cw_serde]
pub struct MsgWithdrawDelegatorReward {
    #[serde(rename = "@type")]
    pub ty: String,
    pub delegator_address: String,
    pub validator_address: String,
}

#[cw_serde]
pub struct MsgDelegate {
    #[serde(rename = "@type")]
    pub ty: String,
    pub delegator_address: String,
    pub validator_address: String,
    pub amount: Coin,
}

#[cw_serde]
pub struct MsgBeginRedelegate {
    #[serde(rename = "@type")]
    pub ty: String,
    pub delegator_address: String,
    pub validator_src_address: String,
    pub validator_dst_address: String,
    pub amount: Coin,
}

#[cw_serde]
pub struct MsgUndelegate {
    #[serde(rename = "@type")]
    pub ty: String,
    pub delegator_address: String,
    pub validator_address: String,
    pub amount: Coin,
}
