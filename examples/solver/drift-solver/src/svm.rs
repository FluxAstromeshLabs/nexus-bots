use std::collections::BTreeMap;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, StdError, Uint64, Deps};
use sha2::{Digest, Sha256};
const PDA_MARKER: &[u8; 21] = b"ProgramDerivedAddress";
pub const SPL_TOKEN2022_PROGRAM_ID: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";
pub const SYSTEM_PROGRAM_ID: &str = "11111111111111111111111111111111";
pub const SYS_VAR_RENT_ID: &str = "SysvarRent111111111111111111111111111111111";
pub const MINT: &str = "C3xXmrQWWnTmYABa8YTKrYU5jkonkTwz1qQCJbVX3mQh";
pub const ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";
#[cw_serde]
pub struct Link {
    pub cosmos_addr: String,
    pub svm_addr: String,
    pub height: Uint64,
}

#[cw_serde]
pub struct AccountLink {
    pub link: Link,
}

#[cw_serde]
pub struct MsgTransaction {
    /// Sender is the address of the actor that signed the message
    pub signers: Vec<String>,
    /// Accounts are the cosmos addresses that sign this message
    pub accounts: Vec<String>,
    /// Instructions are the instructions for the transaction
    pub instructions: Vec<Instruction>,
    /// ComputeBudget is the budget for computation
    pub compute_budget: u64,
}

#[cw_serde]
pub struct InstructionAccount {
    pub id_index: u32,
    pub caller_index: u32,
    pub callee_index: u32,
    pub is_signer: bool,
    pub is_writable: bool,
}

#[cw_serde]
pub struct Instruction {
    /// ProgramIndex is a list of program indices
    pub program_index: Vec<u32>,
    /// Accounts are the accounts involved in the instruction
    pub accounts: Vec<InstructionAccount>,
    /// Data is the data for the instruction
    pub data: Binary,
}

#[derive(Clone, Default, Debug)]
pub struct InstructionAccountMeta {
    pub pubkey: String,
    pub is_signer: bool,
    pub is_writable: bool,
}

#[derive(Clone, Default, Debug)]
pub struct InstructionMeta {
    pub program_id: String,
    pub account_meta: Vec<InstructionAccountMeta>,
    pub data: Binary,
}

pub struct TransactionBuilder {
    instructions: Vec<InstructionMeta>,
}

impl TransactionBuilder {
    pub fn new() -> TransactionBuilder {
        TransactionBuilder {
            instructions: vec![],
        }
    }

    pub fn add_instruction(&mut self, ix: InstructionMeta) -> &mut Self {
        self.instructions.push(ix);
        self
    }

    pub fn build(&self, cosmos_signers: Vec<String>, compute_budget: u64) -> MsgTransaction {
        // Collect unique accounts and assign indices using BTreeMap
        let mut account_map: BTreeMap<String, u32> = BTreeMap::new();
        let mut accounts: Vec<String> = Vec::new();
        let mut current_index: u32 = 0;

        for ix in &self.instructions {
            if !account_map.contains_key(&ix.program_id) {
                account_map.insert(ix.program_id.clone(), current_index);
                accounts.push(ix.program_id.clone());
                current_index += 1;
            }

            for meta in &ix.account_meta {
                if !account_map.contains_key(&meta.pubkey) {
                    account_map.insert(meta.pubkey.clone(), current_index);
                    accounts.push(meta.pubkey.clone());
                    current_index += 1;
                }
            }
        }

        // Transform instructions meta into instruction
        let mut instructions: Vec<Instruction> = Vec::new();

        for ix in &self.instructions {
            // Get program index
            let program_idx = *account_map
                .get(&ix.program_id)
                .expect("Program ID not found");

            // Transform account_meta to InstructionAccount
            let mut instruction_accounts: Vec<InstructionAccount> = Vec::new();
            let mut instruction_acc_map: BTreeMap<String, u32> = BTreeMap::new();
            for (i, meta) in ix.account_meta.iter().enumerate() {
                let callee_index = match instruction_acc_map.get(&meta.pubkey) {
                    Some(index) => *index,
                    None => {
                        instruction_acc_map.insert(meta.pubkey.clone(), i as u32);
                        i as u32
                    },
                };

                let id_index = account_map.get(&meta.pubkey).unwrap();
                instruction_accounts.push(InstructionAccount {
                    id_index: *id_index,
                    caller_index: *id_index,
                    callee_index: callee_index,
                    is_signer: meta.is_signer,
                    is_writable: meta.is_writable,
                });
            }

            // Create instructions
            instructions.push(Instruction {
                program_index: vec![program_idx],
                accounts: instruction_accounts,
                data: ix.data.clone(),
            });
        }

        // Assemble MsgTransaction
        MsgTransaction {
            signers: cosmos_signers,
            accounts,
            instructions,
            compute_budget,
        }
    }
}

// === crypto utils ===

#[derive(Debug)]
pub struct Pubkey(pub [u8; 32]);

pub enum PubkeyError {
    MaxSeedLengthExceeded,
    InvalidSeeds,
    IllegalOwner,
}

#[derive(Clone, Default)]
pub struct Hasher {
    hasher: Sha256,
}

pub struct Hash(pub(crate) [u8; 32]);

impl Hasher {
    pub fn hash(&mut self, val: &[u8]) {
        self.hasher.update(val);
    }
    pub fn hashv(&mut self, vals: &[&[u8]]) {
        for val in vals {
            self.hash(val);
        }
    }
    pub fn result(self) -> Hash {
        Hash(self.hasher.finalize().into())
    }
}

impl Pubkey {
    pub fn to_string(&self) -> String {
        bs58::encode(self.0).into_string()
    }

    pub fn from_slice(bz: &[u8]) -> Result<Self, StdError> {
        if bz.len() != 32 {
            return Err(StdError::generic_err(format!(
                "pubkey must be 32 bytes: {}",
                bz.len()
            )));
        }

        let mut pubkey: [u8; 32] = [0; 32];
        pubkey.copy_from_slice(bz);
        Ok(Self(pubkey))
    }

    pub fn from_string(s: &String) -> Result<Self, StdError> {
        let bz = bs58::decode(s.as_str())
            .into_vec()
            .map_err(|e| StdError::generic_err(e.to_string()))?;
        Pubkey::from_slice(bz.as_slice())
            .map_err(|e| StdError::generic_err(format!("pubkey from string: {}: {}", s, e)))
    }

    pub fn find_program_address(seeds: &[&[u8]], program_id: &Pubkey) -> Option<(Pubkey, u8)> {
        let mut bump_seed = [u8::MAX];
        for _ in 0..u8::MAX {
            {
                let mut seeds_with_bump = seeds.to_vec();
                seeds_with_bump.push(&bump_seed);
                match Self::create_program_address(&seeds_with_bump, program_id) {
                    Ok(address) => return Some((address, bump_seed[0])),
                    Err(PubkeyError::InvalidSeeds) => (),
                    _ => break,
                }
            }
            bump_seed[0] -= 1;
        }
        None
    }

    pub fn create_program_address(
        seeds: &[&[u8]],
        program_id: &Pubkey,
    ) -> Result<Pubkey, PubkeyError> {
        if seeds.len() > 255 {
            return Err(PubkeyError::MaxSeedLengthExceeded);
        }

        for seed in seeds.iter() {
            if seed.len() > 32 {
                return Err(PubkeyError::MaxSeedLengthExceeded);
            }
        }

        let mut hasher = Hasher::default();
        for seed in seeds.iter() {
            hasher.hash(seed);
        }
        hasher.hashv(&[program_id.0.as_slice(), PDA_MARKER]);
        let hash = hasher.result();

        if bytes_are_curve_point(hash.0) {
            return Err(PubkeyError::InvalidSeeds);
        }

        Ok(Pubkey::from_slice(hash.0.as_slice()).unwrap())
    }
}

pub fn bytes_are_curve_point<T: AsRef<[u8]>>(_bytes: T) -> bool {
    curve25519_dalek::edwards::CompressedEdwardsY::from_slice(_bytes.as_ref())
        .unwrap()
        .decompress()
        .is_some()
}
