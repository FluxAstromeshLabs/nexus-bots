#[cfg(test)]
mod tests {
    use cosmwasm_std::HexBinary;
    use rlp::RlpStream;

    #[test]
    fn test_calculate_denom_svm_address() {
        let (_, addr) = bech32::decode("lux158ucxjzr6ccrlpmz8z05wylu8tr5eueqcp2afu").unwrap();
        let sequence = 0u64;
        let mut stream = RlpStream::new_list(2); // Specify the list size
        stream.append(&addr);
        stream.append(&sequence);
        let encoded_list = stream.out();
        assert_eq!(
            HexBinary::from(encoded_list.to_vec()).to_string().as_str(),
            "d694a1f9834843d6303f8762389f4713fc3ac74cf32080"
        );
    }
}
