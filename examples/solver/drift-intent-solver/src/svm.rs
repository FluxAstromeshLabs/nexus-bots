use cosmwasm_std::{from_json, Binary, StdError, Uint64};
use serde::{Deserialize, Serialize};
use bs58;
use std::convert::TryInto;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Account {
    pub pubkey: Binary,
    pub owner: Binary,
    pub lamports: Uint64, // JSON decoding returns string, so we use Uint64.
    pub data: Binary,
    pub executable: bool,
    pub rent_epoch: Uint64,
}

impl Account {
    pub fn from_json_bytes(bz: &[u8]) -> Result<Self, StdError> {
        from_json(bz)
    }
}

#[derive(Debug)]
pub struct AccountMeta {
    pub pubkey: Binary,
    pub is_writable: bool,
    pub is_signer: bool,
}

#[derive(Debug, Clone)]
pub struct Pubkey(pub [u8; 32]);

pub enum PubkeyError {
    MaxSeedLengthExceeded,
    InvalidSeeds,
    IllegalOwner,
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

    pub fn from_string(s: &str) -> Result<Self, StdError> {
        let bz = bs58::decode(s)
            .into_vec()
            .map_err(|e| StdError::generic_err(e.to_string()))?;
        Pubkey::from_slice(&bz)
            .map_err(|e| StdError::generic_err(format!("pubkey from string: {}: {}", s, e)))
    }

    pub fn find_program_address(seeds: &[&[u8]], program_id: &Pubkey) -> Option<(Pubkey, u8)> {
        let mut bump_seed = [u8::MAX];
        for _ in 0..u8::MAX {
            let mut seeds_with_bump = seeds.to_vec();
            seeds_with_bump.push(&bump_seed);
            match Self::create_program_address(&seeds_with_bump, program_id) {
                Ok(address) => return Some((address, bump_seed[0])),
                Err(PubkeyError::InvalidSeeds) => (),
                _ => break,
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

        for seed in seeds {
            if seed.len() > 32 {
                return Err(PubkeyError::MaxSeedLengthExceeded);
            }
        }

        let mut hasher = sha2::Sha256::default();
        for seed in seeds {
            hasher.update(seed);
        }
        hasher.update(&program_id.0);
        hasher.update(&[1]); // PDA marker
        let hash = hasher.finalize();

        if bytes_are_curve_point(&hash) {
            return Err(PubkeyError::InvalidSeeds);
        }

        Ok(Pubkey::from_slice(&hash).unwrap())
    }
}

#[derive(Debug)]
pub struct TokenAccount {
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
}

impl TokenAccount {
    pub fn unpack(bz: &[u8]) -> Result<Self, StdError> {
        if bz.len() < 72 {
            return Err(StdError::generic_err("token account size must >= 72 bytes"));
        }

        Ok(Self {
            mint: Pubkey::from_slice(&bz[0..32])?,
            owner: Pubkey::from_slice(&bz[32..64])?,
            amount: u64::from_le_bytes(bz[64..72].try_into().unwrap()),
        })
    }
}

pub fn get_denom(denom: &str) -> String {
    match denom {
        "btc" => "5ouhhEqV1L9gj3qTg3nQhkYuAuw72suktwJ4PvGo32SP".to_string(),
        "eth" => "4SgGYkKAF4k3uAmkKaqMFnAuZkAhyzUuabRpHhssyW9B".to_string(),
        "sol" => "CPozhCGVaGAcPVkxERsUYat4b7NKT9QeAR9KjNH4JpDG".to_string(),
        "usdt" => "C3xXmrQWWnTmYABa8YTKrYU5jkonkTwz1qQCJbVX3mQh".to_string(),
        _ => denom.to_string(),
    }
}
