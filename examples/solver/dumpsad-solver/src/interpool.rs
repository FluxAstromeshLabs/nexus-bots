use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Coin, Int64, Uint64};

#[cw_serde]
pub struct CommissionConfig {
    pub management_fee_rate: Int64,
    pub management_fee_interval: Int64,
    pub trading_fee_rate: Int64,
}

impl CommissionConfig {
    pub fn new(
        management_fee_rate: i64,
        management_fee_interval: i64,
        trading_fee_rate: i64,
    ) -> Self {
        CommissionConfig {
            management_fee_rate: Int64::new(management_fee_rate),
            management_fee_interval: Int64::new(management_fee_interval),
            trading_fee_rate: Int64::new(trading_fee_rate),
        }
    }
}

#[cw_serde]
pub struct MsgCreatePool {
    #[serde(rename = "@type")]
    pub ty: String,
    pub sender: String,
    pub operator_commission_config: Option<CommissionConfig>,
}

impl MsgCreatePool {
    pub fn new(sender: String, operator_commission_config: Option<CommissionConfig>) -> Self {
        MsgCreatePool {
            ty: "/flux.interpool.v1beta1.MsgCreatePool".to_string(),
            sender,
            operator_commission_config,
        }
    }
}

#[cw_serde]
pub struct MsgUpdatePool {
    #[serde(rename = "@type")]
    pub ty: String,
    pub sender: String,
    pub pool_id: String,
    pub input_blob: Vec<u8>,
    pub output_blob: Vec<u8>,
    pub charge_management_fee: bool,
    pub trading_fee: Vec<Coin>,
    pub cron_id: String,
    pub solver_id: String,
}

impl MsgUpdatePool {
    pub fn new(
        sender: String,
        pool_id: String,
        input_blob: Vec<u8>,
        output_blob: Vec<u8>,
        charge_management_fee: bool,
        trading_fee: Vec<Coin>,
        cron_id: String,
        solver_id: String,
    ) -> Self {
        MsgUpdatePool {
            ty: "/flux.interpool.v1beta1.MsgUpdatePool".to_string(),
            sender,
            pool_id,
            input_blob,
            output_blob,
            charge_management_fee,
            trading_fee,
            cron_id,
            solver_id,
        }
    }
}

#[cw_serde]
pub struct InterPool {
    pub pool_id: String,               // HexBytes -> String
    pub operator_addr: String,         // String remains String
    pub inventory_snapshot: Vec<Coin>, // []Coin -> Vec<Coin>
    pub base_capital: Vec<Coin>,       // []Coin -> Vec<Coin>
    pub operator_commission_config: Option<CommissionConfig>,
    pub operator_commission_fees: Option<CommissionFees>,
    pub input_blob: Option<Binary>,  // []byte -> Vec<u8>
    pub output_blob: Option<Binary>, // []byte -> Vec<u8>
    pub cron_id: String,             // HexBytes -> String
    pub pool_account: String,        // String remains String
    pub next_commission_time: Int64, // int64 -> Int64
    pub solver_id: String,           // HexBytes -> String
}

#[cw_serde]
pub struct CommissionFees {
    pub management_fees: Vec<Coin>, // []Coin -> Vec<Coin>
    pub trading_fees: Vec<Coin>,    // []Coin -> Vec<Coin>
}

#[cw_serde]
pub struct QueryPoolResponse {
    pub pool: InterPool,
}
