use cosmwasm_std::{from_json, Binary, StdError, Uint64};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub mod raydium {
    use crate::astromesh::{FISInput, FISInstruction, Pool, Swap, ETH_DECIMAL_DIFF};
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{to_json_vec, Binary, Int128, Int256, StdError};
    use tiny_keccak::{Hasher, Keccak};

    use super::{
        get_denom, Account, Instruction, InstructionAccount, MsgTransaction, PoolState, Pubkey,
        TokenAccount,
    };

    pub const RAYDIUM: &str = "raydium";
    pub const SPL_TOKEN_2022: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";
    pub const CPMM_PROGRAM_ID: &str = "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C";
    pub const ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";
    pub const SYSTEM_PROGRAM_ID: &str = "11111111111111111111111111111111";
    pub const BPS: i128 = 1000000i128;

    #[derive(Clone)]
    pub struct PoolAccounts {
        pub authority_account: String,
        pub amm_config_account: String,
        pub pool_state_account: String,
        pub token0_mint: String,
        pub token1_mint: String,
        pub token0_vault: String,
        pub token1_vault: String, // can be calculated
        pub observer_state: String,
    }

    pub fn get_pool_accounts_by_name(pool_name: &String) -> Result<PoolAccounts, StdError> {
        match pool_name.as_str() {
            "btc-usdt" => Ok(PoolAccounts {
                authority_account: "GpMZbSM2GgvTKHJirzeGfMFoaZ8UR2X7F4v8vHTvxFbL".to_string(),
                amm_config_account: "D4FPEruKEHrG5TenZ2mpDGEfu1iUvTiqBxvpU8HLBvC2".to_string(),
                pool_state_account: "HUtjobntUDzrsq1k7xLM6SLzuZyUvr2U8skA8SUWevFd".to_string(),
                token0_mint: "ENyus6yS21v95sreLKcVEA5Wjcyh8jg6w4jBFHzJaPox".to_string(),
                token1_mint: "ErDYXZUZ9rpSSvdWvrsQwgh6K4BQeoY2CPyv1FeD1S9r".to_string(),
                token0_vault: "9U5Lpfmc6u1rCRAfzGe883KnK5Avm76zX4te6sexvCEk".to_string(),
                token1_vault: "UURmKznoUTh8Dt9wgyusq6u1ETuY8Zj79NFAtfQJ7HB".to_string(),
                observer_state: "FXqXrt2xDrxg7J5wdXrTbB2hCGajSzXLvwvc4x3Uw7i".to_string(),
            }),
            "eth-usdt" => Ok(PoolAccounts {
                authority_account: "GpMZbSM2GgvTKHJirzeGfMFoaZ8UR2X7F4v8vHTvxFbL".to_string(),
                amm_config_account: "D4FPEruKEHrG5TenZ2mpDGEfu1iUvTiqBxvpU8HLBvC2".to_string(),
                pool_state_account: "GASMVGvEguNjicG1UhaTiYDPib4geFQBXjtbqAG1HPLH".to_string(),
                token0_mint: "7Smiqjum5Xd7sZYysWXuS4Qbws6Y1rUKjcxudFJsLGJc".to_string(),
                token1_mint: "ErDYXZUZ9rpSSvdWvrsQwgh6K4BQeoY2CPyv1FeD1S9r".to_string(),
                token0_vault: "CP9w46ipnMBBQP2Nqg8DceobmnTFeb9Pri5W2RX1CiSV".to_string(),
                token1_vault: "DCJQyrGYeHWocMxpBBWCSJEgtMFZXgwMuXxZnkrHtuvW".to_string(),
                observer_state: "aLPmyw8Zs6kivaeaysiA1CXyhKCngeUW1deStmbn7ri".to_string(),
            }),
            "sol-usdt" => Ok(PoolAccounts {
                authority_account: "GpMZbSM2GgvTKHJirzeGfMFoaZ8UR2X7F4v8vHTvxFbL".to_string(),
                amm_config_account: "D4FPEruKEHrG5TenZ2mpDGEfu1iUvTiqBxvpU8HLBvC2".to_string(),
                pool_state_account: "F5h7xu4VdUdY3LRxCWo8Jv6HcdVK4tNsnEwdhBHvQA9K".to_string(),
                token0_mint: "1a5UtpbTcDiUPQcQ5tMSKQoLJXTzQRrjitQXxozn4ga".to_string(),
                token1_mint: "ErDYXZUZ9rpSSvdWvrsQwgh6K4BQeoY2CPyv1FeD1S9r".to_string(),
                token0_vault: "HNHWS8EqDH8GCW5XeL6dVirSRPcKEn5mZ7qUzvHWfizD".to_string(),
                token1_vault: "6DY4BxWgdoNG557vXUif4A6AdMSSrR7RH4uarfBW7vb5".to_string(),
                observer_state: "8rvsAHa9HztPWoioR8w6FR64VdS3TZCCmK52i1xCEJoF".to_string(),
            }),
            name => Err(StdError::generic_err(format!(
                "raydium pair not found: {}",
                name
            ))),
        }
    }

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
            SYSTEM_PROGRAM_ID.to_string(),
            ASSOCIATED_TOKEN_PROGRAM_ID.to_string(),
        ];

        // This instruction is idempotent, cost less fee when account exists
        let create_output_ata_ix = Instruction {
            program_index: vec![14],
            accounts: vec![
                InstructionAccount {
                    id_index: 0,
                    caller_index: 0,
                    callee_index: 0,
                    is_signer: true,
                    is_writable: true,
                },
                InstructionAccount {
                    id_index: 5,
                    caller_index: 5,
                    callee_index: 1,
                    is_signer: false,
                    is_writable: true,
                },
                InstructionAccount {
                    id_index: 0,
                    caller_index: 0,
                    callee_index: 0,
                    is_signer: false,
                    is_writable: false,
                },
                InstructionAccount {
                    id_index: 10,
                    caller_index: 10,
                    callee_index: 3,
                    is_signer: false,
                    is_writable: false,
                },
                InstructionAccount {
                    id_index: 13,
                    caller_index: 13,
                    callee_index: 4,
                    is_signer: false,
                    is_writable: false,
                },
                InstructionAccount {
                    id_index: 8,
                    caller_index: 8,
                    callee_index: 5,
                    is_signer: false,
                    is_writable: false,
                },
            ],
            data: Binary::new(vec![1]), // 1 = CreateIdempotent instruction
        };

        let mut data_bz: Vec<u8> = vec![143, 190, 90, 218, 196, 30, 51, 222]; // swap_base_input ix signature
        data_bz.extend(amount_in.to_le_bytes());
        data_bz.extend(min_amount_out.to_le_bytes());

        let swap_ix = Instruction {
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
        };

        MsgTransaction {
            signers: vec![sender],
            accounts,
            instructions: vec![create_output_ata_ix, swap_ix],
            compute_budget: 10_000_000,
        }
    }

    pub fn keccak256(input: &[u8]) -> [u8; 32] {
        let mut hash = Keccak::v256();
        hash.update(input);
        let mut output = [0u8; 32];
        hash.finalize(output.as_mut_slice());
        output
    }

    #[cw_serde]
    #[derive(Default)]
    pub struct RaydiumPool {
        pub dex_name: String,
        pub denom_plane: String,
        pub a: Int256,
        pub b: Int256,
        pub fee_rate: Int256,
        pub denom_a: String,
        pub denom_b: String,
    }

    impl RaydiumPool {
        pub fn new(pair: &str) -> Result<RaydiumPool, StdError> {
            // Fetch pool accounts using the provided pair name
            let pool_accounts = get_pool_accounts_by_name(&pair.to_string())?;

            // Create and return the RaydiumPool struct with amounts set to zero and denominations extracted from pair
            Ok(RaydiumPool {
                dex_name: RAYDIUM.to_string(),
                denom_plane: "SVM".to_string(),
                a: Int256::zero(),
                b: Int256::zero(),
                fee_rate: Int256::from(1000i128),
                denom_a: pool_accounts.token0_mint,
                denom_b: pool_accounts.token1_mint,
            })
        }

        pub fn from_fis(input: &FISInput) -> Result<Self, StdError> {
            let token_0_vault_account = Account::from_json_bytes(
                input
                    .data
                    .first()
                    .ok_or(StdError::generic_err("expected account 0"))?,
            )?;
            let token_1_vault_account = Account::from_json_bytes(
                input
                    .data
                    .get(1)
                    .ok_or(StdError::generic_err("expected account 1"))?,
            )?;
            let pool_state = Account::from_json_bytes(
                input
                    .data
                    .get(2)
                    .ok_or(StdError::generic_err("expected account 2"))?,
            )?;

            let mut token_0_info = TokenAccount::unpack(token_0_vault_account.data.as_slice())?;
            let mut token_1_info = TokenAccount::unpack(token_1_vault_account.data.as_slice())?;
            let mut pool_state_info = PoolState::unpack(pool_state.data.as_slice())?;

            // let (mut protocol_fees_token_0, mut protocol_fees_token_1, mut fund_fees_token_0, mut fund_fees_token_1)
            // TODO: more constraint as validate basic
            let (mut a, mut b) = (token_0_info.amount, token_1_info.amount);
            // we always swap from usdt so let it be the first
            if token_0_info.mint.to_string() != get_denom("usdt") {
                (a, b) = (b, a);
                (token_0_info, token_1_info) = (token_1_info, token_0_info);
                (
                    pool_state_info.protocol_fees_token_0,
                    pool_state_info.protocol_fees_token_1,
                ) = (
                    pool_state_info.protocol_fees_token_1,
                    pool_state_info.protocol_fees_token_0,
                );
                (
                    pool_state_info.fund_fees_token_0,
                    pool_state_info.fund_fees_token_1,
                ) = (
                    pool_state_info.fund_fees_token_1,
                    pool_state_info.fund_fees_token_0,
                );
            }

            a = a - (pool_state_info.protocol_fees_token_0 + pool_state_info.fund_fees_token_0);
            b = b - (pool_state_info.protocol_fees_token_1 + pool_state_info.fund_fees_token_1);

            // hack: ETH, it's 18 decimals on cosmos/wasm/evm => need to convert the eth amount from 9 decimals to 18 decimals
            // hardcode for now, should use denom link later on
            let mut decimal_multiplier = Int256::one();
            if &token_1_info.mint.to_string() == &get_denom("eth") {
                decimal_multiplier = Int256::from_i128(1_000_000_000i128);
            }

            Ok(Self {
                dex_name: RAYDIUM.to_string(),
                denom_plane: "SVM".to_string(),
                a: Int256::from_i128(a as i128),
                b: Int256::from_i128(b as i128) * decimal_multiplier,
                fee_rate: Int256::from(1000i128),
                denom_a: token_0_info.mint.to_string(),
                denom_b: token_1_info.mint.to_string(),
            })
        }
    }

    impl Pool for RaydiumPool {
        fn dex_name(&self) -> String {
            self.dex_name.clone()
        }

        fn denom_plane(&self) -> String {
            self.denom_plane.clone()
        }

        fn a(&self) -> Int256 {
            self.a
        }

        fn b(&self) -> Int256 {
            self.b
        }

        fn swap_output(&self, x: Int256, a_for_b: bool) -> (String, Int256) {
            let bps = Int256::from_i128(BPS);
            let mut x = x * (bps - self.fee_rate) / bps;
            let a = self.a;
            let b = self.b;

            if a_for_b {
                let denom = self.denom_b.clone();
                let mut output_amount = (b * x) / (a + x);

                // rounding down for ETH so that it could be transferred out
                if &self.denom_b == &get_denom("eth") {
                    let decimal_diff = Int256::from_i128(ETH_DECIMAL_DIFF as i128);
                    output_amount = (output_amount / decimal_diff) * decimal_diff;
                }
                (denom, output_amount)
            } else {
                // rounding down for ETH so that it match the transferred in amount
                if &self.denom_b == &get_denom("eth") {
                    let decimal_diff = Int256::from_i128(ETH_DECIMAL_DIFF as i128);
                    x = (x / decimal_diff) * decimal_diff;
                }

                (self.denom_a.clone(), (a * x) / (b + x))
            }
        }

        fn compose_swap_fis(&self, swap: &Swap) -> Result<Vec<FISInstruction>, StdError> {
            let accounts = get_pool_accounts_by_name(&swap.pool_name)?;
            let sender_svm_account = Pubkey::from_string(&swap.sender_svm)
                .map_err(|e| StdError::generic_err(format!("parse svm address err: {}", e)))?;
            let input_denom = get_denom(&swap.denom);
            let mut amount = swap.amount;
            if input_denom == get_denom("eth") {
                amount = amount / Int128::from(ETH_DECIMAL_DIFF as i128)
            }

            let (mut input_vault, mut output_vault) =
                (accounts.token0_vault, accounts.token1_vault);
            if &input_denom == &accounts.token1_mint {
                (input_vault, output_vault) = (output_vault, input_vault);
            }

            let output_denom = if input_denom == accounts.token0_mint {
                accounts.token1_mint
            } else {
                accounts.token0_mint
            };

            let input_denom_pk = Pubkey::from_string(&input_denom)?;
            let output_denom_pk = Pubkey::from_string(&output_denom)?;
            let token_program = Pubkey::from_string(&SPL_TOKEN_2022.to_string())?;
            let ata_program = Pubkey::from_string(&ASSOCIATED_TOKEN_PROGRAM_ID.to_string())?;

            let (input_token_account, _) = Pubkey::find_program_address(
                &[&sender_svm_account.0, &token_program.0, &input_denom_pk.0],
                &ata_program,
            )
            .unwrap();
            let (output_token_account, _) = Pubkey::find_program_address(
                &[&sender_svm_account.0, &token_program.0, &output_denom_pk.0],
                &ata_program,
            )
            .unwrap();

            let msg = swap_base_input(
                swap.sender.clone(),
                amount.i128() as u64,
                0,
                sender_svm_account.to_string(),
                // sender_svm_account.to_string(),
                accounts.authority_account,
                accounts.amm_config_account,
                accounts.pool_state_account,
                input_token_account.to_string(),
                output_token_account.to_string(),
                input_vault,
                output_vault,
                input_denom,
                output_denom,
                accounts.observer_state,
            );
            Ok(vec![FISInstruction {
                plane: "SVM".to_string(),
                action: "VM_INVOKE".to_string(),
                address: "".to_string(),
                msg: to_json_vec(&msg)?,
            }])
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

const PDA_MARKER: &[u8; 21] = b"ProgramDerivedAddress";

pub struct Hash(pub(crate) [u8; 32]);

#[derive(Clone, Default)]
pub struct Hasher {
    hasher: Sha256,
}

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

pub fn bytes_are_curve_point<T: AsRef<[u8]>>(_bytes: T) -> bool {
    curve25519_dalek::edwards::CompressedEdwardsY::from_slice(_bytes.as_ref())
        .unwrap()
        .decompress()
        .is_some()
}

#[derive(Debug)]
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

// Simplified version of token account
#[derive(Debug)]
pub struct TokenAccount {
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
}

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

pub fn get_denom(denom: &str) -> String {
    match denom {
        "btc" => "ENyus6yS21v95sreLKcVEA5Wjcyh8jg6w4jBFHzJaPox".to_string(),
        "eth" => "7Smiqjum5Xd7sZYysWXuS4Qbws6Y1rUKjcxudFJsLGJc".to_string(),
        "sol" => "1a5UtpbTcDiUPQcQ5tMSKQoLJXTzQRrjitQXxozn4ga".to_string(),
        "usdt" => "ErDYXZUZ9rpSSvdWvrsQwgh6K4BQeoY2CPyv1FeD1S9r".to_string(),
        _ => denom.to_string(),
    }
}

// Simplified version of poolstate
#[derive(Debug)]
pub struct PoolState {
    // additional information goes here
    pub protocol_fees_token_0: u64,
    pub protocol_fees_token_1: u64,
    pub fund_fees_token_0: u64,
    pub fund_fees_token_1: u64,
}

impl PoolState {
    pub const LEN: usize = 8 + 10 * 32 + 1 * 5 + 8 * 6 + 8 * 32;

    pub fn unpack(bz: &[u8]) -> Result<PoolState, StdError> {
        if bz.len() != Self::LEN {
            return Err(StdError::generic_err(format!(
                "pool state account must size must >= {} bytes, current len: {}",
                Self::LEN,
                bz.len()
            )));
        }

        let protocol_fees_token_0 = u64::from_le_bytes(bz[341..349].try_into().unwrap());
        let protocol_fees_token_1 = u64::from_le_bytes(bz[349..357].try_into().unwrap());
        let fund_fees_token_0 = u64::from_le_bytes(bz[357..365].try_into().unwrap());
        let fund_fees_token_1 = u64::from_le_bytes(bz[365..373].try_into().unwrap());

        Ok(PoolState {
            protocol_fees_token_0,
            protocol_fees_token_1,
            fund_fees_token_0,
            fund_fees_token_1,
        })
    }
}
