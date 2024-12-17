use cosmwasm_std::{Binary, StdError};
use rug::{Float, Integer};

use serde::{Deserialize, Serialize};

pub mod uniswap {

    use cosmwasm_std::Uint128;

    use crate::astromesh::{self, FISInstruction, PoolManager, ACTION_VM_INVOKE, PLANE_EVM};

    pub struct Uniswap {
        pub fee: Uint128,
        pub price: f64,
    }

    pub const UNISWAP: &str = "uniswap";
    pub const POOL_MANAGER: &str = "6ff00f6b2120157fca353fbe24d25536042197df";
    pub const POOL_ACTION: &str = "366c9837f9a2cc11ac5cac1602e57b73e6bf784";

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

    pub struct ModifyLiquidityParams {
        tick_lower: i32,
        tick_upper: i32,
        liquidity_delta: i32,
        salt: [u8; 32],
    }

    impl ModifyLiquidityParams {
        pub fn new(tick_lower: i32, tick_upper: i32, liquidity_delta: i32, salt: [u8; 32]) -> Self {
            Self {
                tick_lower,
                tick_upper,
                liquidity_delta,
                salt,
            }
        }

        pub fn serialize(&self) -> Vec<u8> {
            let mut serialized = Vec::with_capacity(128);
            // Adding padding for tick_lower
            serialized.extend_from_slice(&[0u8; 28]); // padding to make it 32 bytes
            serialized.extend_from_slice(&self.tick_lower.to_be_bytes());
            
            // Adding padding for tick_upper
            serialized.extend_from_slice(&[0u8; 28]); // padding to make it 32 bytes
            serialized.extend_from_slice(&self.tick_upper.to_be_bytes());

            // Adding padding for liquidity_delta
            serialized.extend_from_slice(&[0u8; 28]); // padding to make it 32 bytes
            serialized.extend_from_slice(&self.liquidity_delta.to_be_bytes());

            // Adding padding for salt
            serialized.extend_from_slice(&self.salt);
            
            serialized
        }
    }

    fn compute_sqrt_price_x96_int(p: f64) -> Integer {
        // Set precision for floating-point calculations
        let precision = 128;
    
        // sqrtPrice = sqrt(p)
        let p_big = Float::with_val(precision, p);
        let sqrt_price = p_big.sqrt();
    
        // factor = 2^96
        let factor = Float::with_val(precision, Integer::from(1) << 96);
    
        // sqrtPrice * factor
        let sqrt_price_x96 = &sqrt_price * &factor;
    
        // Convert to Integer
        sqrt_price_x96.to_integer()
    }

    fn compute_tick(price: f64, spacing: i64) -> Integer {
        // Precision for floating-point calculations
        let precision = 64;
    
        // Initialize constants
        let factor = Float::with_val(precision, 1.0001);
        let price_big = Float::with_val(precision, price);
    
        // Compute logarithms
        let log_price = price_big.ln();
        let log_factor = factor.ln();
    
        // floatTick = log(price) / log(1.0001)
        let float_tick = &log_price / &log_factor;
    
        // Convert floatTick to an integer
        let int_tick = float_tick.to_integer();
    
        // Round the tick to the closest spacing
        let spacing_big = Integer::from(spacing);
        let rounded_tick = (&int_tick / &spacing_big) * &spacing_big;
    
        rounded_tick
    }

    fn serialize_initilize_calldata(
        sender: String,
        denom_0: String,
        amount_0: Uint128,
        denom_1: String,
        amount_1: Uint128,
    ) -> Vec<u8> {
        // not done things: 
        // 1. fee
        // 2. price
        // 3. transfer from denom to address
        let signature = (0x695c5bf5u32).to_be_bytes();
        let sqrt_price_x96_int = compute_sqrt_price_x96_int(self.price);
        let tick_spacing = 60;
        let pool_key = PoolKey {
            parse_addr(&denom_0),
            parse_addr(&denom_1),
            self.fee.u32(),
            tick_spacing,
            hooks: [0; 20],
        };
        let price = amount_0.u128().to_be_bytes();
        let empty_hook_data = Binary::from_base64("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAASAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==").unwrap();
        
        let mut res = Vec::new();
        res.extend(signature);
        res.extend(pool_key.serialize());
        res.extend(sqrt_price_x96_int.to_be_bytes());
        res.extend(empty_hook_data.iter());

        res
    }

    fn serialised_provide_liquidity_calldata() -> Vec<u8> {
        let signature = (0x568846efu32).to_be_bytes();
        let tick_spacing = 60;
        let pool_key = PoolKey {
            parse_addr(&denom_0),
            parse_addr(&denom_1),
            self.fee.u32(),
            tick_spacing,
            hooks: [0; 20],
        };
        let salt: [u8; 32] = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 
            7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2,
        ];

        let lower_price = price * 0.8;
        let upper_price = price * 1.2;

        let tick_lower = compute_tick(lower_price, tick_spacing);
        let tick_upper = compute_tick(upper_price, tick_spacing);

        let modify_liquidity_params = ModifyLiquidityParams {
            tick_lower: tick_lower,
            tick_upper: tick_upper,
            liquidity_delta: 1000000000,
            salt: salt,
        };
        let empty_hook_data = Binary::from_base64("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAASAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==").unwrap();

        let mut res = Vec::new();
        res.extend(signature);
        res.extend(pool_key.serialize());
        res.extend(modify_liquidity_params.serialize());
        res.extend(empty_hook_data.iter());

        res
    }

    fn parse_addr(addr: &str) -> [u8; 20] {
        let mut res = [0u8; 20];
        hex::decode_to_slice(addr, res.as_mut_slice()).unwrap();
        res
    }

    impl PoolManager for Uniswap {
        fn create_pool_with_initial_liquidity(
            &self,
            sender: String,
            denom_0: String,
            amount_0: Uint128,
            denom_1: String,
            amount_1: Uint128,
        ) -> Vec<FISInstruction> {
            
            calldata_initilize = serialised_initilize_calldata(sender, denom_0, amount_0, denom_1, amount_1);
            calldata_provide_liquidity = serialised_provide_liquidity_calldata();

            vec![
                FISInstruction {
                    plane: PLANE_EVM.to_string(),
                    action: ACTION_VM_INVOKE.to_string(),
                    address: "".to_string(),
                    msg: to_json_vec(&MsgExecuteContract::new(
                        sender.clone(),
                        Binary::from(hex::decode(POOL_MANAGER).unwrap()),
                        Binary::from(calldata_initilize),
                        Binary::from(vec![]),
                    ))
                    .unwrap(),
                },
                FISInstruction {
                    plane: PLANE_EVM.to_string(),
                    action: ACTION_VM_INVOKE.to_string(),
                    address: "".to_string(),
                    msg: to_json_vec(&MsgExecuteContract::new(
                        sender.clone(),
                        Binary::from(hex::decode(POOL_ACTION).unwrap()),
                        Binary::from(calldata_provide_liquidity),
                        Binary::from(vec![]),
                    ))
                    .unwrap(),
                },
            ]
        }
    }
}

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
