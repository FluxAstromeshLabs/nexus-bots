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
    pub delegate: Prompt,
    pub undelegate: Prompt,
    pub claim_all_rewards: Prompt,
    pub claim_rewards_and_redelegate: Prompt,
    pub redelegate: Prompt,
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

    let delegate_prompt = Prompt {
        template: "delegate ${amount:number} lux to validator ${validator_name:string}"
            .to_string(),
        msg_fields: vec!["amount".to_string(), "validator_name".to_string()],
        query: Query {
            instructions: vec![
                QueryInstruction {
                    plane: "COSMOS".to_string(),
                    action: "COSMOS_QUERY".to_string(),
                    address: Binary::new(vec![]),
                    input: vec![Binary::from(
                        "/cosmos/staking/v1beta1/validators".as_bytes(),
                    )],
                },
            ],
        },
    };

    let undelegate_prompt = Prompt {
        template: "undelegate ${amount:number} lux from validator ${validator_name:string}".to_string(),
        msg_fields: vec![
            "amount".to_string(),
            "validator_name".to_string(),
        ],
        query: Query {
            instructions: vec![
                QueryInstruction {
                    plane: "COSMOS".to_string(),
                    action: "COSMOS_QUERY".to_string(),
                    address: Binary::new(vec![]),
                    input: vec![Binary::from(
                        "/cosmos/staking/v1beta1/delegations/${wallet}".as_bytes(),
                    )],
                },
                QueryInstruction {
                    plane: "COSMOS".to_string(),
                    action: "COSMOS_QUERY".to_string(),
                    address: Binary::new(vec![]),
                    input: vec![Binary::from(
                        "/cosmos/staking/v1beta1/validators".as_bytes(),
                    )],
                },
            ],
        },
    };

    
    
    let claim_all_rewards_prompt = Prompt {
        template: "claim all rewards from all validators".to_string(),
        msg_fields: vec![],
        query: Query {
            instructions: vec![
                QueryInstruction {
                    plane: "COSMOS".to_string(),
                    action: "COSMOS_QUERY".to_string(),
                    address: Binary::new(vec![]),
                    input: vec![Binary::from(
                        "/cosmos/distribution/v1beta1/delegators/${wallet}/rewards".as_bytes(),
                    )],
                }],
            },
        };
        
        
    let claim_rewards_and_redelegate_prompt = Prompt {
        template: "claim all rewards and delegate to same validators".to_string(),
        msg_fields: vec![],
        query: Query {
            instructions: vec![
                QueryInstruction {
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
            instructions: vec![
                QueryInstruction {
                    plane: "COSMOS".to_string(),
                    action: "COSMOS_QUERY".to_string(),
                    address: Binary::new(vec![]),
                    input: vec![Binary::from(
                        "/cosmos/distribution/v1beta1/delegators/${wallet}/rewards".as_bytes(),
                    )],
                },
                QueryInstruction {
                    plane: "COSMOS".to_string(),
                    action: "COSMOS_QUERY".to_string(),
                    address: Binary::new(vec![]),
                    input: vec![Binary::from(
                        "/cosmos/staking/v1beta1/validators".as_bytes(),
                    )],
                },
            ],
        },
    };

    // Constructing the group "Staking Solver"
    let group = Group {
        name: "Staking Solver".to_string(),
        prompts: Prompts {
            delegate: delegate_prompt,
            undelegate: undelegate_prompt,
            claim_all_rewards: claim_all_rewards_prompt,
            claim_rewards_and_redelegate: claim_rewards_and_redelegate_prompt,
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
