use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, to_json_vec, Binary, Coin,
    DelegationTotalRewardsResponse, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    Uint128,
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
pub struct MsgWithdrawDelegatorReward {
    #[serde(rename = "@type")]
    pub ty: String,
    pub delegator_address: String,
    pub validator_address: String,
}

#[cw_serde]
pub struct MsgDelegate {
    #[serde(rename = "@type")]
    pub ty: String,
    pub delegator_address: String,
    pub validator_address: String,
    pub amount: Vec<Coin>,
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
    amount: Vec<Coin>,
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
    let mut instructions = vec![];

    // 1. parse claimable reards
    let fis = &msg.fis_input[0];

    if fis.data.len() == 0 {
        return Err(StdError::generic_err(
            format!("No rewards to claim").as_str(),
        ));
    }

    let delegator_address = env.contract.address.clone().into_string();

    for idx in 0..fis.data.len() {
        let rewards_response =
            from_json::<DelegationTotalRewardsResponse>(fis.data[idx].clone()).unwrap();

        let rewards = rewards_response.rewards;
        let totals = rewards_response.total;

        let validator_address = rewards[0].validator_address.clone();
        let reward = &rewards[0].reward[0];
        let reward_amount_uint256 = reward.amount.to_uint_floor();
        let reward_amount = reward_amount_uint256
            .to_string()
            .parse::<Uint128>()
            .unwrap();

        let total_amount_uint256 = totals[0].amount.to_uint_floor();
        let total_amount = total_amount_uint256.to_string().parse::<Uint128>().unwrap();

        // 2. compose cosmos msg to claim rewards
        let claim_reward = MsgWithdrawDelegatorReward {
            ty: "/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward".to_string(),
            delegator_address: delegator_address.clone(),
            validator_address: validator_address.to_string(),
        };

        instructions.push(FISInstruction {
            plane: "COSMOS".to_string(),
            action: "COSMOS_INVOKE".to_string(),
            address: "".to_string(),
            msg: to_json_vec(&claim_reward).unwrap(),
        });

        // 3. compose cosmos msg to stake the claimed rewards
        let stake_reward = MsgDelegate {
            ty: "/cosmos.staking.v1beta1.MsgDelegate".to_string(),
            delegator_address: delegator_address.clone(),
            validator_address,
            amount: vec![Coin {
                denom: "usdt".to_string(),
                amount: (reward_amount + total_amount).into(),
            }],
        };

        instructions.push(FISInstruction {
            plane: "COSMOS".to_string(),
            action: "COSMOS_INVOKE".to_string(),
            address: "".to_string(),
            msg: to_json_vec(&stake_reward).unwrap(),
        });
    }

    Ok(to_json_binary(&StrategyOutput { instructions }).unwrap())
}
