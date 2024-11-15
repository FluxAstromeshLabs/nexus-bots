use cosmwasm_schema::cw_serde;
use cosmwasm_std::{entry_point, from_json, to_json_binary, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128};
use std::{vec::Vec};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
pub struct QueryMsg {
    msg: Binary,
    fis_input: Vec<FisInput>
}

#[cw_serde]
pub struct InterpoolResponse {
    pub pool: InterPool
}

#[cw_serde]
pub struct InterPool {
    pub pool_id: String,                // HexBytes equivalent as a hex string
    pub operator_addr: String,          // Address of the pool operator
    pub inventory_snapshot: Vec<Coin>,  // Ongoing assets in the pool
    pub base_capital: Vec<Coin>,        // Initial assets before any trades
    pub operator_commission: u64,       // Commission percentage for the operator
    pub input_blob: Binary,             // Flow control data for cron service
    pub output_blob: Binary,            // Extra state for LP, reward tokens
    pub cron_job_id: Binary,            // Cron job controlling the pool
}

#[cw_serde]
pub struct FisInput {
    data: Vec<Binary>
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
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    // parse cron input
    let pool_info = from_json::<InterpoolResponse>(msg.fis_input.get(0).unwrap().data.get(0).unwrap())?;
    if pool_info.pool.inventory_snapshot.is_empty() {
        return Ok(to_json_binary(&StrategyOutput { instructions: vec![] }).unwrap())
    }

    // parse command, we can store it as proto bytes, encrypted binary
    let mut instructions = vec![];

    // send usdt
    instructions.push(FISInstruction {
        plane: "COSMOS".to_string(),
        action: "COSMOS_BANK_SEND".to_string(),
        address: "".to_string(),
        msg: to_json_binary(&MsgSend {
            from_address: "lux1prtjtfxwzhgtsxwh8n2r9540z3m4uttk7tr9gn".to_string(),
            to_address: pool_info.pool.operator_addr,
            amount: vec![Coin {
                denom: "lux".to_string(),
                amount: Uint128::one(),
            }],
        })
        .unwrap()
        .to_vec(),
    });

    Ok(to_json_binary(&StrategyOutput { instructions }).unwrap())
}
