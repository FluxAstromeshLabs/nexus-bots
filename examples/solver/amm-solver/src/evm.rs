use cosmwasm_std::{Binary, StdError, Uint256};
// use fixed::{types::extra, FixedU128};

use serde::{Deserialize, Serialize};

const MAX_TICK: u32 = 887272u32;

pub mod uniswap {
    use cosmwasm_std::{Binary, Int256, StdError, Uint256};
    use serde::{Deserialize, Serialize};

    use crate::astromesh::{FISInstruction, Swap};

    use super::{get_price_at_tick, left_pad};

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
        pub amount: Uint256,
        pub sqrt_price_limit_x96: Uint256,
    }

    impl SwapParams {
        pub fn new(zero_for_one: bool, amount: Uint256, sqrt_price_limit_x96: Uint256) -> Self {
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

    pub fn serialize_swap_calldata(pook_key: PoolKey, swap_params: SwapParams) -> Vec<u8> {
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

    impl PoolInfo {
        // returns currentcy0, currency1 amount
        pub fn calculate_liquidity_amounts(
            &self,
            liquidity: Uint256,
            lower_tick: i32,
            upper_tick: i32,
        ) -> (Int256, Int256) {
            let lower_price = get_price_at_tick(lower_tick);
            let upper_price = get_price_at_tick(upper_tick);
            let cur_price = self.sqrt_price_x96;

            // token0 = L * (1/sqrt(curPrice) - 1/sqrt(upperPrice))
            // token1 =  L * (sqrt(curPrice) - sqrt(lowerPrice))
            let denom_0_amount =
                ((liquidity * (upper_price - cur_price)) << 96) / (cur_price * upper_price);
            let denom_1_amount = (liquidity * (cur_price - lower_price)) >> 96;

            (
                Int256::from_be_bytes(denom_0_amount.to_be_bytes()),
                Int256::from_be_bytes(denom_1_amount.to_be_bytes()),
            )
        }
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

pub fn get_price_at_tick(tick: i32) -> Uint256 {
    let abs_tick: u32 = if tick < 0 {
        (-tick) as u32
    } else {
        tick as u32
    };

    if abs_tick > MAX_TICK {
        panic!("InvalidTick");
    }

    let mut price: Uint256 = if abs_tick & 1 != 0 {
        Uint256::from(0xfffcb933bd6fad37aa2d162d1a594001u128)
    } else {
        Uint256::from_be_bytes(
            left_pad(
                hex::decode("0100000000000000000000000000000000")
                    .unwrap()
                    .as_slice(),
                32,
            )
            .unwrap()
            .try_into()
            .unwrap(),
        )
    };

    if abs_tick & 2 != 0 {
        price = (price * Uint256::from(0xfff97272373d413259a46990580e213a_u128)) >> 128;
    }
    if abs_tick & 4 != 0 {
        price = (price * Uint256::from(0xfff2e50f5f656932ef12357cf3c7fdcc_u128)) >> 128;
    }
    if abs_tick & 8 != 0 {
        price = (price * Uint256::from(0xffe5caca7e10e4e61c3624eaa0941cd0_u128)) >> 128;
    }
    if abs_tick & 16 != 0 {
        price = (price * Uint256::from(0xffcb9843d60f6159c9db58835c926644_u128)) >> 128;
    }
    if abs_tick & 32 != 0 {
        price = (price * Uint256::from(0xff973b41fa98c081472e6896dfb254c0_u128)) >> 128;
    }
    if abs_tick & 64 != 0 {
        price = (price * Uint256::from(0xff2ea16466c96a3843ec78b326b52861_u128)) >> 128;
    }
    if abs_tick & 128 != 0 {
        price = (price * Uint256::from(0xfe5dee046a99a2a811c461f1969c3053_u128)) >> 128;
    }
    if abs_tick & 256 != 0 {
        price = (price * Uint256::from(0xfcbe86c7900a88aedcffc83b479aa3a4_u128)) >> 128;
    }
    if abs_tick & 512 != 0 {
        price = (price * Uint256::from(0xf987a7253ac413176f2b074cf7815e54_u128)) >> 128;
    }
    if abs_tick & 1024 != 0 {
        price = (price * Uint256::from(0xf3392b0822b70005940c7a398e4b70f3_u128)) >> 128;
    }
    if abs_tick & 2048 != 0 {
        price = (price * Uint256::from(0xe7159475a2c29b7443b29c7fa6e889d9_u128)) >> 128;
    }
    if abs_tick & 4096 != 0 {
        price = (price * Uint256::from(0xd097f3bdfd2022b8845ad8f792aa5825_u128)) >> 128;
    }
    if abs_tick & 8192 != 0 {
        price = (price * Uint256::from(0xa9f746462d870fdf8a65dc1f90e061e5_u128)) >> 128;
    }
    if abs_tick & 16384 != 0 {
        price = (price * Uint256::from(0x70d869a156d2a1b890bb3df62baf32f7_u128)) >> 128;
    }
    if abs_tick & 32768 != 0 {
        price = (price * Uint256::from(0x31be135f97d08fd981231505542fcfa6_u128)) >> 128;
    }
    if abs_tick & 65536 != 0 {
        price = (price * Uint256::from(0x9aa508b5b7a84e1c677de54f3e99bc9_u128)) >> 128;
    }
    if abs_tick & 131072 != 0 {
        price = (price * Uint256::from(0x5d6af8dedb81196699c329225ee604_u128)) >> 128;
    }
    if abs_tick & 262144 != 0 {
        price = (price * Uint256::from(0x2216e584f5fa1ea926041bedfe98_u128)) >> 128;
    }
    if abs_tick & 524288 != 0 {
        price = (price * Uint256::from(0x48a170391f7dc42444e8fa2_u128)) >> 128;
    }

    if tick > 0 {
        price = Uint256::MAX / price;
    }

    // Convert to Q128.96 by shifting right by 32 bits and rounding up
    let price_with_rounding = price + Uint256::from((1u128 << 32) - 1);
    
    price_with_rounding >> 32
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
