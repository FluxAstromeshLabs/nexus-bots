use cosmwasm_std::Binary;
use serde::{Deserialize, Serialize};

/*
{
  "sender": "lux1cml96vmptgw99syqrrz8az79xer2pcgp209sv4",
  "accounts": [
    "GonQpn9zzCF2rD521AiYg1RFpC4aFEzJ8RwC9XDi54L6",
    "3LLwUBAjxx3sueJex7tjMCDRsTXnWdifmH3pmKcLs6ft",
    "DbWxCi22jGDa7cJDmijRHiYHac2TFbmzUMCnxd8nVp4A",
    "TL89ZcvzEAZhbAXyXB83vhwVNDafPSRHCVvApL1trPT",
    "8a9wZcBo39FnRnBJXfd77VnSBaw3bjjQxctVkK9tvM3s",
    "HWXu89yPn2VgXLVmUExFaNwQyE2KS5CrBd86u1XnDhk",
    "G7PDs2GToeRC4jkYh1a7hBqtU6Ss2R9K3KF7bMpqcW7W",
    "GpMZbSM2GgvTKHJirzeGfMFoaZ8UR2X7F4v8vHTvxFbL",
    "D4FPEruKEHrG5TenZ2mpDGEfu1iUvTiqBxvpU8HLBvC2",
    "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
    "5GqSuujoC3QkLypS3RHibBYUK4pKJLgGqMyf1Mkt4ghb",
    "4oFvCXEirbLCqK3i4aBmrgmt18Hf7KwBdjjfpUpEUMkA",
    "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C"
  ],
  "instructions": [
    {
      "program_index": [
        12
      ],
      "accounts": [
        {
          "id_index": 0,
          "caller_index": 0,
          "callee_index": 0,
          "is_signer": true,
          "is_writable": true
        },
        {
          "id_index": 7,
          "caller_index": 7,
          "callee_index": 1,
          "is_signer": false,
          "is_writable": false
        },
        {
          "id_index": 8,
          "caller_index": 8,
          "callee_index": 2,
          "is_signer": false,
          "is_writable": false
        },
        {
          "id_index": 1,
          "caller_index": 1,
          "callee_index": 3,
          "is_signer": false,
          "is_writable": true
        },
        {
          "id_index": 2,
          "caller_index": 2,
          "callee_index": 4,
          "is_signer": false,
          "is_writable": true
        },
        {
          "id_index": 3,
          "caller_index": 3,
          "callee_index": 5,
          "is_signer": false,
          "is_writable": true
        },
        {
          "id_index": 4,
          "caller_index": 4,
          "callee_index": 6,
          "is_signer": false,
          "is_writable": true
        },
        {
          "id_index": 5,
          "caller_index": 5,
          "callee_index": 7,
          "is_signer": false,
          "is_writable": true
        },
        {
          "id_index": 9,
          "caller_index": 9,
          "callee_index": 8,
          "is_signer": false,
          "is_writable": false
        },
        {
          "id_index": 9,
          "caller_index": 9,
          "callee_index": 8,
          "is_signer": false,
          "is_writable": false
        },
        {
          "id_index": 10,
          "caller_index": 10,
          "callee_index": 10,
          "is_signer": false,
          "is_writable": false
        },
        {
          "id_index": 11,
          "caller_index": 11,
          "callee_index": 11,
          "is_signer": false,
          "is_writable": false
        },
        {
          "id_index": 6,
          "caller_index": 6,
          "callee_index": 12,
          "is_signer": false,
          "is_writable": true
        }
      ],
      "data": "j75a2sQeM94A4fUFAAAAABfJgRMAAAAA"
    }
  ],
  "compute_budget": 10000000
}
*/
pub mod raydium {
    use cosmwasm_std::{Binary, Uint128};

    use super::{Instruction, InstructionAccount, MsgTransaction};

    const SPL_TOKEN_2022: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";
    const CPMM_PROGRAM_ID: &str = "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C";

    pub fn swap_base_input(
        sender: String,
        amount_in: u64,
        min_amount_out: u64,
        sender_svm_account: String,
        authority_account: String,
        amm_config_account: String,
        pool_state_account: String,
        input_token_account: String,
        output_token_account: String,
        input_vault: String,
        output_vault: String,
        input_token_mint: String,
        output_token_mint: String,
        observer_state: String,
    ) -> MsgTransaction {
        let accounts = vec![
            sender_svm_account,
            authority_account,
            amm_config_account,
            pool_state_account,
            input_token_account,
            output_token_account,
            input_vault,
            output_vault,
            SPL_TOKEN_2022.to_string(),
            SPL_TOKEN_2022.to_string(),
            input_token_mint,
            output_token_mint,
            observer_state,
            CPMM_PROGRAM_ID.to_string(),
        ];

        let mut data_bz: Vec<u8> = vec![143, 190, 90, 218, 196, 30, 51, 222];
        data_bz.extend(amount_in.to_le_bytes());
        data_bz.extend(min_amount_out.to_le_bytes());

        MsgTransaction {
            ty: "flux.svm.v1beta1.MsgTransaction".to_string(),
            sender,
            accounts,
            instructions: vec![Instruction {
                program_index: vec![12],
                accounts: vec![
                    InstructionAccount {
                        id_index: 0,
                        caller_index: 0,
                        callee_index: 0,
                        is_signer: true,
                        is_writable: true,
                    },
                    InstructionAccount {
                        id_index: 7,
                        caller_index: 7,
                        callee_index: 1,
                        is_signer: false,
                        is_writable: false,
                    },
                    InstructionAccount {
                        id_index: 8,
                        caller_index: 8,
                        callee_index: 2,
                        is_signer: false,
                        is_writable: false,
                    },
                    InstructionAccount {
                        id_index: 1,
                        caller_index: 1,
                        callee_index: 3,
                        is_signer: false,
                        is_writable: true,
                    },
                    InstructionAccount {
                        id_index: 2,
                        caller_index: 2,
                        callee_index: 4,
                        is_signer: false,
                        is_writable: true,
                    },
                    InstructionAccount {
                        id_index: 3,
                        caller_index: 3,
                        callee_index: 5,
                        is_signer: false,
                        is_writable: true,
                    },
                    InstructionAccount {
                        id_index: 4,
                        caller_index: 4,
                        callee_index: 6,
                        is_signer: false,
                        is_writable: true,
                    },
                    InstructionAccount {
                        id_index: 5,
                        caller_index: 5,
                        callee_index: 7,
                        is_signer: false,
                        is_writable: true,
                    },
                    InstructionAccount {
                        id_index: 9,
                        caller_index: 9,
                        callee_index: 8,
                        is_signer: false,
                        is_writable: false,
                    },
                    InstructionAccount {
                        id_index: 9,
                        caller_index: 9,
                        callee_index: 8,
                        is_signer: false,
                        is_writable: false,
                    },
                    InstructionAccount {
                        id_index: 10,
                        caller_index: 10,
                        callee_index: 10,
                        is_signer: false,
                        is_writable: false,
                    },
                    InstructionAccount {
                        id_index: 11,
                        caller_index: 11,
                        callee_index: 11,
                        is_signer: false,
                        is_writable: false,
                    },
                    InstructionAccount {
                        id_index: 6,
                        caller_index: 6,
                        callee_index: 12,
                        is_signer: false,
                        is_writable: true,
                    },
                ],
                data: Binary::from(data_bz),
            }],
            compute_budget: 10_000_000,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MsgTransaction {
    #[serde(rename = "@type")]
    pub ty: String,
    /// Sender is the address of the actor that signed the message
    pub sender: String,
    /// Accounts are the cosmos addresses that sign this message
    pub accounts: Vec<String>,
    /// Instructions are the instructions for the transaction
    pub instructions: Vec<Instruction>,
    /// ComputeBudget is the budget for computation
    pub compute_budget: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Instruction {
    /// ProgramIndex is a list of program indices
    pub program_index: Vec<u32>,
    /// Accounts are the accounts involved in the instruction
    pub accounts: Vec<InstructionAccount>,
    /// Data is the data for the instruction
    pub data: Binary,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InstructionAccount {
    /// IdIndex is the index of the account ID
    pub id_index: u32,
    /// CallerIndex is the index of the caller account
    pub caller_index: u32,
    /// CalleeIndex is the index of the callee account
    pub callee_index: u32,
    /// IsSigner indicates if the account is a signer
    pub is_signer: bool,
    /// IsWritable indicates if the account is writable
    pub is_writable: bool,
}
