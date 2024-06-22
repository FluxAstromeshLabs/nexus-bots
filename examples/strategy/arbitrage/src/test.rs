#[cfg(test)]
mod tests {
    use crate::evm;

    #[test]
    fn hello_test() {
        println!("OUTPUT: hello world");
    }

    #[test]
    fn test_parse_pool_info() {
        let data = hex::decode("000000000bb800000001b326000000000000010655c244ab2aaa152ba8352d52")
            .unwrap();
        let pool_info = evm::uniswap::parse_pool_info(&data.as_slice());
        println!("pool info: {:#?}", pool_info);
    }
}
