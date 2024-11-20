use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, to_json_vec, Binary, Coin, Deps, DepsMut, Env, Int64, MessageInfo, Response, StdResult, Uint128, Uint64
};
use std::vec::Vec;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
pub struct QueryMsg {
    msg: Binary,
    fis_input: Vec<FisInput>,
}

#[cw_serde]
pub struct InterpoolResponse {
    pub pool: InterPool,
}

#[cw_serde]
pub struct CommissionConfig {
    pub management_fee_rate: Int64,      // Rate for management fee
    pub management_fee_interval: Int64,  // Interval for applying management fee
    pub trading_fee_rate: Int64,         // Rate for trading fee
}

#[cw_serde]
pub struct CommissionFees {
    pub management_fees: Vec<Coin>, // Fees collected for management
    pub trading_fees: Vec<Coin>,    // Fees collected for trading
}

#[cw_serde]
pub struct MsgUpdatePool {
    pub sender: String,                  // Sender address
    pub pool_id: String,                 // Pool ID
    pub input_blob: Vec<u8>,             // Input blob for processing
    pub output_blob: Vec<u8>,            // Output blob for results
    pub charge_management_fee: bool,     // Flag to charge management fee
    pub trading_fee: Vec<Coin>,          // Trading fees to be charged
    pub cron_id: String,                 // ID of the cron job associated with this update
}

#[cw_serde]
pub struct InterPool {
    pub pool_id: String,               // HexBytes equivalent as a hex string
    pub operator_addr: String,         // Address of the pool operator
    pub inventory_snapshot: Vec<Coin>, // Ongoing assets in the pool
    pub base_capital: Vec<Coin>,       // Initial assets before any trades
    pub operator_commission_config: Option<CommissionConfig>,   // Commission percentage for the operator
    pub operator_commission_fees: Option<CommissionFees>,   // Commission percentage for the operator
    pub input_blob: Option<Binary>,    // Flow control data for cron service
    pub output_blob: Option<Binary>,   // Extra state for LP, reward tokens
    pub cron_id: String,   // Cron job controlling the pool
    pub pool_account: String,
    pub next_commission_time: Uint64,
}

#[cw_serde]
pub struct FisInput {
    data: Vec<Binary>,
}

#[cw_serde]
pub struct CronInput {
    receiver: String,
    amount: String,
    denom: String,
}

#[cw_serde]
pub struct FISInstruction {
    plane: String,
    action: String,
    address: String,
    msg: Vec<u8>,
}

#[cw_serde]
pub struct StrategyOutput {
    instructions: Vec<FISInstruction>,
}

#[cw_serde]
pub struct MsgSend {
    from_address: String,
    to_address: String,
    amount: Vec<Coin>,
}

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[entry_point]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> StdResult<Response> {
    Ok(Response::new().add_attribute("method", "execute"))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    // parse cron input
    let input = msg.fis_input.get(0).unwrap().data.get(0).unwrap();
    deps.api
        .debug(format!("pool input: {:?}", input.to_string()).as_str());
    let pool_info = from_json::<InterpoolResponse>(input)?;
    if pool_info.pool.inventory_snapshot.is_empty() {
        return Ok(to_json_binary(&StrategyOutput {
            instructions: vec![],
        })
        .unwrap());
    }

    deps.api
        .debug(format!("pool info: {:?}", pool_info).as_str());
    // parse command, we can store it as proto bytes, encrypted binary
    let mut instructions = vec![];

    // send usdt
    instructions.push(FISInstruction {
        plane: "COSMOS".to_string(),
        action: "COSMOS_BANK_SEND".to_string(),
        address: "".to_string(),
        msg: to_json_binary(&MsgSend {
            from_address: pool_info.pool.pool_account.clone(),
            to_address: pool_info.pool.operator_addr,
            amount: vec![Coin {
                denom: "usdt".to_string(),
                amount: Uint128::one(),
            }],
        })
        .unwrap()
        .to_vec(),
    });

    instructions.push(FISInstruction {
        plane: "COSMOS".to_string(),
        action: "COSMOS_INVOKE".to_string(),
        address: "".to_string(),
        msg: to_json_vec(&MsgUpdatePool {
            sender: pool_info.pool.pool_account.clone(),
            pool_id: pool_info.pool.pool_id,
            input_blob: vec![],
            output_blob: vec![],
            charge_management_fee: false,
            trading_fee: vec![Coin {
                denom: "usdt".to_string(),
                amount: 10u128.into(),
            }],
            cron_id: "".to_string(),
        }).unwrap(),
    });

    Ok(to_json_binary(&StrategyOutput { instructions }).unwrap())
}
