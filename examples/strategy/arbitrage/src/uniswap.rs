
use cosmwasm_std::{Binary, Uint256};
use serde::{Deserialize, Serialize};

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
    pub fn new(currency0: [u8; 20], currency1: [u8; 20], fee: u32, tick_spacing: i32, hooks: [u8; 20]) -> Self {
        Self {
            currency0,
            currency1,
            fee,
            tick_spacing,
            hooks,
        }
    }

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
// exchanges, dapp client goes here
