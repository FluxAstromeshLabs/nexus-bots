use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, StdError, Uint64};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
const PDA_MARKER: &[u8; 21] = b"ProgramDerivedAddress";
pub const SPL_TOKEN2022_PROGRAM_ID: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";
pub const SPL_TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
pub const SYSTEM_PROGRAM_ID: &str = "11111111111111111111111111111111";
pub const SYS_VAR_RENT_ID: &str = "SysvarRent111111111111111111111111111111111";
pub const MINT: &str = "C3xXmrQWWnTmYABa8YTKrYU5jkonkTwz1qQCJbVX3mQh";
pub const ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";
pub const AMM_CONFIG_ACCOUNT: &str = "EHR3a7vLxBREzXic1rp7tyPPen6wy8VzdnYfKKRDXJG9";
pub const AUTHORITY_ACCOUNT: &str = "3NTS4CmziURYZJ1JywCaCF4urzVbhL6kxNLbpuLzaaR7";
pub const POOL_FEE_RECEIVER_ACCOUNT: &str = "TODO: FILL";

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
pub struct Account {
    pub pubkey: Binary,
    pub owner: Binary,
    pub lamports: Uint64, // JSON cdc returns string (with quotes), standard u64 can't be parsed
    pub data: Binary,
    pub executable: bool,
    pub rent_epoch: Uint64,
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

    pub fn add_instructions(&mut self, ixs: Vec<InstructionMeta>) -> &mut Self {
        self.instructions.extend(ixs);
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
                    }
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
#[derive(Debug, Clone, PartialEq, Eq, Default)]
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

pub mod raydium {
    use cosmwasm_std::{to_json_vec, Binary, HexBinary, Uint128};

    use crate::astromesh::{self, FISInstruction, PoolManager, ACTION_VM_INVOKE, PLANE_SVM};

    use super::{
        InstructionAccountMeta, InstructionMeta, Pubkey, TransactionBuilder, AMM_CONFIG_ACCOUNT,
        AUTHORITY_ACCOUNT, POOL_FEE_RECEIVER_ACCOUNT, SPL_TOKEN2022_PROGRAM_ID,
        SPL_TOKEN_PROGRAM_ID, SYS_VAR_RENT_ID,
    };

    pub const SPL_TOKEN_2022: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";
    pub const CPMM_PROGRAM_ID: &str = "6W19gt519Ruyw3s4BiKtQXvxETzPbptjgfgB5gMgrfAf";
    pub const ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";
    pub const SYSTEM_PROGRAM_ID: &str = "11111111111111111111111111111111";

    pub struct InitializeInstruction {
        pub init_amount0: u64,
        pub init_amount1: u64,
        pub open_time: u64,
        pub account_metas: Vec<InstructionAccountMeta>,
    }

    impl InitializeInstruction {
        pub fn new(
            init_amount0: u64,
            init_amount1: u64,
            open_time: u64,
            account_metas: Vec<InstructionAccountMeta>,
        ) -> Self {
            InitializeInstruction {
                init_amount0,
                init_amount1,
                open_time,
                account_metas,
            }
        }

        pub fn build_instruction(&self) -> InstructionMeta {
            let data = self.encode_data();

            InstructionMeta {
                program_id: CPMM_PROGRAM_ID.to_string(),
                account_meta: self.account_metas.clone(),
                data,
            }
        }

        fn encode_data(&self) -> Binary {
            let mut data: Vec<u8> = Vec::from(vec![175, 175, 109, 31, 13, 152, 155, 237]);

            data.extend(&self.init_amount0.to_le_bytes());
            data.extend(&self.init_amount1.to_le_bytes());
            data.extend(&self.open_time.to_le_bytes());

            Binary::from(data)
        }
    }

    pub fn create_initialize_instruction(
        // Parameters:
        init_amount0: u64,
        init_amount1: u64,
        open_time: u64,
        // Accounts:
        creator: String,
        amm_config: String,
        authority: String,
        pool_state: String,
        token0_mint: String,
        token1_mint: String,
        lp_mint: String,
        creator_token0: String,
        creator_token1: String,
        creator_lp_token: String,
        token0_vault: String,
        token1_vault: String,
        create_pool_fee: String,
        observation_state: String,
        token_program: String,
        token0_program: String,
        token1_program: String,
        associated_token_program: String,
        system_program: String,
        rent: String,
    ) -> InstructionMeta {
        let account_metas = vec![
            InstructionAccountMeta {
                pubkey: creator,
                is_signer: true,
                is_writable: true,
            },
            InstructionAccountMeta {
                pubkey: amm_config,
                is_signer: false,
                is_writable: false,
            },
            InstructionAccountMeta {
                pubkey: authority,
                is_signer: false,
                is_writable: false,
            },
            InstructionAccountMeta {
                pubkey: pool_state,
                is_signer: false,
                is_writable: true,
            },
            InstructionAccountMeta {
                pubkey: token0_mint,
                is_signer: false,
                is_writable: false,
            },
            InstructionAccountMeta {
                pubkey: token1_mint,
                is_signer: false,
                is_writable: false,
            },
            InstructionAccountMeta {
                pubkey: lp_mint,
                is_signer: false,
                is_writable: true,
            },
            InstructionAccountMeta {
                pubkey: creator_token0,
                is_signer: false,
                is_writable: true,
            },
            InstructionAccountMeta {
                pubkey: creator_token1,
                is_signer: false,
                is_writable: true,
            },
            InstructionAccountMeta {
                pubkey: creator_lp_token,
                is_signer: false,
                is_writable: true,
            },
            InstructionAccountMeta {
                pubkey: token0_vault,
                is_signer: false,
                is_writable: true,
            },
            InstructionAccountMeta {
                pubkey: token1_vault,
                is_signer: false,
                is_writable: true,
            },
            InstructionAccountMeta {
                pubkey: create_pool_fee,
                is_signer: false,
                is_writable: true,
            },
            InstructionAccountMeta {
                pubkey: observation_state,
                is_signer: false,
                is_writable: true,
            },
            InstructionAccountMeta {
                pubkey: token_program,
                is_signer: false,
                is_writable: false,
            },
            InstructionAccountMeta {
                pubkey: token0_program,
                is_signer: false,
                is_writable: false,
            },
            InstructionAccountMeta {
                pubkey: token1_program,
                is_signer: false,
                is_writable: false,
            },
            InstructionAccountMeta {
                pubkey: associated_token_program,
                is_signer: false,
                is_writable: false,
            },
            InstructionAccountMeta {
                pubkey: system_program,
                is_signer: false,
                is_writable: false,
            },
            InstructionAccountMeta {
                pubkey: rent,
                is_signer: false,
                is_writable: false,
            },
        ];

        let instruction = InitializeInstruction {
            init_amount0,
            init_amount1,
            open_time,
            account_metas,
        };

        instruction.build_instruction()
    }

    pub struct Raydium {
        pub svm_creator: String,
        pub open_time: u64,
    }

    fn must_find_ata(
        wallet: &Pubkey,
        token_program: &Pubkey,
        mint: &Pubkey,
        ata_program: &Pubkey,
    ) -> Pubkey {
        let (ata, _) =
            Pubkey::find_program_address(&[&wallet.0, &token_program.0, &mint.0], ata_program)
                .unwrap();
        ata
    }

    impl PoolManager for Raydium {
        fn create_pool_with_initial_liquidity(
            &self,
            sender: String,
            denom_0: String,
            amount_0: Uint128,
            denom_1: String,
            amount_1: Uint128,
        ) -> Vec<FISInstruction> {
            let raydium_swap_program = Pubkey::from_string(&CPMM_PROGRAM_ID.to_string()).unwrap();
            let amm_config = Pubkey::from_string(&AMM_CONFIG_ACCOUNT.to_string()).unwrap();

            let sender_svm_bz = Pubkey::from_string(&self.svm_creator).unwrap();
            let denom_0_bz = HexBinary::from_hex(&denom_0).unwrap();
            let denom_1_bz = HexBinary::from_hex(&denom_1).unwrap();
            let spl_token_2022_program = Pubkey::from_string(&SPL_TOKEN_2022.to_string()).unwrap();
            let ata_program =
                Pubkey::from_string(&ASSOCIATED_TOKEN_PROGRAM_ID.to_string()).unwrap();

            // Find the pool state account
            let (pool_state_account, _) = Pubkey::find_program_address(
                &[b"pool", &amm_config.0, &denom_0_bz, &denom_1_bz],
                &raydium_swap_program,
            )
            .unwrap();

            let (lp_mint, _) = Pubkey::find_program_address(
                &[b"pool_lp_mint", &pool_state_account.0],
                &raydium_swap_program,
            )
            .unwrap();

            let creator_token0_ata = must_find_ata(
                &sender_svm_bz,
                &spl_token_2022_program, // Replace with Token2022 program ID
                &Pubkey::from_slice(denom_0_bz.as_slice()).unwrap(),
                &ata_program, // Replace with ATA program ID
            );

            let creator_token1_ata = must_find_ata(
                &sender_svm_bz,
                &spl_token_2022_program, // Replace with Token2022 program ID
                &Pubkey::from_slice(denom_1_bz.as_slice()).unwrap(),
                &ata_program, // Replace with ATA program ID
            );

            let creator_lp_ata = must_find_ata(
                &sender_svm_bz,
                &spl_token_2022_program, // Replace with Token2022 program ID
                &lp_mint,
                &ata_program, // Replace with ATA program ID
            );

            let (token0_vault, _) = Pubkey::find_program_address(
                &[b"pool_vault", &pool_state_account.0, &denom_0_bz],
                &raydium_swap_program,
            )
            .unwrap();

            let (token1_vault, _) = Pubkey::find_program_address(
                &[b"pool_vault", &pool_state_account.0, &denom_1_bz],
                &raydium_swap_program,
            )
            .unwrap();

            // find the oracle observer account
            let (oracle_observer_state, _) = Pubkey::find_program_address(
                &[b"observation", &pool_state_account.0],
                &raydium_swap_program,
            )
            .unwrap();

            let initialize_pool = create_initialize_instruction(
                amount_0.u128() as u64,
                amount_1.u128() as u64,
                self.open_time,
                // accounts
                self.svm_creator.clone(),
                AMM_CONFIG_ACCOUNT.to_string(),
                AUTHORITY_ACCOUNT.to_string(),
                pool_state_account.to_string(),
                Pubkey::from_slice(&denom_0_bz.as_slice())
                    .unwrap()
                    .to_string(),
                Pubkey::from_slice(&denom_1_bz.as_slice())
                    .unwrap()
                    .to_string(),
                lp_mint.to_string(),
                creator_token0_ata.to_string(),
                creator_token1_ata.to_string(),
                creator_lp_ata.to_string(),
                token0_vault.to_string(),
                token1_vault.to_string(),
                POOL_FEE_RECEIVER_ACCOUNT.to_string(),
                oracle_observer_state.to_string(),
                SPL_TOKEN_PROGRAM_ID.to_string(),
                SPL_TOKEN2022_PROGRAM_ID.to_string(),
                SPL_TOKEN2022_PROGRAM_ID.to_string(),
                ASSOCIATED_TOKEN_PROGRAM_ID.to_string(),
                SYSTEM_PROGRAM_ID.to_string(),
                SYS_VAR_RENT_ID.to_string(),
            );

            let mut tx = TransactionBuilder::new();
            tx.add_instructions(vec![initialize_pool]);

            let msg_transaction = tx.build(vec![sender], 10_000_000);

            vec![FISInstruction {
                plane: PLANE_SVM.to_string(),
                action: ACTION_VM_INVOKE.to_string(),
                address: "".to_string(),
                msg: to_json_vec(&msg_transaction).unwrap(),
            }]
        }
    }
}
