pub mod raydium {
    use cosmwasm_std::Uint128;

    use crate::astromesh::{self, FISInstruction, PoolManager};

    pub struct Raydium {}

    impl PoolManager for Raydium {
        fn create_pool_with_initial_liquidity(
            &self,
            sender: String,
            denom_0: String,
            amount_0: Uint128,
            denom_1: String,
            amount_1: Uint128,
        ) -> Vec<FISInstruction> {
            vec![]
        }
    }
}
