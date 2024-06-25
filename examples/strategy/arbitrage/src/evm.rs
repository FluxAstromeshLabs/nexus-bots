use cosmwasm_std::Binary;
use serde::{Deserialize, Serialize};

pub mod uniswap {
    use cosmwasm_std::{Binary, Int256, StdError, Uint256};
    use serde::{Deserialize, Serialize};

    use crate::astromesh::{FISInstruction, Swap};

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

    fn left_pad(input: &[u8], expected_len: usize) -> Result<Vec<u8>, StdError> {
        if input.len() > expected_len {
            return Err(StdError::generic_err(
                "input len must not exceeds expected len",
            ));
        }

        let mut padded = vec![0u8; expected_len];
        let start_index = expected_len - input.len();
        padded[start_index..].copy_from_slice(&input);

        Ok(padded)
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

    // TODO: Fill in
    pub fn compose_swap_fis(_sender: String, _swap: Swap) -> Result<FISInstruction, StdError> {
        Err(StdError::generic_err("unimplemented"))
    }
}

// more dapp types goes here
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MsgExecuteContract {
    #[serde(rename = "@type")]
    pub ty: String,
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
            ty: "flux.evm.v1beta1.MsgExecuteContract".to_string(),
            sender,
            contract_address,
            calldata,
            input_amount,
        }
    }
}
