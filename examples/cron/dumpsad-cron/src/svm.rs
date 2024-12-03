pub mod raydium {
    use cosmwasm_std::Uint128;

    use crate::astromesh::{self, FISInstruction, PoolManager};

    pub struct Raydium {}

    impl PoolManager for Raydium {
        fn create_pool(&self, denom_0: String, denom_1: String) -> Vec<FISInstruction> {
            vec![]
        }

        fn provide_liquidity_no_lp(
            &self,
            pool_id: String,
            denom_0_amount: Uint128,
            denom_1_amount: Uint128,
        ) -> Vec<FISInstruction> {
            vec![]
        }
    }
}
