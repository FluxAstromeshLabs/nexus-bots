use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, Binary, Coin, Deps, DepsMut, Env, Int64, MessageInfo, Response, StdResult, Uint128
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
pub struct FisInput {
    data: Vec<Binary>,
}

#[cw_serde]
pub struct Command {
    denom: String,
    amount: Uint128,
}

// simplified contract info
#[cw_serde]
pub struct ContractInfo {
    address: String,
    bytecode: Binary,
    hash: Binary,
    sender: String,
    calldata: Binary,
    value: Binary,
    number: Int64,
}

#[cw_serde]
pub struct EventContractDeployed {
    contract: ContractInfo,
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
pub fn query(_deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    // no event => do nothing
    if msg.fis_input.len() == 0 || msg.fis_input.get(0).unwrap().data.len() == 0 {
        return Ok(to_json_binary(&StrategyOutput {
            instructions: vec![],
        })
        .unwrap());
    }

    // parse cron input
    let events: Vec<EventContractDeployed> = msg
        .fis_input
        .get(0)
        .unwrap()
        .data
        .iter()
        .map(|m| {
            from_json::<EventContractDeployed>(m).unwrap()
        })
        .collect();
    
    let command = from_json::<Command>(msg.msg)?;
    let instructions = events
        .iter()
        .map(|e| FISInstruction {
            plane: "COSMOS".to_string(),
            action: "COSMOS_BANK_SEND".to_string(),
            address: "".to_string(),
            msg: to_json_binary(&MsgSend {
                from_address: env.contract.address.clone().into_string(),
                to_address: e.contract.sender.clone(),
                amount: vec![Coin {
                    denom: command.denom.to_string(),
                    amount: command.amount,
                }],
            })
            .unwrap()
            .to_vec(),
        })
        .collect();

    Ok(to_json_binary(&StrategyOutput { instructions }).unwrap())
}
