use astromesh::{FISInput, FISInstruction, MsgAstroTransfer, NexusAction};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, to_json_vec, Binary, Coin, Deps, DepsMut, Env, Int128, MessageInfo, Response, StdError, StdResult, Uint128, Uint64
};
use std::{collections::HashMap, vec::Vec};
use svm::{Account, AccountLink, Pubkey, TransactionBuilder};
mod astromesh;
mod svm;
mod test;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {}

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

#[cw_serde]
pub struct QueryMsg {
    msg: Binary,
    fis_input: Vec<FISInput>,
}

#[cw_serde]
pub struct StrategyOutput {
    instructions: Vec<FISInstruction>,
}


#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    
    let nexus_action: NexusAction = from_json(&msg.msg)?;
    let mut instructions = vec![];

    match nexus_action {
        NexusAction::CreateToken {
            name,
            description,
            uri,
            target_vm,
        } => {
            // Construct the BankMint message
            let mint_msg = MsgAstroTransfer::new(
                solver_address.clone(),
                solver_address.clone(),
                "COSMOS".to_string(),
                target_vm.clone(),
                Coin {
                    denom: format!("{}-denom", name),
                    amount: initial_mint.into(),
                },
            );

            instructions.push(FISInstruction {
                plane: "COSMOS".to_string(),
                action: "COSMOS_INVOKE".to_string(),
                address: "".to_string(),
                msg: to_json_vec(&mint_msg).unwrap(),
            });
        }
        NexusAction::Buy {
            denom,
            amount,
            slippage,
        } => {
            // Deposit quote amount from initiator to solver account
            let deposit_msg = MsgAstroTransfer::new(
                initiator_address.clone(),
                solver_address.clone(),
                "COSMOS".to_string(),
                "COSMOS".to_string(),
                Coin {
                    denom: "quote-token".to_string(),
                    amount: amount.into(),
                },
            );

            // Transfer the denom from solver account to initiator
            let transfer_msg = MsgAstroTransfer::new(
                solver_address.clone(),
                initiator_address.clone(),
                "COSMOS".to_string(),
                "COSMOS".to_string(),
                Coin {
                    denom: denom.clone(),
                    amount: (amount * (Uint128::new(100) - slippage)) / Uint128::new(100),
                },
            );

            instructions.push(FISInstruction {
                plane: "COSMOS".to_string(),
                action: "COSMOS_INVOKE".to_string(),
                address: "".to_string(),
                msg: to_json_vec(&deposit_msg).unwrap(),
            });

            instructions.push(FISInstruction {
                plane: "COSMOS".to_string(),
                action: "COSMOS_INVOKE".to_string(),
                address: "".to_string(),
                msg: to_json_vec(&transfer_msg).unwrap(),
            });
        }
        NexusAction::Sell {
            denom,
            amount,
            slippage,
        } => {
            // Deposit the denom from initiator to solver account
            let deposit_msg = MsgAstroTransfer::new(
                initiator_address.clone(),
                solver_address.clone(),
                "COSMOS".to_string(),
                "COSMOS".to_string(),
                Coin {
                    denom: denom.clone(),
                    amount: amount.into(),
                },
            );

            // Transfer the quote amount from solver account to initiator
            let transfer_msg = MsgAstroTransfer::new(
                solver_address.clone(),
                initiator_address.clone(),
                "COSMOS".to_string(),
                "COSMOS".to_string(),
                Coin {
                    denom: "quote-token".to_string(),
                    amount: (amount * (Uint128::new(100) - slippage)) / Uint128::new(100),
                },
            );

            instructions.push(FISInstruction {
                plane: "COSMOS".to_string(),
                action: "COSMOS_INVOKE".to_string(),
                address: "".to_string(),
                msg: to_json_vec(&deposit_msg).unwrap(),
            });

            instructions.push(FISInstruction {
                plane: "COSMOS".to_string(),
                action: "COSMOS_INVOKE".to_string(),
                address: "".to_string(),
                msg: to_json_vec(&transfer_msg).unwrap(),
            });
        }
    }

    Ok(to_json_binary(&StrategyOutput { instructions }).unwrap())
}
