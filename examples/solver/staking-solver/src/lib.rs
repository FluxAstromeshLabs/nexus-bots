pub mod astromesh;
use astromesh::{
    MsgBeginRedelegate, MsgDelegate, MsgUndelegate, MsgWithdrawDelegatorReward, NexusAction,
    ValidatorResponse,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, to_json_vec, Binary, Coin,
    DelegationTotalRewardsResponse, DelegatorReward, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, Uint128,
};
use std::vec::Vec;

#[cw_serde]
pub struct FisInput {
    data: Vec<Binary>,
}

#[cw_serde]
pub struct FISInstruction {
    pub plane: String,
    pub action: String,
    pub address: String,
    pub msg: Vec<u8>,
}

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
pub struct StrategyOutput {
    instructions: Vec<FISInstruction>,
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

pub fn ix_delegate(
    _deps: Deps,
    amount: Uint128,
    validator_address: String,
    delegator_address: String,
) -> FISInstruction {
    let stake_reward = MsgDelegate {
        ty: "/cosmos.staking.v1beta1.MsgDelegate".to_string(),
        delegator_address: delegator_address.clone(),
        validator_address: validator_address.clone(),
        amount: Coin {
            denom: "lux".to_string(),
            amount: amount.into(),
        },
    };

    let instruction = FISInstruction {
        plane: "COSMOS".to_string(),
        action: "COSMOS_INVOKE".to_string(),
        address: "".to_string(),
        msg: to_json_vec(&stake_reward).unwrap(),
    };

    instruction
}

pub fn ix_withdraw_delegator_reward(
    _deps: Deps,
    validator_address: String,
    delegator_address: String,
) -> FISInstruction {
    let claim_reward = MsgWithdrawDelegatorReward {
        ty: "/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward".to_string(),
        delegator_address: delegator_address.clone(),
        validator_address: validator_address.to_string(),
    };

    let instruction = FISInstruction {
        plane: "COSMOS".to_string(),
        action: "COSMOS_INVOKE".to_string(),
        address: "".to_string(),
        msg: to_json_vec(&claim_reward).unwrap(),
    };

    instruction
}

pub fn ix_undelegate(
    _deps: Deps,
    delegator_address: String,
    validator_address: String,
    amount: Uint128,
) -> FISInstruction {
    let undelegate = MsgUndelegate {
        ty: "/cosmos.staking.v1beta1.MsgUndelegate".to_string(),
        delegator_address: delegator_address.clone(),
        validator_address: validator_address.clone(),
        amount: Coin {
            denom: "lux".to_string(),
            amount: amount.into(),
        },
    };

    let instruction = FISInstruction {
        plane: "COSMOS".to_string(),
        action: "COSMOS_INVOKE".to_string(),
        address: "".to_string(),
        msg: to_json_vec(&undelegate).unwrap(),
    };

    instruction
}

pub fn get_rewards(_deps: Deps, fis_input: &Vec<FisInput>) -> StdResult<Vec<DelegatorReward>> {
    let fis = &fis_input[0];
    let rewards_response =
        from_json::<DelegationTotalRewardsResponse>(fis.data.get(0).unwrap()).unwrap();

    let rewards = rewards_response.rewards;
    if rewards.len() == 0 {
        return Err(StdError::generic_err(
            format!("No rewards to claim").as_str(),
        ));
    }

    Ok(rewards)
}

pub fn get_validator_by_name(
    fis_input: &Vec<FisInput>,
    validator_name: String,
) -> StdResult<String> {
    let fis = &fis_input[1];
    let validators_response = from_json::<ValidatorResponse>(fis.data.get(0).unwrap()).unwrap();

    if validators_response.validators.len() == 0 {
        return Err(StdError::generic_err(
            format!("No validators found").as_str(),
        ));
    }

    for idx in 0..validators_response.validators.len() {
        if validators_response.validators[idx].description.moniker == validator_name {
            return Ok(validators_response.validators[idx].operator_address.clone());
        }
    }

    Err(StdError::generic_err(
        format!("Validator {} not found", validator_name).as_str(),
    ))
}

pub fn delegate(
    deps: Deps,
    env: Env,
    amount: Uint128,
    validator_name: String,
    fis_input: &Vec<FisInput>,
) -> StdResult<Binary> {
    let delegator_address = env.contract.address.to_string();
    let validator_address = get_validator_by_name(fis_input, validator_name.clone())?;

    let instruction = ix_delegate(deps, amount, validator_address, delegator_address);
    Ok(to_json_binary(&StrategyOutput {
        instructions: vec![instruction],
    })
    .unwrap())
}

pub fn claim_all_rewards(deps: Deps, env: Env, fis_input: &Vec<FisInput>) -> StdResult<Binary> {
    let delegator_address = env.contract.address.to_string();
    let mut instructions = vec![];

    let rewards = get_rewards(deps, &fis_input)?;

    for idx in 0..rewards.len() {
        let validator_address = rewards[idx].validator_address.clone();

        instructions.push(ix_withdraw_delegator_reward(
            deps,
            validator_address,
            delegator_address.clone(),
        ));
    }

    Ok(to_json_binary(&StrategyOutput { instructions }).unwrap())
}

pub fn claim_rewards_and_redelegate(
    deps: Deps,
    env: Env,
    fis_input: &Vec<FisInput>,
) -> StdResult<Binary> {
    let delegator_address = env.contract.address.to_string();
    let mut instructions = vec![];

    let rewards = get_rewards(deps, &fis_input)?;

    for idx in 0..rewards.len() {
        let validator_address = rewards[idx].validator_address.clone();
        let reward = &rewards[idx].reward[0];
        let reward_amount_uint256 = reward.amount.to_uint_floor();
        let reward_amount = reward_amount_uint256
            .to_string()
            .parse::<Uint128>()
            .unwrap();

        instructions.push(ix_withdraw_delegator_reward(
            deps,
            validator_address.clone(),
            delegator_address.clone(),
        ));

        instructions.push(ix_delegate(
            deps,
            reward_amount,
            validator_address.clone(),
            delegator_address.clone(),
        ));
    }

    Ok(to_json_binary(&StrategyOutput { instructions }).unwrap())
}

pub fn undelegate(
    deps: Deps,
    env: Env,
    amount: Uint128,
    validator_name: String,
    fis_input: &Vec<FisInput>,
) -> StdResult<Binary> {
    let delegator_address = env.contract.address.to_string();
    let mut instructions = vec![];

    let rewards = get_rewards(deps, &fis_input.clone())?;
    let validator_address = get_validator_by_name(&fis_input, validator_name.clone())?;

    for idx in 0..rewards.len() {
        if rewards[idx].validator_address != validator_address {
            continue;
        }

        let reward = &rewards[idx].reward[0];
        let reward_amount_uint256 = reward.amount.to_uint_floor();
        let reward_amount = reward_amount_uint256
            .to_string()
            .parse::<Uint128>()
            .unwrap();

        if reward_amount < amount {
            return Err(StdError::generic_err(
                format!("Not enough rewards to undelegate").as_str(),
            ));
        }

        instructions.push(ix_undelegate(
            deps,
            delegator_address.clone(),
            validator_address.clone(),
            amount,
        ));

        instructions.push(ix_withdraw_delegator_reward(
            deps,
            validator_address.clone(),
            delegator_address.clone(),
        ));

        return Ok(to_json_binary(&StrategyOutput { instructions }).unwrap());
    }

    Err(StdError::generic_err(
        format!("Validator {} not found", validator_name).as_str(),
    ))
}

pub fn redelegate(
    _deps: Deps,
    env: Env,
    amount: Uint128,
    src_validator_address: String,
    new_validator_address: String,
) -> StdResult<Binary> {
    let delegator = env.contract.address.to_string();
    let mut instructions = vec![];

    let redelegate = MsgBeginRedelegate {
        ty: "/cosmos.staking.v1beta1.MsgBeginRedelegate".to_string(),
        delegator_address: delegator.clone(),
        validator_src_address: src_validator_address.clone(),
        validator_dst_address: new_validator_address.clone(),
        amount: Coin {
            denom: "lux".to_string(),
            amount: amount.into(),
        },
    };

    instructions.push(FISInstruction {
        plane: "COSMOS".to_string(),
        action: "COSMOS_INVOKE".to_string(),
        address: "".to_string(),
        msg: to_json_vec(&redelegate).unwrap(),
    });

    Ok(to_json_binary(&StrategyOutput { instructions }).unwrap())
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let action = from_json::<NexusAction>(msg.msg)?;
    match action {
        NexusAction::Delegate {
            amount,
            validator_name,
        } => delegate(deps, env, amount, validator_name, &msg.fis_input),
        NexusAction::ClaimAllRewards {} => claim_all_rewards(deps, env, &msg.fis_input),
        NexusAction::ClaimRewardsAndRedelegate {} => {
            claim_rewards_and_redelegate(deps, env, &msg.fis_input)
        }
        NexusAction::Undelegate {
            amount,
            validator_name,
        } => undelegate(deps, env, amount, validator_name, &msg.fis_input),
        NexusAction::Redelegate {
            amount,
            src_validator_address,
            new_validator_address,
        } => redelegate(
            deps,
            env,
            amount,
            src_validator_address,
            new_validator_address,
        ),
    }
}
