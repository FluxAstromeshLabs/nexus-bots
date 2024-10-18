use cosmwasm_schema::cw_serde;
use cosmwasm_std::{entry_point, from_json, to_json_binary, to_json_vec, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use std::{vec::Vec};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
pub struct QueryMsg {
    msg: Binary,
    fis_input: Vec<FisInput>
}

#[cw_serde]
pub struct FisInput {
    data: Vec<Binary>
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
    pub amount: Vec<BankAmount>,
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

#[cw_serde]
struct ValidatorReward {
    validator_address: String,
    reward: Vec<BankAmount>,
}

#[cw_serde]
struct RewardsResponse {
    rewards: Vec<ValidatorReward>,
    total: Vec<BankAmount>
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
    // 1. parse claimable rewards
    let fis = &msg.fis_input[0];
    let rewards_response = from_json::<RewardsResponse>(fis.data.first().unwrap()).unwrap();

    let rewards = rewards_response.rewards.clone(); 

    let validator_address = rewards[0].validator_address.clone();
    let reward = &rewards[0].reward[0]; 
    let reward_amount = reward.amount.parse::<u64>().unwrap(); 

    let total = &rewards_response.total[0]; 
    let total_amount = total.amount.parse::<u64>().unwrap(); 

    let delegator_address = env.contract.address.into_string();

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
    let stake_reward = MsgDelegate{
        ty: "/cosmos.staking.v1beta1.MsgDelegate".to_string(),
        delegator_address,
        validator_address,
        amount: vec![BankAmount {
            denom: "lux".to_string(),
            amount: (reward_amount + total_amount).to_string(),
        }],
    };

    instructions.push(FISInstruction {
        plane: "COSMOS".to_string(),
        action: "COSMOS_INVOKE".to_string(),
        address: "".to_string(),
        msg: to_json_vec(&stake_reward).unwrap(),
    });

    Ok(to_json_binary(&StrategyOutput { instructions }).unwrap())
}
