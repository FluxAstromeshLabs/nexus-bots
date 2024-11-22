use astromesh::{MsgAstroTransfer, ACTION_COSMOS_INVOKE, PLANE_COSMOS, PLANE_EVM};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    coin, entry_point, from_json, to_json_binary, to_json_vec, Binary, Coin, Deps, DepsMut, Env,
    HexBinary, Int64, MessageInfo, Response, StdResult, Uint128, Uint256, Uint64,
};
use evm::{erc20_approve, fill, parse_addr, Fill, LiquidityRequestEvent};
use std::{collections::BTreeMap, str::FromStr, vec::Vec};
mod astromesh;
mod evm;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
pub struct QueryMsg {
    msg: Binary,
    fis_input: Vec<FisInput>,
}

#[cw_serde]
pub struct InterpoolResponse {
    pub pool: InterPool,
}

#[cw_serde]
pub struct CommissionConfig {
    pub management_fee_rate: Int64,     // Rate for management fee
    pub management_fee_interval: Int64, // Interval for applying management fee
    pub trading_fee_rate: Int64,        // Rate for trading fee
}

#[cw_serde]
pub struct CommissionFees {
    pub management_fees: Vec<Coin>, // Fees collected for management
    pub trading_fees: Vec<Coin>,    // Fees collected for trading
}

#[cw_serde]
pub struct MsgUpdatePool {
    #[serde(rename = "@type")]
    pub ty: String,
    pub sender: String,              // Sender address
    pub pool_id: String,             // Pool ID
    pub input_blob: Vec<u8>,         // Input blob for processing
    pub output_blob: Vec<u8>,        // Output blob for results
    pub charge_management_fee: bool, // Flag to charge management fee
    pub trading_fee: Vec<Coin>,      // Trading fees to be charged
    pub cron_id: String,             // ID of the cron job associated with this update
}

#[cw_serde]
pub struct InterPool {
    pub pool_id: String,               // HexBytes equivalent as a hex string
    pub operator_addr: String,         // Address of the pool operator
    pub inventory_snapshot: Vec<Coin>, // Ongoing assets in the pool
    pub base_capital: Vec<Coin>,       // Initial assets before any trades
    pub operator_commission_config: Option<CommissionConfig>, // Commission percentage for the operator
    pub operator_commission_fees: Option<CommissionFees>, // Commission percentage for the operator
    pub input_blob: Option<Binary>,                       // Flow control data for cron service
    pub output_blob: Option<Binary>,                      // Extra state for LP, reward tokens
    pub cron_id: String,                                  // Cron job controlling the pool
    pub pool_account: String,
    pub next_commission_time: Uint64,
}

#[cw_serde]
pub struct EmitLogEvent {
    pub op: String,          // Operation code
    pub address: String,     // Address related to the event
    pub topics: Vec<Binary>, // List of topics as Binary
    pub data: Binary,        // Data as Binary
}

#[cw_serde]
pub struct FisInput {
    data: Vec<Binary>,
}

#[cw_serde]
pub struct CronInput {
    receiver: String,
    amount: String,
    denom: String,
}

#[cw_serde]
pub struct FISInstruction {
    plane: String,
    action: String,
    address: String,
    msg: Vec<u8>,
}

#[cw_serde]
pub struct StrategyOutput {
    instructions: Vec<FISInstruction>,
}

#[cw_serde]
pub struct MsgSend {
    from_address: String,
    to_address: String,
    amount: Vec<Coin>,
}

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[entry_point]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> StdResult<Response> {
    Ok(Response::new().add_attribute("method", "execute"))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    // parse cron input
    let pool_input = msg.fis_input.get(0).unwrap().data.get(0).unwrap();
    let pool_info = from_json::<InterpoolResponse>(pool_input)?;
    if pool_info.pool.inventory_snapshot.is_empty() {
        return Ok(to_json_binary(&StrategyOutput {
            instructions: vec![],
        })
        .unwrap());
    }

    let (_, pool_account_bz) = bech32::decode(pool_info.pool.pool_account.as_str()).unwrap();

    let mut coin_map = BTreeMap::new();
    for snapshot in pool_info.pool.inventory_snapshot {
        coin_map.insert(snapshot.denom, snapshot.amount);
    }

    let event_inputs = &msg.fis_input.get(1).unwrap().data;
    let mut instructions = vec![];
    for e in event_inputs {
        let parsed_event = from_json::<EmitLogEvent>(e)?;
        if parsed_event.topics.len() < 1 {
            continue;
        }

        if !parsed_event.topics[0]
            .to_vec()
            .eq(LiquidityRequestEvent::SIGNATURE)
        {
            continue;
        }

        // TODO: fill them as long as pool has enough money
        let liquidity_request = LiquidityRequestEvent::from_bytes(&parsed_event.data)?;
        let denom_hex = HexBinary::from(liquidity_request.dst_token).to_string();
        let pool_denom_dst = match evm::denom_to_cosmos(denom_hex.as_str()) {
            Ok(d) => d,
            _ => continue,
        };
        let existing_fund = Uint256::from_u128(coin_map.get(pool_denom_dst).unwrap().u128());
        if existing_fund.lt(&liquidity_request.dst_amount) {
            continue;
        }

        deps.api
            .debug(format!("accepted liquidity request: {:?}", liquidity_request).as_str());
        // fill the order = transfer funds + approve contract to spend money + fill + transfer back the amount to pool
        // MsgAstroTransfer::new(sender, receiver, src_plane, dst_plane, coin)
        let dst_amount =
            Uint128::from_str(liquidity_request.dst_amount.to_string().as_str()).unwrap();
        let transfer_to_evm = FISInstruction {
            plane: PLANE_COSMOS.to_string(),
            action: ACTION_COSMOS_INVOKE.to_string(),
            address: "".to_string(),
            msg: to_json_vec(&MsgAstroTransfer::new(
                pool_info.pool.pool_account.clone(),
                pool_info.pool.pool_account.clone(),
                PLANE_COSMOS.to_string(),
                PLANE_EVM.to_string(),
                Coin {
                    denom: pool_denom_dst.to_string(),
                    amount: dst_amount,
                },
            ))
            .unwrap(),
        };

        let liquidity_contract = parse_addr(parsed_event.address.as_str());
        let approve = FISInstruction {
            plane: PLANE_EVM.to_string(),
            action: ACTION_COSMOS_INVOKE.to_string(),
            address: "".to_string(),
            msg: to_json_vec(&erc20_approve(
                &pool_info.pool.pool_account,
                &parse_addr(parsed_event.address.as_str()),
                &pool_account_bz.clone().try_into().unwrap(),
                liquidity_request.dst_amount,
            )?)
            .unwrap(),
        };

        // Fill
        let fill = FISInstruction {
            plane: PLANE_EVM.to_string(),
            action: ACTION_COSMOS_INVOKE.to_string(),
            address: "".to_string(),
            msg: to_json_vec(&fill(
                &pool_info.pool.pool_account,
                &liquidity_contract,
                liquidity_request.user,
                liquidity_request.src_token,
                liquidity_request.dst_token,
            )?)
            .unwrap(),
        };

        // MsgAstroTransfer back
        let transfer_to_cosmos = FISInstruction {
            plane: PLANE_COSMOS.to_string(),
            action: ACTION_COSMOS_INVOKE.to_string(),
            address: "".to_string(),
            msg: to_json_vec(&MsgAstroTransfer::new(
                pool_info.pool.pool_account.clone(),
                pool_info.pool.pool_account.clone(),
                PLANE_EVM.to_string(),
                PLANE_COSMOS.to_string(),
                Coin {
                    denom: format!(
                        "astro/{}",
                        HexBinary::from(liquidity_request.dst_token).to_hex()
                    ),
                    amount: dst_amount,
                },
            ))
            .unwrap(),
        };

        instructions.extend(vec![transfer_to_evm, approve, fill, transfer_to_cosmos]);
    }

    // parse command, we can store it as proto bytes, encrypted binary
    Ok(to_json_binary(&StrategyOutput { instructions }).unwrap())
}
