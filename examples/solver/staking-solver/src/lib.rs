pub mod astromesh;
use astromesh::{
    MsgBeginRedelegate, MsgDelegate, MsgUndelegate, MsgWithdrawDelegatorReward, NexusAction,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, to_json_vec, Binary, Coin,
    DelegationTotalRewardsResponse, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    Uint128,
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

pub fn staking(
    _deps: Deps,
    amount: Uint128,
    validator: String,
    delegator: String,
) -> FISInstruction {
    let stake_reward = MsgDelegate {
        ty: "/cosmos.staking.v1beta1.MsgDelegate".to_string(),
        delegator_address: delegator.clone(),
        validator_address: validator.clone(),
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

pub fn withdraw_delegator_reward(
    _deps: Deps,
    validator: String,
    delegator: String,
) -> FISInstruction {
    let claim_reward = MsgWithdrawDelegatorReward {
        ty: "/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward".to_string(),
        delegator_address: delegator.clone(),
        validator_address: validator.to_string(),
    };

    let instruction = FISInstruction {
        plane: "COSMOS".to_string(),
        action: "COSMOS_INVOKE".to_string(),
        address: "".to_string(),
        msg: to_json_vec(&claim_reward).unwrap(),
    };

    instruction
}

pub fn undelegate(
    _deps: Deps,
    delegator: String,
    validator: String,
    amount: Uint128,
) -> FISInstruction {
    let undelegate = MsgUndelegate {
        ty: "/cosmos.staking.v1beta1.MsgUndelegate".to_string(),
        delegator_address: delegator.clone(),
        validator_address: validator.clone(),
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

pub fn stake_default(deps: Deps, env: Env, amount: Uint128) -> StdResult<Binary> {
    let delegator = env.contract.address.to_string();
    let validator = "luxvaloper1qry5x2d383v9hkqc0fpez53yluyxvey2c957m4".to_string();

    let instructions = vec![staking(deps, amount, validator, delegator)];
    Ok(to_json_binary(&StrategyOutput { instructions }).unwrap())
}

pub fn stake(deps: Deps, env: Env, amount: Uint128, validator: String) -> StdResult<Binary> {
    let delegator = env.contract.address.to_string();

    let instructions = vec![staking(deps, amount, validator, delegator)];
    Ok(to_json_binary(&StrategyOutput { instructions }).unwrap())
}

pub fn claim_all_rewards(deps: Deps, env: Env, fis_input: Vec<FisInput>) -> StdResult<Binary> {
    let delegator_address = env.contract.address.to_string();
    let mut instructions = vec![];

    let fis = &fis_input[0];
    let rewards_response =
        from_json::<DelegationTotalRewardsResponse>(fis.data.first().unwrap()).unwrap();

    let rewards = rewards_response.rewards;
    if rewards.len() == 0 {
        return Err(StdError::generic_err(
            format!("No rewards to claim").as_str(),
        ));
    }

    for idx in 0..rewards.len() {
        let validator_address = rewards[idx].validator_address.clone();

        instructions.push(withdraw_delegator_reward(
            deps,
            validator_address,
            delegator_address.clone(),
        ));
    }

    Ok(to_json_binary(&StrategyOutput { instructions }).unwrap())
}

pub fn claim_rewards_and_restake(
    deps: Deps,
    env: Env,
    fis_input: Vec<FisInput>,
) -> StdResult<Binary> {
    let delegator_address = env.contract.address.to_string();
    let mut instructions = vec![];

    let fis = &fis_input[0];
    let rewards_response =
        from_json::<DelegationTotalRewardsResponse>(fis.data.first().unwrap()).unwrap();

    let rewards = rewards_response.rewards;
    if rewards.len() == 0 {
        return Err(StdError::generic_err(
            format!("No rewards to claim").as_str(),
        ));
    }

    for idx in 0..rewards.len() {
        let validator_address = rewards[idx].validator_address.clone();
        let reward = &rewards[idx].reward[0];
        let reward_amount_uint256 = reward.amount.to_uint_floor();
        let reward_amount = reward_amount_uint256
            .to_string()
            .parse::<Uint128>()
            .unwrap();

        instructions.push(withdraw_delegator_reward(
            deps,
            validator_address.clone(),
            delegator_address.clone(),
        ));
        instructions.push(staking(
            deps,
            reward_amount,
            validator_address.clone(),
            delegator_address.clone(),
        ));
    }

    Ok(to_json_binary(&StrategyOutput { instructions }).unwrap())
}

pub fn unstake_all(deps: Deps, env: Env, fis_input: Vec<FisInput>) -> StdResult<Binary> {
    let delegator = env.contract.address.to_string();
    let mut instructions = vec![];

    let fis = &fis_input[0];
    let rewards_response =
        from_json::<DelegationTotalRewardsResponse>(fis.data.first().unwrap()).unwrap();

    let rewards = rewards_response.rewards;
    if rewards.len() == 0 {
        return Err(StdError::generic_err(
            format!("No rewards to unstake").as_str(),
        ));
    }

    for idx in 0..rewards.len() {
        let validator_address = rewards[idx].validator_address.clone();
        let reward = &rewards[idx].reward[0];
        let reward_amount_uint256 = reward.amount.to_uint_floor();
        let reward_amount = reward_amount_uint256
            .to_string()
            .parse::<Uint128>()
            .unwrap();

        instructions.push(undelegate(
            deps,
            delegator.clone(),
            validator_address.clone(),
            reward_amount,
        ));

        instructions.push(withdraw_delegator_reward(
            deps,
            validator_address.clone(),
            delegator.clone(),
        ));
    }

    Ok(to_json_binary(&StrategyOutput { instructions }).unwrap())
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
        NexusAction::StakeDefault { amount } => stake_default(deps, env, amount),
        NexusAction::Stake {
            amount,
            validator_address,
        } => stake(deps, env, amount, validator_address),
        NexusAction::ClaimAllRewards {} => claim_all_rewards(deps, env, msg.fis_input),
        NexusAction::ClaimRewardsAndRestake {} => {
            claim_rewards_and_restake(deps, env, msg.fis_input)
        }
        NexusAction::UnstakeAll {} => unstake_all(deps, env, msg.fis_input),
        NexusAction::ReDelegate {
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
