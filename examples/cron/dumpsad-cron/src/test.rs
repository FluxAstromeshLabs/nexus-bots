#[cfg(test)]
mod tests {
    use bech32::{Bech32, Hrp};

    use crate::{astromesh::module_address, wasm::astroport::PAIR_CODE_ID};

    #[test]
    fn test_precalculate_wasm_contract_address() {
        let sequence_number = 8u64;
        let contract_id = &[
            "wasm".as_bytes(),
            &[0],
            PAIR_CODE_ID.to_be_bytes().as_slice(),
            sequence_number.to_be_bytes().as_slice(),
        ]
        .concat();
        let pair_address_bz = module_address("module", &contract_id);
        let pair_address_str =
            bech32::encode::<Bech32>(Hrp::parse("lux").unwrap(), &pair_address_bz).unwrap();

        println!("contract address str: {}", pair_address_str)
    }
}
