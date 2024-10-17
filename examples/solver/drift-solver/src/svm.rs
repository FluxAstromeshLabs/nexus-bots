use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, StdError, Uint64};
use sha2::{Digest, Sha256};
const PDA_MARKER: &[u8; 21] = b"ProgramDerivedAddress";

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

// === instruction utils ===
pub mod drift {
    use super::Instruction;

    fn create_intialize_user_ix(
        sender_svm_account: String,
    ) -> Instruction {
        
    }

    fn create_intialize_userstats_ix(
        sender_svm_account: String,
    ) -> Instruction {
        
    }

    fn create_deposit_ix(
        sender_svm_account: String,
    ) -> Instruction {
        
    }

    fn create_place_perp_order_ix(
        sender_svm_account: String,
    ) -> Instruction {
        
    }

    fn create_fill_perp_order_ix(
        sender_svm_account: String,
    ) -> Instruction {
        
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