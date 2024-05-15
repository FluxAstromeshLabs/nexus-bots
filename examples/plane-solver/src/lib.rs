use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Uint256,
};
use std::{str::FromStr, vec::Vec};
use std::string::ToString;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
pub struct QueryMsg {
    msg: Binary,
    fis_input: Vec<Binary>,
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
    amount: String
}

const PLANE_COSMOS: String = "COSMOS".to_string();
const PLANE_EVM: String = "EVM".to_string();
const PLANE_WASM: String = "WASM".to_string();
const PLANE_SVM: String = "SVM".to_string();


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
    // parse command, we can store it as proto bytes, encrypted binary
    let command = from_json::<Fund>(msg.msg)?;
    let mut instructions = vec![];
    for i in 0..msg.fis_input.len() {
        let fis_input = from_json::<BankAmount>(msg.fis_input.get(i).unwrap())?;
        let balance = Uint256::from_str(fis_input.amount.as_str()).unwrap();
        if balance % Uint256::from_u128(2u128) == Uint256::zero() {
            instructions.push(FISInstruction{
                plane: PLANE_COSMOS,
                action: "COSMOS_ASTROMESH_TRANSFER".to_string(),
                address: "".to_string(),
                msg: to_json_binary(&AstroTransferMsg{
                    sender: env.contract.address.to_string(),
                    receiver: env.contract.address.to_string(),
                    src_plane: "EVM".to_string(),
                    dst_plane: "COSMOS".to_string(),
                    coin: Coin{
                        denom: "",
                        amount: "",
                    },
                })
                .unwrap()
                .to_vec(),
            })
        }
    }

    StdResult::Ok(to_json_binary(&StrategyOutput { instructions }).unwrap())
}
