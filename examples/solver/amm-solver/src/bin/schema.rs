use amm_solver::{
    svm::{raydium, Pubkey},
    wasm::astroport,
};
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
    pub swap: Prompt,
    pub arbitrage: Prompt,
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

pub fn raydium_accounts(pool_name: String) -> Result<Vec<Binary>, StdError> {
    let pool = raydium::get_pool_accounts_by_name(&pool_name)?;
    Ok(vec![
        Binary::from(Pubkey::from_string(&pool.token0_vault)?.0),
        Binary::from(Pubkey::from_string(&pool.token1_vault)?.0),
        Binary::from(Pubkey::from_string(&pool.pool_state_account)?.0),
    ])
}

pub fn decode_bech32(bech32_addr: &String) -> Binary {
    Binary::from(bech32::decode(bech32_addr).unwrap().1)
}

fn main() {
    let arbitrage_instructions = vec![
        QueryInstruction {
            plane: "WASM".to_string(),
            action: "VM_QUERY".to_string(),
            address: decode_bech32(
                &astroport::get_pool_meta_by_name(&"btc-usdt".to_string())
                    .unwrap()
                    .contract,
            ),
            input: vec![Binary::new(r#"{"pool":{}}"#.as_bytes().to_vec())],
        },
        QueryInstruction {
            plane: "SVM".to_string(),
            action: "VM_QUERY".to_string(),
            address: Binary::new(vec![]),
            input: raydium_accounts("btc-usdt".to_string()).unwrap(),
        },
        QueryInstruction {
            plane: "WASM".to_string(),
            action: "VM_QUERY".to_string(),
            address: decode_bech32(
                &astroport::get_pool_meta_by_name(&"eth-usdt".to_string())
                    .unwrap()
                    .contract,
            ),
            input: vec![Binary::new(r#"{"pool":{}}"#.as_bytes().to_vec())],
        },
        QueryInstruction {
            plane: "SVM".to_string(),
            action: "VM_QUERY".to_string(),
            address: Binary::new(vec![]),
            input: raydium_accounts("eth-usdt".to_string()).unwrap(),
        },
        QueryInstruction {
            plane: "WASM".to_string(),
            action: "VM_QUERY".to_string(),
            address: decode_bech32(
                &astroport::get_pool_meta_by_name(&"sol-usdt".to_string())
                    .unwrap()
                    .contract,
            ),
            input: vec![Binary::new(r#"{"pool":{}}"#.as_bytes().to_vec())],
        },
        QueryInstruction {
            plane: "SVM".to_string(),
            action: "VM_QUERY".to_string(),
            address: Binary::new(vec![]),
            input: raydium_accounts("sol-usdt".to_string()).unwrap(),
        },
        QueryInstruction {
            plane: "COSMOS".to_string(),
            action: "COSMOS_QUERY".to_string(),
            address: Binary::new(vec![]),
            input: vec![Binary::from(
                "/flux/svm/v1beta1/account_link/cosmos/${wallet}".as_bytes(),
            )],
        },
    ];

    // Manually constructing the "swap" and "arbitrage" prompts
    let swap_prompt = Prompt {
        template:
            "swap ${amount:number} ${src_denom:string} to ${dst_denom:string} on ${dex_name:string}"
                .to_string(),
        msg_fields: vec![
            "amount".to_string(),
            "src_denom".to_string(),
            "dst_denom".to_string(),
            "dex_name".to_string(),
        ],
        query: Query {
            instructions: vec![],
        },
    };

    let arbitrage_prompt = Prompt {
        template: "arbitrage ${amount:number} USDT on pair ${pair:string} with minimum profit = ${min_profit:number} USDT".to_string(),
        msg_fields: vec![
            "amount".to_string(),
            "pair".to_string(),
            "min_profit".to_string(),
        ],
        query: Query {
            instructions: arbitrage_instructions,
        },
    };

    // Constructing the group "AMM Solver"
    let group = Group {
        name: "AMM Solver".to_string(),
        prompts: Prompts {
            swap: swap_prompt,
            arbitrage: arbitrage_prompt,
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
