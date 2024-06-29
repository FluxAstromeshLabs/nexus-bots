#[cfg(test)]
mod tests {
    use crate::{
        astromesh::{Pool, Swap}, evm, svm::{raydium::{self, keccak256, ASSOCIATED_TOKEN_PROGRAM_ID, RAYDIUM, SPL_TOKEN_2022}, Pubkey, TokenAccount}
    };
    use cosmwasm_std::{to_json_string, Binary, Int128};
    #[test]
    fn test_parse_pool_info() {
        let data = hex::decode("000000000bb800000001b326000000000000010655c244ab2aaa152ba8352d52")
            .unwrap();
        let pool_info = evm::uniswap::parse_pool_info(&data.as_slice()).unwrap();
        assert_eq!(
            pool_info.sqrt_price_x96.to_string(),
            "20811535737222300946999034249216",
            "unexpected price"
        );
        assert_eq!(pool_info.tick.to_string(), "111424", "unexpected tick");
        assert_eq!(pool_info.lp_fee, 3000, "unexpected lp fee");
    }

    #[test]
    fn test_parse_token_account() {
        // {"Mint":"AarDASauqWwFsuG9r62pYCH3m9CFtvvQucUs2iLg18AW","Owner":"GonQpn9zzCF2rD521AiYg1RFpC4aFEzJ8RwC9XDi54L6","Amount":399000000,"Delegate":null,"State":1,"IsNative":null,"DelegatedAmount":0,"CloseAuthority":null}
        let account_data = Binary::from_base64("jmT9mZ6EZ9zYFBt+eBm190VZG6DWEc7ey8OWylSvrTPq214B6Yq6zIE78NDn5DRCobUH5USgLNzQMQKlCAQt78BByBcAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgcAAAA=").unwrap();
        let acc = TokenAccount::unpack(&account_data.as_slice()).unwrap();
        assert_eq!(
            acc.mint.to_string(),
            "AarDASauqWwFsuG9r62pYCH3m9CFtvvQucUs2iLg18AW".to_string()
        );
        assert_eq!(
            acc.owner.to_string(),
            "GonQpn9zzCF2rD521AiYg1RFpC4aFEzJ8RwC9XDi54L6".to_string()
        );
        assert_eq!(acc.amount, 399000000);
    }

    #[test]
    fn test_calculate_svm_address() {
        let sender = "lux1jcltmuhplrdcwp7stlr4hlhlhgd4htqhu86cqx";
        let (_, sender_bz) = bech32::decode("lux1jcltmuhplrdcwp7stlr4hlhlhgd4htqhu86cqx").unwrap();
        let sender_svm_account = Pubkey::from_slice(keccak256(&sender_bz.as_slice()).as_slice()).unwrap();
        assert_eq!(sender_svm_account.to_string(), "DRK5Bi2NwkGRPsqHJSyy6rhUo3uQ8YHtt1xUWbu7Bnsx".to_string());

        let token_program = Pubkey::from_string(&SPL_TOKEN_2022.to_string()).unwrap();
        let output_denom_pk = Pubkey::from_string(&"ErDYXZUZ9rpSSvdWvrsQwgh6K4BQeoY2CPyv1FeD1S9r".to_string()).unwrap();
        let input_denom_pk = Pubkey::from_string(&"ENyus6yS21v95sreLKcVEA5Wjcyh8jg6w4jBFHzJaPox".to_string()).unwrap();
        let ata_program = Pubkey::from_string(&ASSOCIATED_TOKEN_PROGRAM_ID.to_string()).unwrap();

        let (output_token_account, _) = Pubkey::find_program_address(
            &[&sender_svm_account.0, &token_program.0, &output_denom_pk.0],
            &ata_program,
        )
        .unwrap();

        let (input_token_account, _) = Pubkey::find_program_address(
            &[&sender_svm_account.0, &token_program.0, &input_denom_pk.0],
            &ata_program,
        )
        .unwrap();

        println!("input ata: {}, output ata: {}", input_token_account.to_string(), output_token_account.to_string());

        let mut pool = raydium::RaydiumPool::default();
        pool.denom_a = "ENyus6yS21v95sreLKcVEA5Wjcyh8jg6w4jBFHzJaPox".to_string();
        pool.denom_b = "ErDYXZUZ9rpSSvdWvrsQwgh6K4BQeoY2CPyv1FeD1S9r".to_string();

        let swap = Swap {
            dex_name: RAYDIUM.to_string(),
            pool_name: "btc-usdt".to_string(),
            sender: sender.to_string(),
            denom: "usdt".to_string(),
            amount: Int128::new(10i128),
        };

        let inst = pool.compose_swap_fis(&swap).unwrap();
        println!("fis marshaled: {}", String::from_utf8(inst.msg).unwrap());
    }

    // #[test]
    // fn test_arbitrage_profit() {
    //     let input_amount = Int256::from(4990212513i128);
    //     let (_, _, _, second_swap_output) = calculate_pools_output(
    //         &Box::new(raydium::RaydiumPool {
    //             dex_name: ASTROPORT.to_string(),
    //             denom_plane: "COSMOS".to_string(),
    //             a: 10000000000i128.into(),
    //             b: 10000000000i128.into(),
    //             fee_rate: Int256::from_i128(10000),
    //             denom_a: "".to_string(),
    //             denom_b: "".to_string(),
    //         }),
    //         &crate::Pool {
    //             dex_name: RAYDIUM.to_string(),
    //             denom_plane: "SVM".to_string(),
    //             a: 139304175643i128.into(),
    //             b: 201000000i128.into(),
    //             fee_rate: Int256::from_i128(1000),
    //         },
    //         input_amount,
    //     );
    //     assert!((second_swap_output - input_amount).gt(&Int256::zero()) == true);
    // }
}
