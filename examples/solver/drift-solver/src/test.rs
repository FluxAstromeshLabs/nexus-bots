#[cfg(test)]
mod tests {
    use cosmwasm_std::Binary;

    use crate::{drift::{create_place_order_ix, Example, MarketType, OrderParams, OrderTriggerCondition, OrderType, PositionDirection, PostOnlyParam, User}, svm::{Instruction, InstructionAccount, InstructionAccountMeta, InstructionMeta, TransactionBuilder}};

    #[test]
    fn test_build_transaction() {
        // Define two accounts for the instruction meta with base58 encoded addresses
        let feepayer = "RM3uwjR7LUugxxfZLe9grNC9HNW7BMU9227KsyFsbfB";
        let account_1 = "8LBk2doATLb8M6JX4auYe1gGQMqimHi1hwkKSkLzo6f5";
        let account_2 = "Zs5KiCvJHCN2PwZqEQczvQGizKr6en9AAotSfi9AeWH";
        let system_program = "11111111111111111111111111111111";

        // First create account instruction meta
        let create_account1 = InstructionMeta {
            program_id: system_program.to_string(),
            account_meta: vec![
                InstructionAccountMeta {
                    pubkey: feepayer.to_string(),
                    is_signer: true,
                    is_writable: true,
                },
                InstructionAccountMeta {
                    pubkey: account_1.to_string(),
                    is_signer: true,
                    is_writable: true,
                },
            ],
            data: Binary::from(vec![0x01, 0x02, 0x03]), // Dummy data for instruction
        };

        // Second create account instruction meta
        let create_account2 = InstructionMeta {
            program_id: system_program.to_string(),
            account_meta: vec![
                InstructionAccountMeta {
                    pubkey: feepayer.to_string(),
                    is_signer: true,
                    is_writable: true,
                },
                InstructionAccountMeta {
                    pubkey: account_2.to_string(),
                    is_signer: true,
                    is_writable: true,
                },
                InstructionAccountMeta {
                    pubkey: account_2.to_string(),
                    is_signer: false,
                    is_writable: true,
                },
            ],
            data: Binary::from(vec![0x04, 0x05, 0x06]), // Dummy data for instruction
        };

        // Initialize the transaction builder
        let mut tx_builder = TransactionBuilder::new();
        // Add both create account instructions
        tx_builder.add_instruction(create_account1);
        tx_builder.add_instruction(create_account2);

        // Define dummy cosmos signers with base58 encoding and compute budget
        let cosmos_signers = vec![
            "lux1jcltmuhplrdcwp7stlr4hlhlhgd4htqhu86cqx".to_string(),
            "lux1dzqd00lfd4y4qy2pxa0dsdwzfnmsu27hdef8k5".to_string(),
            "lux1kmmz47pr8h46wcyxw8h3k8s85x0ncykqp0xmgj".to_string(),
        ];
        let compute_budget = 1000;
        // Build the transaction
        let transaction = tx_builder.build(cosmos_signers, compute_budget);
        
        // Assertions for instructions
        assert_eq!(transaction.instructions.len(), 2);

        // First instruction expected
        let expected_first_instruction = Instruction {
            program_index: vec![0],
            accounts: vec![
                InstructionAccount {
                    id_index: 1,
                    caller_index: 1,
                    callee_index: 0,
                    is_signer: true,
                    is_writable: true,
                },
                InstructionAccount {
                    id_index: 2,
                    caller_index: 2,
                    callee_index: 1,
                    is_signer: true,
                    is_writable: true,
                },
            ],
            data: Binary::from(vec![0x01, 0x02, 0x03]),
        };
        assert_eq!(transaction.instructions[0], expected_first_instruction);

        // Second instruction expected
        let expected_second_instruction = Instruction {
            program_index: vec![0],
            accounts: vec![
                InstructionAccount {
                    id_index: 1,
                    caller_index: 1,
                    callee_index: 0,
                    is_signer: true,
                    is_writable: true,
                },
                InstructionAccount {
                    id_index: 3,
                    caller_index: 3,
                    callee_index: 1,
                    is_signer: true,
                    is_writable: true,
                },
                InstructionAccount {
                    id_index: 3,
                    caller_index: 3,
                    callee_index: 1,
                    is_signer: false,
                    is_writable: true,
                },
            ],
            data: Binary::from(vec![0x04, 0x05, 0x06]),
        };
        assert_eq!(transaction.instructions[1], expected_second_instruction);

        // Assertion for compute_budget
        assert_eq!(transaction.compute_budget, compute_budget);
    }

    #[test]
    fn test_parse_user_data() {
        let user_data_b64 = "n3Vf4++XOuwDdZ/tByh5bdCG+STnop80uiI/zlyHH323gqc3qmZ22AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIEqp0QEAAAAAAAAAAAAAAAAAAAAAAAAAlDV3AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAXH0DAAAAAAAM4Rz//////wzhHP//////Ihsd//////8goQcAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAAAAAAAAAAAIKEHAAAAAACGE+n//////4YT6f//////Yxnp//////8goQcAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQABAAAAAAAAAAAAIKEHAAAAAACS2v7//////5La/v//////3tr+//////8AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAYFAAAAAAAAQNREJA8AAAAgoQcAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAB3fiMPAAAAQNREJA8AAABRuxtnAAAAAAAAAAAEAAAAAAABAAEEAAAAAAAACgAAAAcFAAAAAAAAAGcNswAAAAAgoQcAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAECg37IAAAAAAGcNswAAAABSuxtnAAAAAAAAAAAFAAAAAQABAAEFAAAAAAAACgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAJQ1dwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAHBQAAAAAAAAYAAAAAAAAAAQAAAAAAAAIBAgEAAAAAACq2G2cAAAAAAAAAAAAAAAA=";
        let user_data = Binary::from_base64(&user_data_b64).unwrap();
        let user: User = borsh::from_slice(&user_data.as_slice()[8..]).unwrap();

        assert_eq!(user.authority.to_string(), "EWFZJuFzfx1bdWH5wa67KL1joEvYhGF5m6cL9mjHp1V", "authority mismatch");
    }

    #[test]
    fn test_encode_order_param() {
        let order_params = OrderParams {
            order_type: OrderType::Market, // OrderType is 0, which we'll assume is Limit
            market_type: MarketType::Perp, // MarketType is 1, which we'll assume is Perp
            direction: PositionDirection::Long, // Direction is 0, which we'll assume is Long
            user_order_id: 4, // UserOrderId is 4
            base_asset_amount: 500000, // BaseAssetAmount is 500000
            price: 65033000000, // Price is 65033000000
            market_index: 0, // MarketIndex is 0
            reduce_only: false, // ReduceOnly is false
            post_only: PostOnlyParam::None, // PostOnly is 0, which we'll assume means None
            immediate_or_cancel: false, // ImmediateOrCancel is false
            max_ts: Some(1729870673), // MaxTs is 1729870673
            trigger_price: Some(0), // TriggerPrice is 0
            trigger_condition: OrderTriggerCondition::Above, // TriggerCondition is 0, which we'll assume is None
            oracle_price_offset: Some(0), // OraclePriceOffset is 0
            auction_duration: Some(10), // AuctionDuration is 10
            auction_start_price: Some(65020000000), // AuctionStartPrice is 65020000000
            auction_end_price: Some(65033000000), // AuctionEndPrice is 65033000000
        };
        
        let ix = create_place_order_ix(
            "7WrZxBiKCMGuzLCW2VwKK7sQjhTZLbDe5sKfJsEcARpF".to_string(), 
            "2GKUdmaBJNjfCucDT14HrsWchVrm3yvj4QY2jjnUEg3v".to_string(), order_params
        ).unwrap();

        assert_eq!(ix.get(0).unwrap().data.to_vec(), [69, 161, 93, 202, 120, 126, 76, 185, 0, 1, 0, 4, 32, 161, 7, 0, 0, 0, 0, 0, 64, 212, 68, 36, 15, 0, 0, 0, 0, 0, 0, 0, 0, 1, 81, 187, 27, 103, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 10, 1, 0, 119, 126, 35, 15, 0, 0, 0, 1, 64, 212, 68, 36, 15, 0, 0, 0]);
    }
}