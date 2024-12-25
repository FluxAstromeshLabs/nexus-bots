use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct CreateTokenEvent {
    pub denom: String,
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub pool_id: String,
    pub vm: String,
    pub logo: String,
    pub cron_id: String,
    pub solver_id: String,
}

#[cw_serde]
pub struct TradeTokenEvent {
    pub denom: String,
    pub price: Uint128,
    pub trader: String,
    pub curve_sol_amount: Uint128,
    pub meme_amount: Uint128,
    pub sol_amount: Uint128,
}

#[cw_serde]
pub struct GraduateEvent {
    pub price: Uint128,
    pub pool_address: String,
    pub meme_denom: String,
    pub meme_amount: Uint128,
    pub sol_amount: Uint128,
    pub vm: String,
    pub pool_svm_address: String,
    pub meme_denom_link: String,
}
