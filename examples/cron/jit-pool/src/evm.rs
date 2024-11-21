use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, HexBinary, StdError, Uint256};
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
    pub const SIGNATURE: &[u8] = &[1];

    pub fn from_bytes(data: &[u8]) -> Result<Self, StdError> {
        if data.len() != 160 {
            return Err(StdError::generic_err(format!(
                "Invalid data length: expected 160 bytes, got {}",
                data.len()
            )));
        }

        // Extract 32-byte chunks and convert them to the appropriate types
        let user = data[12..32].try_into().map_err(|_| StdError::generic_err("Invalid user address length"))?;
        let src_token = data[44..64].try_into().map_err(|_| StdError::generic_err("Invalid source token address length"))?;
        let src_amount = Uint256::from_be_bytes(data[64..96].try_into().unwrap());
        let dst_token = data[108..128].try_into().map_err(|_| StdError::generic_err("Invalid destination token address length"))?;
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

#[cw_serde]
pub struct MsgExecuteContract {
    pub sender: String,
    pub contract_address: Binary,
    pub calldata: Binary,
    pub input_amount: Binary,
}

impl MsgExecuteContract {
    pub fn new(
        sender: String,
        contract_address: Binary,
        calldata: Binary,
        input_amount: Binary,
    ) -> Self {
        MsgExecuteContract {
            sender,
            contract_address,
            calldata,
            input_amount,
        }
    }
}


fn left_pad(input: &[u8], expected_len: usize) -> Result<Vec<u8>, StdError> {
    if input.len() > expected_len {
        return Err(StdError::generic_err(
            "input len must not exceeds expected len",
        ));
    }

    let mut padded = vec![0u8; expected_len];
    let start_index = expected_len - input.len();
    padded[start_index..].copy_from_slice(input);

    Ok(padded)
}

fn erc20_approve(
    sender: &String,
    erc20_addr: &[u8; 20],
    delegator: &[u8; 20],
    amount: Uint256,
) -> Result<MsgExecuteContract, StdError> {
    let signature: [u8; 4] = (0x095ea7b3u32).to_be_bytes();
    let padded_delegator = left_pad(delegator, 32)?;
    let amount_bytes = amount.to_be_bytes();
    let mut calldata = Vec::new();
    calldata.extend(signature);
    calldata.extend(padded_delegator);
    calldata.extend(amount_bytes);

    let msg = MsgExecuteContract::new(
        sender.clone(),
        Binary::new(erc20_addr.to_vec()),
        Binary::from(calldata),
        Binary::from(vec![]),
    );

    Ok(msg)
}


fn fill(
    sender: &String,
    contract_address: [u8; 20],
    user: [u8; 20],
    src_token: [u8; 20],
    dst_token: [u8; 20],
) -> Result<MsgExecuteContract, StdError> {
    let fill = Fill {
        user,
        src_token,
        dst_token,
    };

    let msg = MsgExecuteContract::new(
        sender.clone(),
        Binary::new(contract_address.to_vec()),
        Binary::from(fill.serialize()),
        Binary::from(vec![]),
    );

    Ok(msg)
}


fn parse_addr(addr: &str) -> [u8; 20] {
    let hex_binary = HexBinary::from_hex(addr).unwrap();
    hex_binary.to_array().unwrap()
}

pub fn denom_to_cosmos(alias: &str) -> Result<&str, StdError> {
    match alias {
        "0c7bd7e65621073f481c5a6cc33876b7fd552c2a" => Ok("btc"),
        "1a38c7b3f073c038cc7e0e92648e15dd36485259" => Ok("usdt"),
        "eef74ab95099c8d1ad8de02ba6bdab9cbc9dbf93" => Ok("sol"),
        "d1738300cda711f4e4c6989856c6b83326c6053e" => Ok("eth"),
        _ => Err(StdError::generic_err(format!(
            "unknown evm denom: {}",
            alias
        ))),
    }
}