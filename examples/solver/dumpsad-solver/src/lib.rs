use astromesh::{
    FISInput, FISInstruction, InitialMint, MsgAstroTransfer, MsgCreateBankDenom, NexusAction, PLANE_COSMOS,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, to_json_vec, BankMsg, Binary, Coin, Decimal, DenomMetadata, DenomUnit, Deps, DepsMut, Env, Int128, MessageInfo, Response, StdError, StdResult, Uint128, Uint64
};
use curve::BondingCurve;
use std::{collections::HashMap, vec::Vec};
use svm::{Account, AccountLink, Pubkey, TransactionBuilder};
mod astromesh;
mod svm;
mod test;
mod curve;

const PERCENTAGE_BPS: u128 = 10_000;

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

fn handle_create_token(
    deps: Deps,
    env: Env,
    name: String,
    description: String,
    uri: String,
    target_vm: String,
    solver_address: String,
    fis_input: &Vec<FISInput>,
) -> StdResult<Vec<FISInstruction>> {
    let creator = env.contract.address.to_string();
    let initial_amount = Uint128::new(1_000_000_000);
    let denom_name = format!("{}_denom", name);
    let symbol = format!("{}SYM", target_vm.to_uppercase());
    let display = format!("{}DISPLAY", name.to_lowercase());

    let create_denom_msg = MsgCreateBankDenom::new(
        creator.clone(),
        DenomMetadata {
            description: description.clone(),
            denom_units: vec![
                DenomUnit {
                    denom: denom_name.clone(),
                    exponent: 0,
                    aliases: vec![format!("{}_alias", name)],
                },
                DenomUnit {
                    denom: display.clone(),
                    exponent: 6,
                    aliases: vec![format!("{}_unit", name)],
                },
            ],
            base: denom_name.clone(),
            display: display.clone(),
            name: name.clone(),
            symbol: symbol.clone(),
            uri: uri.clone(),
            uri_hash: "".to_string(),
        },
        "".to_string(),
        vec![InitialMint {
            address: solver_address.clone(),
            amount: initial_amount,
        }],
    );

    Ok(vec![FISInstruction {
        plane: "COSMOS".to_string(),
        action: "COSMOS_INVOKE".to_string(),
        address: "".to_string(),
        msg: to_json_vec(&create_denom_msg)?,
    }])
}

fn handle_buy(
    _deps: Deps,
    env: Env,
    denom: String,
    amount: Uint128,
    slippage: Uint128,
    solver_address: String,
    fis_input: &Vec<FISInput>,
) -> StdResult<Vec<FISInstruction>> {
    let trader = env.contract.address.clone();
    // load quote amount
    let quote_coin = from_json::<Coin>(fis_input.get(0).unwrap().data.get(0).unwrap())?;
    let meme_coin = from_json::<Coin>(fis_input.get(0).unwrap().data.get(1).unwrap())?;

    // calculate the delta Y
    let mut curve = BondingCurve::default(quote_coin.amount, meme_coin.amount);
    let pre_price = curve.price();
    let worst_price = pre_price.checked_mul(slippage.checked_add(Uint128::new(PERCENTAGE_BPS))?)?.checked_div(Uint128::new(PERCENTAGE_BPS))?;

    let bought_amount = curve.buy(amount);
    let post_price = curve.price();
    assert!(post_price.lt(&worst_price), "slippage exceeds, pre price: {}, post price: {}", pre_price, post_price);
    // TODO: Consider charging fee

    // send quote to vault
    let trader_send_quote = FISInstruction {
        plane: PLANE_COSMOS.to_string(),
        action: "COSMOS_INVOKE".to_string(),
        address: "".to_string(),
        msg: to_json_vec(&MsgAstroTransfer::new(
            trader.to_string(), 
            solver_address.clone(), 
            PLANE_COSMOS.to_string(), 
            PLANE_COSMOS.to_string(), 
            Coin { denom: quote_coin.denom, amount }))?,
    };

    // send meme to trader
    let vault_send_meme = FISInstruction {
        plane: PLANE_COSMOS.to_string(),
        action: "COSMOS_INVOKE".to_string(),
        address: "".to_string(),
        msg: to_json_vec(&MsgAstroTransfer::new(
            solver_address,
            trader.to_string(), 
            PLANE_COSMOS.to_string(), 
            PLANE_COSMOS.to_string(), 
            Coin { denom: meme_coin.denom, amount: bought_amount }))?,
    };

    Ok(vec![
        trader_send_quote,
        vault_send_meme,
    ])
}

fn handle_sell(
    deps: Deps,
    env: Env,
    denom: String,
    amount: Uint128,
    slippage: Uint128,
    solver_address: String,
    fis_input: &Vec<FISInput>,
) -> StdResult<Vec<FISInstruction>> {
    let trader = env.contract.address.clone();

    // Load quote and meme amounts from input
    let quote_coin = from_json::<Coin>(fis_input.get(0).unwrap().data.get(0).unwrap())?;
    let meme_coin = from_json::<Coin>(fis_input.get(0).unwrap().data.get(1).unwrap())?;

    // Initialize bonding curve
    let mut curve = BondingCurve::default(quote_coin.amount, meme_coin.amount);
    let pre_price = curve.price();
    let worst_price = pre_price
        .checked_mul(slippage.checked_add(Uint128::new(PERCENTAGE_BPS))?)?
        .checked_div(Uint128::new(PERCENTAGE_BPS))?;

    // Calculate sold amount and verify slippage
    let sold_amount = curve.sell(amount);
    let post_price = curve.price();
    assert!(
        post_price >= worst_price,
        "slippage exceeds, pre price: {}, post price: {}",
        pre_price,
        post_price
    );

    // Transfer instructions
    let trader_send_meme = FISInstruction {
        plane: PLANE_COSMOS.to_string(),
        action: "COSMOS_INVOKE".to_string(),
        address: "".to_string(),
        msg: to_json_vec(&MsgAstroTransfer::new(
            trader.to_string(),
            solver_address.clone(),
            PLANE_COSMOS.to_string(),
            PLANE_COSMOS.to_string(),
            Coin {
                denom: meme_coin.denom,
                amount,
            },
        ))?,
    };

    let vault_send_quote = FISInstruction {
        plane: PLANE_COSMOS.to_string(),
        action: "COSMOS_INVOKE".to_string(),
        address: "".to_string(),
        msg: to_json_vec(&MsgAstroTransfer::new(
            solver_address,
            trader.to_string(),
            PLANE_COSMOS.to_string(),
            PLANE_COSMOS.to_string(),
            Coin {
                denom: quote_coin.denom,
                amount: sold_amount,
            },
        ))?,
    };

    Ok(vec![trader_send_meme, vault_send_quote])
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let nexus_action: NexusAction = from_json(&msg.msg)?;
    let instructions = match nexus_action {
        NexusAction::CreateToken {
            name,
            description,
            uri,
            target_vm,
            solver_address,
        } => handle_create_token(deps, env, name, description, uri, target_vm, solver_address, &msg.fis_input),
        NexusAction::Buy {
            denom,
            amount,
            slippage,
            solver_address,
        } => handle_buy(deps, env, denom, amount, slippage, solver_address, &msg.fis_input),
        NexusAction::Sell {
            denom,
            amount,
            slippage,
            solver_address,
        } => handle_sell(deps, env, denom, amount, slippage, solver_address, &msg.fis_input),
    }?;

    Ok(to_json_binary(&StrategyOutput { instructions }).unwrap())
}
