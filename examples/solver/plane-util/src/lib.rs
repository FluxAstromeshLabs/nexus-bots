use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, to_json_vec, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint256
};
use std::vec::Vec;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
pub struct FisInput {
    data: Vec<Binary>,
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
pub enum AbstractionObject {
    WithdrawAllPlanes {},
    DepositEqually {
        denom: String,
        amount: Uint256,
    }
}

#[cw_serde]
pub struct StrategyEvent {
    pub topic: String,
    pub data: Binary,
}

#[cw_serde]
pub struct StrategyOutput {
    instructions: Vec<FISInstruction>,
    events: Vec<StrategyEvent>,
    result: String,
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
pub fn query(_deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let abs_obj = from_json::<AbstractionObject>(msg.msg.to_vec()).unwrap();
    let fis_input = &msg.fis_input.get(0).unwrap().data;

    let instructions = match abs_obj {
        AbstractionObject::WithdrawAllPlanes { } => {
            let address = env.contract.address;
            // get wasm, evm, svm balances in order
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
                    ixs.push(FISInstruction {
                        plane: "COSMOS".to_string(),
                        action: "COSMOS_ASTROMESH_TRANSFER".to_string(),
                        address: "".to_string(),
                        msg: to_json_vec(&AstroTransferMsg {
                            sender: address.to_string(),
                            receiver: address.to_string(),
                            src_plane: plane.to_string(),
                            dst_plane: "COSMOS".to_string(),
                            coin: Coin {
                                denom,
                                amount: balance.amount,
                            },
                        })
                        .unwrap(),
                    })
                }
            }
            ixs
        },
        AbstractionObject::DepositEqually { denom, amount } => {
            let address = env.contract.address;
            let balance = from_json::<Coin>(fis_input.get(0).unwrap()).unwrap();
            assert!(
                amount <= balance.amount,
                "transfer amount must not exceed current balance"
            );
            let divided_amount = amount.checked_div(Uint256::from(3u128)).unwrap();
            vec!["WASM", "EVM", "SVM"]
                .iter()
                .map(|plane| FISInstruction {
                    plane: "COSMOS".to_string(),
                    action: "COSMOS_ASTROMESH_TRANSFER".to_string(),
                    address: "".to_string(),
                    msg: to_json_vec(&AstroTransferMsg {
                        sender: address.to_string(),
                        receiver: address.to_string(),
                        src_plane: "COSMOS".to_string(),
                        dst_plane: plane.to_string(),
                        coin: Coin {
                            denom: denom.clone(),
                            amount: divided_amount,
                        },
                    })
                    .unwrap(),
                })
                .collect()
        }
    };

    StdResult::Ok(to_json_binary(&StrategyOutput { 
        instructions,
        events: vec![StrategyEvent {
            topic: "token_transfer".to_string(),
            data: Binary::from("any data".as_bytes()),
        }],
        result: "transferred/withdrew to planes".to_string()
    }).unwrap())
}
