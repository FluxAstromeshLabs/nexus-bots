#[cfg(test)]
mod tests {
    use cosmwasm_std::{Binary, Int256};

    use crate::{evm, calculate_pools_output, svm::TokenAccount, ASTROPORT, RAYDIUM};
    #[test]
    fn test_parse_pool_info() {
        let data = hex::decode("000000000bb800000001b326000000000000010655c244ab2aaa152ba8352d52")
            .unwrap();
        let pool_info = evm::uniswap::parse_pool_info(&data.as_slice()).unwrap();
        assert_eq!(
            pool_info.sqrt_price_x96.to_string(),
            "20784319660459464383123105852754",
            "unexpected price"
        );
        assert_eq!(pool_info.tick.to_string(), "111398", "unexpected tick");
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
    fn test_arbitrage_profit() {
        let input_amount = Int256::from(4990212513i128);
        let (first_swap, second_swap) = calculate_pools_output(
            &crate::Pool { 
                dex_name: ASTROPORT.to_string(), 
                denom_plane: "COSMOS".to_string(), 
                a: 10000000000i128.into(), 
                b: 10000000000i128.into(), 
                fee_rate: Int256::from_i128(10000),
            }, 
            &crate::Pool {
                dex_name: RAYDIUM.to_string(), 
                denom_plane: "SVM".to_string(), 
                a: 139304175643i128.into(), 
                b: 201000000i128.into(), 
                fee_rate: Int256::from_i128(1000),
            }, 
            input_amount,
        );
        assert!((second_swap - input_amount).gt(&Int256::zero()) == true);
    }
}
