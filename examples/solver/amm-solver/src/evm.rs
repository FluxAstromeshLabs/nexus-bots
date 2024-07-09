use cosmwasm_std::{Binary, StdError};
// use fixed::{types::extra, FixedU128};

use serde::{Deserialize, Serialize};

const EVM: &str = "EVM";

pub mod uniswap {
    use cosmwasm_std::{to_json_vec, Binary, Int256, StdError, Uint256};
    use serde::{Deserialize, Serialize};

    use super::{left_pad, MsgExecuteContract, EVM};
    use crate::astromesh::{FISInstruction, Pool, Swap};

    pub const UNISWAP: &str = "UNISWAP";
    pub const POOL_MANAGER: &str = "07aa076883658b7ed99d25b1e6685808372c8fe2";

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct PoolKey {
        /// @notice The lower currency of the pool, sorted numerically
        currency0: [u8; 20],
        /// @notice The higher currency of the pool, sorted numerically
        currency1: [u8; 20],
        /// @notice The pool swap fee, capped at 1_000_000. If the first bit is 1, the pool has a dynamic fee and must be exactly equal to 0x800000
        fee: u32,
        /// @notice Ticks that involve positions must be a multiple of tick spacing
        tick_spacing: i32,
        /// @notice The hooks of the pool
        hooks: [u8; 20],
    }

    pub struct SwapParams {
        pub zero_for_one: bool,
        pub amount: Int256,
        pub sqrt_price_limit_x96: Uint256,
    }

    impl SwapParams {
        pub fn new(zero_for_one: bool, amount: Int256, sqrt_price_limit_x96: Uint256) -> Self {
            Self {
                zero_for_one,
                amount,
                sqrt_price_limit_x96,
            }
        }

        pub fn serialize(&self) -> Vec<u8> {
            let mut serialized = Vec::with_capacity(96); // 32 bytes for each field with padding

            // Adding padding and then zero_for_one
            serialized.extend_from_slice(&[0u8; 31]); // 31 bytes of padding
            serialized.push(self.zero_for_one as u8); // 1 byte for bool

            serialized.extend_from_slice(&self.amount.to_be_bytes());

            serialized.extend_from_slice(&self.sqrt_price_limit_x96.to_be_bytes());

            serialized
        }
    }

    impl PoolKey {
        pub fn new(
            currency0: [u8; 20],
            currency1: [u8; 20],
            fee: u32,
            tick_spacing: i32,
            hooks: [u8; 20],
        ) -> Self {
            Self {
                currency0,
                currency1,
                fee,
                tick_spacing,
                hooks,
            }
        }
        // ethabi (getrandom())

        // x: 9244377900000000000000000000000017cf225befbdc683a48db215305552b3897906f600000000000000000000000018ab0f92ffb8b4f07f2d95b193bafd377ab25cc40000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffec7800000000000000000000000000000000000000eaf261a5dfcea000000000000000000000000000000000000000000000000000000000000000000000000001200000000000000000000000000000000000000000000000000000000000000000
        pub fn serialize(&self) -> Vec<u8> {
            let mut serialized = Vec::with_capacity(160);
            // Adding padding for currency0
            serialized.extend_from_slice(&[0u8; 12]); // padding to make it 32 bytes
            serialized.extend_from_slice(&self.currency0);

            // Adding padding for currency1
            serialized.extend_from_slice(&[0u8; 12]); // padding to make it 32 bytes
            serialized.extend_from_slice(&self.currency1);

            // Adding padding for fee
            serialized.extend_from_slice(&[0u8; 28]); // padding to make it 32 bytes
            serialized.extend_from_slice(&self.fee.to_be_bytes());

            // Adding padding for tick_spacing
            serialized.extend_from_slice(&[0u8; 28]); // padding to make it 32 bytes
            serialized.extend_from_slice(&self.tick_spacing.to_be_bytes());

            // Adding padding for hooks
            serialized.extend_from_slice(&[0u8; 12]); // padding to make it 32 bytes
            serialized.extend_from_slice(&self.hooks);

            serialized
        }
    }

    fn serialize_swap_calldata(pook_key: PoolKey, swap_params: SwapParams) -> Vec<u8> {
        let signature = (0x92443779u32).to_be_bytes();
        let pool_key_bz = pook_key.serialize();
        let swap_params_bz = swap_params.serialize();
        let empty_hook_data = Binary::from_base64("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==").unwrap();
        let mut res = Vec::new();
        res.extend(signature);
        res.extend(pool_key_bz);
        res.extend(swap_params_bz);
        res.extend(empty_hook_data.iter());
        res
    }

    #[derive(Debug, Clone)]
    pub struct PoolInfo {
        pub sqrt_price_x96: Uint256,
        pub tick: Int256,
        pub protocol_fee: u32,
        pub lp_fee: u32,
    }

    fn signed_big_int_from_bytes(b: &[u8]) -> Result<Int256, StdError> {
        Ok(Int256::from_be_bytes(
            left_pad(b, 32)?
                .try_into()
                .expect("Slice with incorrect length"),
        ))
    }

    pub fn parse_pool_info(data: &[u8]) -> Result<PoolInfo, cosmwasm_std::StdError> {
        if data.len() != 32 {
            return Err(StdError::generic_err(
                "input data must be 32 bytes".to_string(),
            ));
        }

        let sqrt_price_x96 =
            Uint256::from_be_bytes(left_pad(&data[12..32], 32)?.as_slice().try_into().unwrap());

        let tick_bytes = &data[9..12];
        let tick = signed_big_int_from_bytes(tick_bytes)?;

        let protocol_fee_bytes = &data[6..9];
        let protocol_fee = u32::from_be_bytes(left_pad(protocol_fee_bytes, 4)?.try_into().unwrap());

        let lp_fee_bytes = &data[3..6];
        let lp_fee = u32::from_be_bytes(left_pad(lp_fee_bytes, 4)?.try_into().unwrap());

        Ok(PoolInfo {
            sqrt_price_x96,
            tick,
            protocol_fee,
            lp_fee,
        })
    }

    fn parse_addr(addr: &str) -> [u8; 20] {
        let mut res = [0u8; 20];
        hex::decode_to_slice(addr, res.as_mut_slice()).unwrap();
        res
    }

    pub fn get_pool_key_by_name(pool_name: &str) -> Result<PoolKey, StdError> {
        match pool_name {
            "btc-usdt" => Ok(PoolKey {
                currency0: parse_addr("17cf225befbdc683a48db215305552b3897906f6"),
                currency1: parse_addr("18ab0f92ffb8b4f07f2d95b193bafd377ab25cc4"),
                fee: 3000u32,
                tick_spacing: 60i32,
                hooks: [0; 20],
            }),
            _ => Err(StdError::generic_err(format!(
                "unsupported pool: {}",
                pool_name
            ))),
        }
    }

    pub fn get_denom(alias: &str) -> Result<[u8; 20], StdError> {
        match alias {
            "btc" => Ok(parse_addr("17cf225befbdc683a48db215305552b3897906f6")),
            "usdt" => Ok(parse_addr("18ab0f92ffb8b4f07f2d95b193bafd377ab25cc4")),
            _ => Err(StdError::generic_err(format!(
                "evm denom not found: {}",
                alias
            ))),
        }
    }

    pub struct UniswapPool {
        pub dex_name: String,
        pub denom_plane: String,
        pub a: Int256,
        pub b: Int256,
        pub fee_rate: Int256,
        pub denom_a: String,
        pub denom_b: String,
        pub tick_spacing: i32,
    }

    impl UniswapPool {
        pub fn new(pair: &str) -> Result<Self, StdError> {
            match pair {
                "btc-usdt" => Ok(Self {
                    dex_name: UNISWAP.to_string(),
                    denom_plane: EVM.to_string(),
                    a: Int256::zero(),
                    b: Int256::zero(),
                    fee_rate: Int256::from_i128(3000),
                    denom_a: "btc".to_string(),
                    denom_b: "usdt".to_string(),
                    tick_spacing: 60,
                }),
                _ => Err(StdError::generic_err(format!(
                    "unsupported uniswap pair: {}",
                    pair
                ))),
            }
        }
    }

    impl Pool for UniswapPool {
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

        fn swap_output(&self, _x: Int256, _a_for_b: bool) -> (String, Int256) {
            // TODO: Implement to estimate swap output for arbitrage
            ("".to_string(), Int256::zero())
        }

        fn compose_swap_fis(&self, swap: &Swap) -> Result<FISInstruction, StdError> {
            let pool_key = get_pool_key_by_name(&swap.pool_name)?;
            let src_denom = get_denom(swap.denom.as_str())?;
            let zero_for_one = src_denom.eq(pool_key.currency0.as_slice());
            let min_price = if zero_for_one {
                Uint256::zero()
            } else {
                Uint256::MAX
            };

            let swap_params = SwapParams {
                zero_for_one,
                amount: Int256::from_i128(-swap.amount.i128()),
                sqrt_price_limit_x96: min_price,
            };

            // compose swap
            let calldata = serialize_swap_calldata(pool_key, swap_params);
            let msg = MsgExecuteContract::new(
                swap.clone().sender,
                Binary::from(hex::decode(POOL_MANAGER).unwrap()),
                Binary::from(calldata),
                Binary::from(vec![]),
            );

            Ok(FISInstruction {
                plane: "EVM".to_string(),
                action: "VM_INVOKE".to_string(),
                address: "".to_string(),
                msg: to_json_vec(&msg)?,
            })
        }
    }
}

fn left_pad(input: &[u8], expected_len: usize) -> Result<Vec<u8>, StdError> {
    if input.len() > expected_len {
        return Err(StdError::generic_err(
            "input len must not exceeds expected len",
        ));
    }

    let mut padded = vec![0u8; expected_len];
    let start_index = expected_len - input.len();
    padded[start_index..].copy_from_slice(input);

    Ok(padded)
}

// more dapp types goes here
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MsgExecuteContract {
    pub sender: String,
    pub contract_address: Binary,
    pub calldata: Binary,
    pub input_amount: Binary,
}

impl MsgExecuteContract {
    pub fn new(
        sender: String,
        contract_address: Binary,
        calldata: Binary,
        input_amount: Binary,
    ) -> Self {
        MsgExecuteContract {
            sender,
            contract_address,
            calldata,
            input_amount,
        }
    }
}
