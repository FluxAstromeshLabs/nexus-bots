use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Uint128};

#[cw_serde]
pub struct StrategyEvent {
    pub strategy_id: String,
    pub op: String,
    pub topic: String,
    pub data: Binary,
}

#[cw_serde]
pub struct GraduateEvent {
    pub price: Uint128,
    pub pool_address: String,
    pub meme_denom: String,
    pub meme_amount: Uint128,
    pub sol_amount: Uint128,
    pub vm: String,
    pub svm_address: String,
    pub meme_vm_denom: String,
}
