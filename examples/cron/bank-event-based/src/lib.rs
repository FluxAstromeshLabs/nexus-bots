use bech32::{Bech32m, Hrp};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, Uint256
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
    receiver: String,
    denom: String,
    amount: String,
}

#[cw_serde]
pub struct Balance {
    acc: Binary,
    balance: Uint128,
}

#[cw_serde]
pub struct DenomUpdate {
    denom: String,
    balances: Vec<Balance>,
}

#[cw_serde]
pub struct BankBalanceUpdate {
    ty: String, // bank event type
    upd: Vec<DenomUpdate>,
    plane: String,
    mode: String,
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
    amount: Vec<BankAmount>,
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
    // parse cron input
    let event_input =
        from_json::<BankBalanceUpdate>(msg.fis_input.get(0).unwrap().data.get(0).unwrap())?;
    let command = from_json::<Command>(msg.msg)?;

    let mut receiver_balance = Option::<Uint128>::None;
    event_input.upd.iter().for_each(|e| {
        if e.denom == command.denom {
            e.balances.iter().for_each(|b| {
                if bech32::encode::<Bech32m>(Hrp::parse("lux").unwrap(), b.acc.as_slice()).unwrap()
                    == command.receiver
                {
                    receiver_balance = Some(b.balance);
                }
            })
        }
    });

    if receiver_balance.is_none() {
        return Ok(to_json_binary(&StrategyOutput {
            instructions: vec![],
        })
        .unwrap());
    }

    // parse command, we can store it as proto bytes, encrypted binary
    let mut instructions = vec![];
    let balance = receiver_balance.unwrap();
    if balance.u128() % 2 == 0 {
        instructions.push(FISInstruction {
            plane: "COSMOS".to_string(),
            action: "COSMOS_BANK_SEND".to_string(),
            address: "".to_string(),
            msg: to_json_binary(&MsgSend {
                from_address: env.contract.address.clone().into_string(),
                to_address: command.receiver.to_string(),
                amount: vec![BankAmount {
                    denom: command.denom.to_string(),
                    amount: command.amount.to_string(),
                }],
            })
            .unwrap()
            .to_vec(),
        });
    
    }
    Ok(to_json_binary(&StrategyOutput { instructions }).unwrap())
}
