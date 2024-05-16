use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, to_json_vec, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint256
};
use serde::Serialize;
use std::{str::FromStr, vec::Vec};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
pub struct FisInput {
    data: Vec<Binary>
}

#[cw_serde]
pub struct QueryMsg {
    msg: Binary,
    fis_input: Vec<FisInput>,
}

#[cw_serde]
pub struct FISInstruction {
    plane: String,
    action: String,
    address: String,
    msg: Vec<u8>,
}

#[cw_serde]
pub struct AbstractionObject {
    action: String,
    denom: String,
    sender: String,
    deposit_amount: Option<Uint256>,
}

#[cw_serde]
pub struct StrategyOutput {
    instructions: Vec<FISInstruction>,
}

#[cw_serde]
pub enum Plane {
    COSMOS,
    WASM,
    EVM,
    SVM,
}

#[cw_serde]
pub struct AstroTransferMsg {
    sender: String,
    receiver: String,
    src_plane: String,
    dst_plane: String,
    coin: Coin,
}

#[cw_serde]
pub struct Coin {
    denom: String,
    amount: Uint256,
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
    assert_eq!(msg.fis_input.len(), 3, "require balance input from 3 planes");
    let abs_obj = from_json::<AbstractionObject>(msg.msg.to_vec()).unwrap();
    // let withdraw_reg = regex::Regex::new("^(lux[a-z,0-9]+) wants to usdt from all planes to cosmos bank account$").unwrap();
    // let deposit_reg = regex::Regex::new("^(lux[a-z,0-9]+) wants to deposit ([0-9]+) usdt equally from bank to all planes$").unwrap();
    let fis_input = &msg.fis_input.get(0).unwrap().data;

    let instructions = if abs_obj.action == "withdraw" {
        let address = abs_obj.sender;
        // get wasm, evm, svm balances    
        let wasm_balance = from_json::<Coin>(fis_input.get(0).unwrap()).unwrap();
        let evm_balance = from_json::<Coin>(fis_input.get(1).unwrap()).unwrap();
        let svm_balance = from_json::<Coin>(fis_input.get(2).unwrap()).unwrap();

        let planes = vec!["WASM", "EVM", "SVM"];
        let balances = vec![wasm_balance, evm_balance, svm_balance];
        let mut ixs = vec![];
        for i in 0..planes.len() {
            let plane = planes.get(i).unwrap();
            let balance = balances.get(i).unwrap();
            let mut denom = balance.clone().denom;
            if plane == &"EVM" || plane == &"SVM" {
                denom = String::from("astro/") + denom.as_str();
            }

            if !balance.amount.is_zero() {
                ixs.push(FISInstruction{
                    plane: "COSMOS".to_string(),
                    action: "COSMOS_ASTROMESH_TRANSFER".to_string(),
                    address: "".to_string(),
                    msg: to_json_vec(&AstroTransferMsg{
                        sender: address.to_string(),
                        receiver: address.to_string(),
                        src_plane: plane.to_string(),
                        dst_plane: "COSMOS".to_string(),
                        coin: Coin { denom, amount: balance.amount},
                    }).unwrap(),
                },)
            }
        }
        ixs
    } else if abs_obj.action == "deposit" {
        let address = abs_obj.sender;
        let amount = abs_obj.deposit_amount.unwrap();
        let balance = from_json::<Coin>(fis_input.get(0).unwrap()).unwrap();
        let denom = &abs_obj.denom;
        if denom != "usdt" || denom != "lux" {
            return  Err(StdError::generic_err("unsupported denom"));
        }

        let decimals = if denom == "usdt" {
            6
        } else {
            18
        };

        let ten_pow_decimals = Uint256::from_u128(10u128).pow(decimals);
        assert!(balance.amount.ge(&amount), "transfer amount must not exceed current balance");
        
        let real_amount = amount.checked_mul(ten_pow_decimals).unwrap();
        let divided_amount = real_amount.checked_div(Uint256::from(3u128)).unwrap();
        vec!["WASM", "EVM", "SVM"].iter().map(
            |plane| FISInstruction{
                plane: "COSMOS".to_string(),
                action: "COSMOS_ASTROMESH_TRANSFER".to_string(),
                address: "".to_string(),
                msg: to_json_vec(&AstroTransferMsg{
                    sender: address.to_string(),
                    receiver: address.to_string(),
                    src_plane:"COSMOS".to_string(),
                    dst_plane: plane.to_string(),
                    coin: Coin { denom: denom.clone(), amount: divided_amount}, 
                }).unwrap(),
            }
        ).collect()
    } else {
        return Err(StdError::generic_err("unsupported intent"));
    };

    StdResult::Ok(to_json_binary(&StrategyOutput { instructions }).unwrap())
}
