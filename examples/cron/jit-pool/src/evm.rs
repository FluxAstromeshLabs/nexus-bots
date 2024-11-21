use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint256;
use std::convert::TryInto;

#[cw_serde]
pub struct LiquidityRequestEvent {
    pub user: [u8; 20],       // The address triggering the swap (as 20-byte array)
    pub src_token: [u8; 20],  // Address of the source token (as 20-byte array)
    pub src_amount: Uint256,  // Source amount
    pub dst_token: [u8; 20],  // Address of the destination token (as 20-byte array)
    pub dst_amount: Uint256,  // Desired destination amount
}

impl LiquidityRequestEvent {
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() != 160 {
            return Err(format!(
                "Invalid data length: expected 160 bytes, got {}",
                data.len()
            ));
        }

        // Extract 32-byte chunks and convert them to the appropriate types
        let user = data[12..32].try_into().map_err(|_| "Invalid user address length")?;
        let src_token = data[44..64].try_into().map_err(|_| "Invalid source token address length")?;
        let src_amount = Uint256::from_be_bytes(data[64..96].try_into().unwrap());
        let dst_token = data[108..128].try_into().map_err(|_| "Invalid destination token address length")?;
        let dst_amount = Uint256::from_be_bytes(data[128..160].try_into().unwrap());

        Ok(LiquidityRequestEvent {
            user,
            src_token,
            src_amount,
            dst_token,
            dst_amount,
        })
    }
}

#[cw_serde]
pub struct Fill {
    pub user: [u8; 20],       // User address for whom the swap is being filled
    pub src_token: [u8; 20],  // Address of the source token (ERC20)
    pub dst_token: [u8; 20],  // Address of the destination token (ERC20)
}

impl Fill {
    pub fn serialize(&self) -> Vec<u8> {
        // Selector: first 4 bytes of the given signature hash
        let selector: [u8; 4] = [0xa7, 0x6d, 0xb3, 0xa9];

        // Helper function to pad and encode an address (20 bytes)
        fn encode_address(address: &[u8; 20]) -> Vec<u8> {
            let mut padded = vec![0u8; 12]; // 12 bytes of leading zeros
            padded.extend_from_slice(address); // Append the 20-byte address
            padded
        }

        // Serialize the fields
        let mut serialized = Vec::new();
        serialized.extend_from_slice(&selector); // Add the selector
        serialized.extend_from_slice(&encode_address(&self.user)); // Add user
        serialized.extend_from_slice(&encode_address(&self.src_token)); // Add src_token
        serialized.extend_from_slice(&encode_address(&self.dst_token)); // Add dst_token

        serialized
    }
}