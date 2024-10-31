use cosmwasm_std::{to_json_string, Binary, StdError};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryInstruction {
    pub plane: String,
    pub action: String,
    pub address: Binary,
    pub input: Vec<Binary>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Query {
    pub instructions: Vec<QueryInstruction>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Prompt {
    pub template: String,
    pub msg_fields: Vec<String>,
    pub query: Query,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Prompts {
    pub stake_default: Prompt,
    pub stake: Prompt,
    pub claim_all_rewards: Prompt,
    pub unstake_all: Prompt,
    pub claim_rewards_and_restake: Prompt,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Group {
    pub name: String,
    pub prompts: Prompts,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Schema {
    pub groups: Vec<Group>,
}

fn main() {

    let stake_default_prompt = Prompt {
        template: "stake ${amount:number} lux".to_string(),
        msg_fields: vec![
            "amount".to_string()
        ],
        query: Query {
            instructions: vec![QueryInstruction {
                plane: "COSMOS".to_string(),
                action: "COSMOS_QUERY".to_string(),
                address: Binary::new(vec![]),
                input: vec![Binary::from(
                    "/cosmos/distribution/v1beta1/delegators/${wallet}/rewards".as_bytes(),
                )],
            }],
        },
    };

    let stake_prompt = Prompt {
        template: "stake ${amount:number} lux with validator at ${validator_address:string}"
            .to_string(),
        msg_fields: vec![
            "amount".to_string(), 
            "validator_address".to_string()
        ],
        query: Query {
            instructions: vec![QueryInstruction {
                plane: "COSMOS".to_string(),
                action: "COSMOS_QUERY".to_string(),
                address: Binary::new(vec![]),
                input: vec![Binary::from(
                    "/cosmos/distribution/v1beta1/delegators/${wallet}/rewards".as_bytes(),
                )],
            }],
        },
    };

    let redelegate_prompt = Prompt {
        template: "move ${amount:number} lux from validator ${src_validator_address:string} to ${new_validator_address:string}".to_string(),
        msg_fields: vec![
            "amount".to_string(),
            "src_validator_address".to_string(),
            "new_validator_address".to_string(),
        ],
        query: Query {
            instructions: vec![QueryInstruction {
                plane: "COSMOS".to_string(),
                action: "COSMOS_QUERY".to_string(),
                address: Binary::new(vec![]),
                input: vec![Binary::from(
                    "/cosmos/distribution/v1beta1/delegators/${wallet}/rewards".as_bytes(),
                )],
            }],
        },
    };

    let claim_all_rewards_prompt = Prompt {
        template: "collect all accumulated staking rewards".to_string(),
        msg_fields: vec![],
        query: Query {
            instructions: vec![QueryInstruction {
                plane: "COSMOS".to_string(),
                action: "COSMOS_QUERY".to_string(),
                address: Binary::new(vec![]),
                input: vec![Binary::from(
                    "/cosmos/distribution/v1beta1/delegators/${wallet}/rewards".as_bytes(),
                )],
            }],
        },
    };

    let unstake_all_prompt = Prompt {
        template: "withdraw all staked lux".to_string(),
        msg_fields: vec![],
        query: Query {
            instructions: vec![QueryInstruction {
                plane: "COSMOS".to_string(),
                action: "COSMOS_QUERY".to_string(),
                address: Binary::new(vec![]),
                input: vec![Binary::from(
                    "/cosmos/distribution/v1beta1/delegators/${wallet}/rewards".as_bytes(),
                )],
            }],
        },
    };

    let claim_rewards_and_restake_prompt = Prompt {
        template: "claim rewards and restake".to_string(),
        msg_fields: vec![],
        query: Query {
            instructions: vec![QueryInstruction {
                plane: "COSMOS".to_string(),
                action: "COSMOS_QUERY".to_string(),
                address: Binary::new(vec![]),
                input: vec![Binary::from(
                    "/cosmos/distribution/v1beta1/delegators/${wallet}/rewards".as_bytes(),
                )],
            }],
        },
    };

    // Constructing the group "Staking Solver"
    let group = Group {
        name: "Staking Solver".to_string(),
        prompts: Prompts {
            stake_default: stake_default_prompt,
            stake: stake_prompt,
            claim_all_rewards: claim_all_rewards_prompt,
            unstake_all: unstake_all_prompt,
            claim_rewards_and_restake: claim_rewards_and_restake_prompt,
        },
    };

    // Creating the schema
    let schema = Schema {
        groups: vec![group],
    };

    // Print the schema as JSON
    let schema_json = to_json_string(&schema).unwrap();
    println!("{}", schema_json);
}
