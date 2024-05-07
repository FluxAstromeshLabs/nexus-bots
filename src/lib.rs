use cosmwasm_std::{
    entry_point, from_json, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, QueryRequest, Response, StdResult, Uint256,
    CustomQuery,
};
use serde::{Deserialize, Serialize};
use cosmwasm_schema::{cw_serde, QueryResponses};
use std::{str::FromStr, vec::Vec};
use schemars::JsonSchema;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(StrategyOutput)]
    Send {
        accounts: Vec<String>,
        amounts: Vec<Vec<BankAmount>>,
    },

    #[returns(StrategyOutput)]
    Distribute {
        accounts: Vec<String>,
        make_odd: bool,
    }
}

#[cw_serde]
pub struct FISInstruction {
    plane: String,
    action: String,
    address: String,
    msg: Vec<u8>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct FISQueryInstruction {
    plane: String,
    action: String,
    address: Binary,
    input: Binary,
}

#[cw_serde]
pub struct FISInstructionResponse {
    plane: String,
    output: Binary,
}

#[cw_serde]
pub struct FISQueryResponse {
    instruction_responses: Vec<FISInstructionResponse>
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct FISQueryRequest {
    instructions: Vec<FISQueryInstruction>,
}

impl CustomQuery for FISQueryRequest {}

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

pub fn fis_query_bank_balances(deps: Deps<FISQueryRequest>, accounts: Vec<String>, denom: String) -> Vec<(String, Uint256)> {
    let query_instructions = (&accounts)
        .into_iter()
        .map(|acc| FISQueryInstruction{
            plane: "COSMOS".to_string(),
            address: Binary::new(deps.api.addr_canonicalize(acc.as_str()).unwrap().to_vec()),
            action: "COSMOS_BANK_BALANCE".to_string(),
            input: Binary::new(denom.clone().into_bytes()),
        }).collect();

    let q = QueryRequest::Custom(FISQueryRequest {
        instructions: query_instructions,
    });

    let result = deps.querier.query::<FISQueryResponse>(&q).unwrap();
    accounts
        .into_iter()
        .enumerate()
        .map(|(i, acc)| {
            let output = from_json::<BankAmount>(&result.instruction_responses[i].output).unwrap();
            (acc, Uint256::from_str(output.amount.as_str()).unwrap())
        }).collect()
}

#[entry_point]
pub fn query(deps: Deps<FISQueryRequest>, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    // if this is a strategy, do 
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
                    msg: to_json_binary(&msg_send).unwrap().to_vec(),
                });
            }
            let strategy_output = StrategyOutput{
                instructions: ixs
            };
            to_json_binary(&strategy_output)
        },
        QueryMsg::Distribute { accounts, make_odd } => {
            let balances = fis_query_bank_balances(deps, accounts, "usdt".to_string());
           
            let mod_result = match make_odd {
                true => 0,
                false => 1,
            };

            let instructions = balances
                .iter()
                .enumerate()
                .filter(|&(_, (_, balance))| *balance % Uint256::from_u128(2u128) == Uint256::from_u128(mod_result as u128))
                .into_iter().map(|(_, (address, _))| FISInstruction {
                    plane: "COSMOS".to_string(),
                    action: "COSMOS_BANK_SEND".to_string(),
                    address: "".to_string(),
                    msg: to_json_binary(&MsgSend{
                        from_address: env.contract.address.clone().into_string(),
                        to_address: address.clone(),
                        amount: vec![BankAmount{
                            denom: "usdt".to_string(),
                            amount: "1".to_string(),
                        }]
                    }).unwrap().to_vec(),
                }).collect();
            
            let strategy_output = StrategyOutput{
                instructions,
            };
            to_json_binary(&strategy_output)
        }
    }
}