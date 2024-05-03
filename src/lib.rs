use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use std::vec::Vec;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
}
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Output)]
    Send {
        accounts: Vec<String>,
        amounts: Vec<Vec<BankAmount>>,
    }
}
#[cw_serde]
pub struct FISInstruction {
    plane: String,
    action: String,
    address: String,
    msg: Vec<u8>
}

#[cw_serde]
pub struct StrategyOutput {
    instructions: Vec<FISInstruction>,
}

#[cw_serde]
pub struct MsgSend {
    from_address: String,
    to_address: String,
    amount: Vec<BankAmount>
}
#[cw_serde]
pub struct BankAmount {
    denom: String,
    amount: String,
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
    match msg {
        QueryMsg::Send{accounts, amounts} => {
            let mut ixs: Vec<FISInstruction> = vec![];
            for i in 0..accounts.len() {
                let msg_send = MsgSend{
                    from_address: env.contract.address.clone().into_string(),
                    to_address: accounts[i].clone(),
                    amount: amounts[i].clone()
                };
                ixs.push(FISInstruction{
                    plane: "COSMOS".to_string(),
                    action: "COSMOS_BANK_SEND".to_string(),
                    address: "".to_string(),
                    msg:  to_json_binary(&msg_send).unwrap().to_vec(),
                });
            }
            let strategy_output = StrategyOutput{
                instructions: ixs
            };
            to_json_binary(&strategy_output)
        }
    }
}