use cosmwasm_std::{from_json, Binary, StdError, Uint64};
use serde::{Deserialize, Serialize};

pub mod raydium {
    use cosmwasm_std::{to_json_vec, Binary, StdError};

    use crate::{astromesh::Swap, FISInstruction};

    use super::{Instruction, InstructionAccount, MsgTransaction};

    const SPL_TOKEN_2022: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";
    const CPMM_PROGRAM_ID: &str = "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C";
    // const ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

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
            input_token_mint,
            output_token_mint,
            observer_state,
            CPMM_PROGRAM_ID.to_string(),
        ];

        let mut data_bz: Vec<u8> = vec![143, 190, 90, 218, 196, 30, 51, 222];
        data_bz.extend(amount_in.to_le_bytes());
        data_bz.extend(min_amount_out.to_le_bytes());

        MsgTransaction {
            // ty: "flux.svm.v1beta1.MsgTransaction".to_string(),
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
                        id_index: 1,
                        caller_index: 1,
                        callee_index: 1,
                        is_signer: false,
                        is_writable: false,
                    },
                    InstructionAccount {
                        id_index: 2,
                        caller_index: 2,
                        callee_index: 2,
                        is_signer: false,
                        is_writable: false,
                    },
                    InstructionAccount {
                        id_index: 3,
                        caller_index: 3,
                        callee_index: 3,
                        is_signer: false,
                        is_writable: true,
                    },
                    InstructionAccount {
                        id_index: 4,
                        caller_index: 4,
                        callee_index: 4,
                        is_signer: false,
                        is_writable: true,
                    },
                    InstructionAccount {
                        id_index: 5,
                        caller_index: 5,
                        callee_index: 5,
                        is_signer: false,
                        is_writable: true,
                    },
                    InstructionAccount {
                        id_index: 6,
                        caller_index: 6,
                        callee_index: 6,
                        is_signer: false,
                        is_writable: true,
                    },
                    InstructionAccount {
                        id_index: 7,
                        caller_index: 7,
                        callee_index: 7,
                        is_signer: false,
                        is_writable: true,
                    },
                    InstructionAccount {
                        id_index: 8,
                        caller_index: 8,
                        callee_index: 8,
                        is_signer: false,
                        is_writable: false,
                    },
                    InstructionAccount {
                        id_index: 8,
                        caller_index: 8,
                        callee_index: 8,
                        is_signer: false,
                        is_writable: false,
                    },
                    InstructionAccount {
                        id_index: 9,
                        caller_index: 9,
                        callee_index: 10,
                        is_signer: false,
                        is_writable: false,
                    },
                    InstructionAccount {
                        id_index: 10,
                        caller_index: 10,
                        callee_index: 11,
                        is_signer: false,
                        is_writable: false,
                    },
                    InstructionAccount {
                        id_index: 11,
                        caller_index: 11,
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

    pub fn compose_swap_fis(sender: String, swap: Swap) -> Result<FISInstruction, StdError> {
        // TODO: Return error instead of unwrapping
        let accounts = swap.raydium_accounts.unwrap();

        let msg = swap_base_input(
            sender,
            swap.input_amount.unwrap().i128() as u64,
            0,
            accounts.sender_svm_account,
            accounts.authority_account,
            accounts.amm_config_account,
            accounts.pool_state_account,
            accounts.input_token_account,
            accounts.output_token_account,
            accounts.input_vault,
            accounts.output_vault,
            swap.input_denom,
            swap.output_denom,
            accounts.observer_state,
        );
        Ok(FISInstruction {
            plane: "SVM".to_string(),
            action: "VM_INVOKE".to_string(),
            address: "".to_string(),
            msg: to_json_vec(&msg)?,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MsgTransaction {
    // #[serde(rename = "@type")]
    // pub ty: String,
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
pub struct Account {
    pub pubkey: Binary,
    pub owner: Binary,
    pub lamports: Uint64, // JSON cdc returns string (with quotes), standard u64 can't be parsed
    pub data: Binary,
    pub executable: bool,
    pub rent_epoch: Uint64,
}

impl Account {
    pub fn from_json_bytes(bz: &[u8]) -> Result<Self, StdError> {
        from_json(bz)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InstructionAccount {
    pub id_index: u32,
    pub caller_index: u32,
    pub callee_index: u32,
    pub is_signer: bool,
    pub is_writable: bool,
}

#[derive(Debug)]
pub struct Pubkey([u8; 32]);

impl Pubkey {
    pub fn to_string(&self) -> String {
        bs58::encode(self.0).into_string()
    }

    pub fn from_slice(bz: &[u8]) -> Result<Self, StdError> {
        if bz.len() != 32 {
            return Err(StdError::generic_err("pubkey must be 32 bytes"));
        }

        let mut pubkey: [u8; 32] = [0; 32];
        pubkey.copy_from_slice(bz);
        Ok(Self(pubkey))
    }

    pub fn from_string(s: String) -> Result<Self, StdError> {
        let bz = bs58::decode(s.as_str())
            .into_vec()
            .or_else(|e| Err(StdError::generic_err(e.to_string())))?;
        Pubkey::from_slice(bz.as_slice())
    }
}

// Simplified version of token account
#[derive(Debug)]
pub struct TokenAccount {
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
}

// "getrandom" dep

impl TokenAccount {
    pub fn unpack(bz: &[u8]) -> Result<TokenAccount, StdError> {
        if bz.len() < 72 {
            return Err(StdError::generic_err("token account size must >= 72 bytes"));
        }

        Ok(TokenAccount {
            mint: Pubkey::from_slice(&bz[0..32])?,
            owner: Pubkey::from_slice(&bz[32..64])?,
            amount: u64::from_le_bytes(bz[64..72].try_into().unwrap()), // we know for sure it's 8 bytes => unwrap() is safe
        })
    }
}

// spl-token library => static check fail
