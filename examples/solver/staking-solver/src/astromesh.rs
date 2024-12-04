use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, Uint128};

#[cw_serde]
pub enum NexusAction {
    Delegate {
        amount: Uint128,
        validator_name: String,
    },
    Undelegate {
        amount: Uint128,
        validator_name: String,
    },
    ClaimAllRewards {},
    ClaimRewardsAndRedelegate {},
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

#[cw_serde]
pub struct ValidatorResponse {
    pub validators: Vec<Validator>,
    pub pagination: Pagination,
}

#[cw_serde]
pub struct Validator {
    pub operator_address: String,
    pub consensus_pubkey: ConsensusPubkey,
    pub jailed: bool,
    pub status: String,
    pub tokens: String,
    pub delegator_shares: String,
    pub description: ValidatorDescription,
    pub unbonding_height: String,
    pub unbonding_time: String,
    pub commission: Commission,
    pub min_self_delegation: String,
    pub unbonding_on_hold_ref_count: String,
    pub unbonding_ids: Vec<String>,
}

#[cw_serde]
pub struct ConsensusPubkey {
    #[serde(rename = "@type")]
    pub type_field: String, // This field corresponds to "@type"
    pub key: String,
}

#[cw_serde]
pub struct ValidatorDescription {
    pub moniker: String,
    pub identity: String,
    pub website: String,
    pub security_contact: String,
    pub details: String,
}

#[cw_serde]
pub struct Commission {
    pub commission_rates: CommissionRates,
    pub update_time: String,
}

#[cw_serde]
pub struct CommissionRates {
    pub rate: String,
    pub max_rate: String,
    pub max_change_rate: String,
}

#[cw_serde]
pub struct Pagination {
    pub next_key: Option<String>, // This could be null, so use Option
    pub total: String,
}

#[cw_serde]
pub struct DelegationResponse {
    pub delegation_responses: Vec<Delegation>,
    pub pagination: Pagination,
}

#[cw_serde]
pub struct Delegation {
    pub delegation: DelegationDetail,
    pub balance: Coin,
}

#[cw_serde]
pub struct DelegationDetail {
    pub delegator_address: String,
    pub validator_address: String,
    pub shares: String,
}
