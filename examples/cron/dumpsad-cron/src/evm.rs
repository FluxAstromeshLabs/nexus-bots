pub mod uniswap {

    use cosmwasm_std::{to_json_vec, Binary, StdError, Uint128, Uint256};
    use serde::{Deserialize, Serialize};

    use crate::astromesh::{FISInstruction, PoolManager, ACTION_VM_INVOKE, PLANE_EVM};

    pub struct Uniswap {
        pub fee: u32,
        pub price: f64,
    }

    pub const POOL_MANAGER: &str = "6ff00f6b2120157fca353fbe24d25536042197df";
    pub const POOL_ACTION: &str = "366c9837f9a32cc11ac5cac1602e57b73e6bf784";

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

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct ModifyLiquidityParams {
        tick_lower: i64,
        tick_upper: i64,
        liquidity_delta: i32,
        salt: [u8; 32],
    }

    impl ModifyLiquidityParams {
        pub fn new(tick_lower: i64, tick_upper: i64, liquidity_delta: i32, salt: [u8; 32]) -> Self {
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
            serialized.extend_from_slice(&[0u8; 24]); // padding to make it 32 bytes
            serialized.extend_from_slice(&self.tick_lower.to_be_bytes());

            // Adding padding for tick_upper
            serialized.extend_from_slice(&[0u8; 24]); // padding to make it 32 bytes
            serialized.extend_from_slice(&self.tick_upper.to_be_bytes());

            // Adding padding for liquidity_delta
            serialized.extend_from_slice(&[0u8; 28]); // padding to make it 32 bytes
            serialized.extend_from_slice(&self.liquidity_delta.to_be_bytes());

            // Adding padding for salt
            serialized.extend_from_slice(&self.salt);

            serialized
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

    pub fn left_pad(input: &[u8], expected_len: usize) -> Result<Vec<u8>, StdError> {
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

    pub fn parse_addr(addr: &str) -> [u8; 20] {
        let mut res = [0u8; 20];
        hex::decode_to_slice(addr, res.as_mut_slice()).unwrap();
        res
    }

    fn compute_sqrt_price_x96_int(price: f64) -> Uint256 {
        let sqrt_price = price.sqrt();
        let scale_factor: f64 = 2_f64.powi(96);
        let sqrt_price_x96_int = sqrt_price * scale_factor;
        Uint256::from(sqrt_price_x96_int as u128)
    }

    fn compute_tick(price: f64, tick_spacing: i64) -> i64 {
        let log_base = 1.0001_f64.ln();
        let tick = (price.ln() / log_base).floor() as i64;
        let valid_tick = tick_spacing * (tick / tick_spacing);

        valid_tick
    }

    fn compose_erc20_approve(
        sender: &String,
        erc20_addr: &[u8; 20],
        delegator: &[u8; 20],
        amount: Uint256,
    ) -> FISInstruction {
        let signature: [u8; 4] = (0x095ea7b3u32).to_be_bytes();
        let padded_delegator = left_pad(delegator, 32).unwrap();
        let amount_bytes = amount.to_be_bytes();
        let mut calldata = Vec::new();
        calldata.extend(signature);
        calldata.extend(padded_delegator);
        calldata.extend(amount_bytes);

        let msg = MsgExecuteContract::new(
            sender.clone(),
            Binary::new(erc20_addr.to_vec()),
            Binary::from(calldata),
            Binary::from(vec![]),
        );

        FISInstruction {
            plane: PLANE_EVM.to_string(),
            action: ACTION_VM_INVOKE.to_string(),
            address: "".to_string(),
            msg: to_json_vec(&msg).unwrap(),
        }
    }

    fn initialize(
        fee: u32,
        price: f64,
        sender: String,
        denom_0: String,
        denom_1: String,
    ) -> FISInstruction {
        let signature: [u8; 4] = (0x695c5bf5u32).to_be_bytes();
        let tick_spacing = 60;

        let pool_key = PoolKey::new(
            parse_addr(&denom_0),
            parse_addr(&denom_1),
            fee,
            tick_spacing,
            [0; 20],
        );
        let sqrt_price_x96_int = compute_sqrt_price_x96_int(price);
        let empty_hook_data: [u8; 64] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 224, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ];

        let mut calldata = Vec::new();
        calldata.extend(signature);
        calldata.extend(pool_key.serialize());
        calldata.extend(sqrt_price_x96_int.to_be_bytes());
        calldata.extend(empty_hook_data.iter());

        let msg = MsgExecuteContract::new(
            sender.to_string(),
            Binary::from(hex::decode(POOL_MANAGER).unwrap()),
            Binary::from(calldata),
            Binary::from(vec![]),
        );

        FISInstruction {
            plane: PLANE_EVM.to_string(),
            action: ACTION_VM_INVOKE.to_string(),
            address: "".to_string(),
            msg: to_json_vec(&msg).unwrap(),
        }
    }

    fn provide_liquidity(
        fee: u32,
        price: f64,
        sender: String,
        denom_0: String,
        denom_1: String,
    ) -> FISInstruction {
        let signature: [u8; 4] = (0x568846efu32).to_be_bytes();
        let tick_spacing = 60;
        let pool_key = PoolKey::new(
            parse_addr(&denom_0),
            parse_addr(&denom_1),
            fee,
            tick_spacing,
            [0; 20],
        );
        let salt: [u8; 32] = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
            0, 1, 2,
        ];

        let lower_price = price * 0.8;
        let upper_price = price * 1.2;

        let tick_lower = compute_tick(lower_price, tick_spacing.into());
        let tick_upper = compute_tick(upper_price, tick_spacing.into());

        let modify_liquidity_params =
            ModifyLiquidityParams::new(tick_lower, tick_upper, 1000000000, salt);
        let empty_hook_data: [u8; 64] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 1, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
        ];

        let mut calldata = Vec::new();
        calldata.extend(signature);
        calldata.extend(pool_key.serialize());
        calldata.extend(modify_liquidity_params.serialize());
        calldata.extend(empty_hook_data);

        let msg = MsgExecuteContract::new(
            sender.to_string(),
            Binary::from(hex::decode(POOL_ACTION).unwrap()),
            Binary::from(calldata),
            Binary::from(vec![]),
        );

        FISInstruction {
            plane: PLANE_EVM.to_string(),
            action: ACTION_VM_INVOKE.to_string(),
            address: "".to_string(),
            msg: to_json_vec(&msg).unwrap(),
        }
    }

    impl PoolManager for Uniswap {
        fn create_pool_with_initial_liquidity(
            &self,
            sender: String,
            denom_0: String,
            _amount_0: Uint128,
            denom_1: String,
            _amount_1: Uint128,
        ) -> Vec<FISInstruction> {
            let mut instructions = Vec::new();

            let allowance: Uint256 = Uint256::from(100000000000000000u128);
            instructions.push(compose_erc20_approve(
                &sender.to_string(),
                &parse_addr(&denom_0),
                &parse_addr(&POOL_ACTION),
                allowance.into(),
            ));

            instructions.push(compose_erc20_approve(
                &sender.to_string(),
                &parse_addr(&denom_1),
                &parse_addr(&POOL_ACTION),
                allowance.into(),
            ));

            instructions.push(initialize(
                self.fee,
                self.price,
                sender.to_string(),
                denom_0.to_string(),
                denom_1.to_string(),
            ));

            instructions.push(provide_liquidity(
                self.fee,
                self.price,
                sender.to_string(),
                denom_0.to_string(),
                denom_1.to_string(),
            ));

            instructions
        }
    }
}
