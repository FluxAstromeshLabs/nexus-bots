#[cfg(test)]
mod tests {
    use cosmwasm_std::Binary;

    use crate::svm::{Instruction, InstructionAccount, InstructionAccountMeta, InstructionMeta, TransactionBuilder};

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
}