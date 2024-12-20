use astromesh::{
    keccak256, sha256, AccountResponse, CommissionConfig, FISInput, FISInstruction, InitialMint,
    MsgAstroTransfer, MsgCreateBankDenom, MsgCreatePool, MsgUpdatePool, NexusAction, PLANE_COSMOS,
    PLANE_EVM, PLANE_SVM, QUERY_ACTION_COSMOS_ASTROMESH_BALANCE, QUERY_ACTION_COSMOS_BANK_BALANCE,
    QUERY_ACTION_COSMOS_KVSTORE, QUERY_ACTION_COSMOS_QUERY,
};
use bech32::{Bech32, Bech32m, Hrp};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, to_json_vec, BankMsg, Binary, Coin, Decimal,
    DenomMetadata, DenomUnit, Deps, DepsMut, Env, HexBinary, Int128, MessageInfo, Response,
    StdError, StdResult, Uint128, Uint256, Uint64,
};
use curve::BondingCurve;
use std::vec::Vec;
use strategy::{
    FISQueryInstruction, FISQueryRequest, MsgConfigStrategy, PermissionConfig, StrategyMetadata,
};
mod astromesh;
mod curve;
mod strategy;
mod svm;
mod test;
use serde::{Deserialize, Serialize};

const PERCENTAGE_BPS: u128 = 10_000;
const EMBEDDED_CRON_BINARY: &[u8] = include_bytes!(
    "../../../cron/dumpsad-cron/target/wasm32-unknown-unknown/release/dumpsad_cron.wasm"
);
const INITIAL_AMOUNT: &Uint128 = &Uint128::new(1_000_000_000_000_000_000);

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {}

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

#[cw_serde]
pub struct QueryMsg {
    msg: Binary,
    fis_input: Vec<FISInput>,
}

#[cw_serde]
pub struct StrategyOutput {
    instructions: Vec<FISInstruction>,
}

pub const ACTION_COSMOS_INVOKE: &str = "COSMOS_INVOKE";
pub const ACTION_VM_INVOKE: &str = "VM_INVOKE";

pub const UNISWAP: &str = "uniswap";
pub const POOL_MANAGER: &str = "6ff00f6b2120157fca353fbe24d25536042197df";
pub const POOL_ACTION: &str = "366c9837f9a32cc11ac5cac1602e57b73e6bf784";

pub fn parse_addr(addr: &str) -> [u8; 20] {
    let mut res = [0u8; 20];
    hex::decode_to_slice(addr, res.as_mut_slice()).unwrap();
    res
}

pub fn left_pad(input: &[u8], expected_len: usize) -> Result<Vec<u8>, StdError> {
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

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PoolKey {
    /// @notice The lower currency of the pool, sorted numerically
    currency0: [u8; 20],
    /// @notice The higher currency of the pool, sorted numerically
    currency1: [u8; 20],
    /// @notice The pool swap fee, capped at 1_000_000. If the first bit is 1, the pool has a dynamic fee and must be exactly equal to 0x800000
    fee: u32,
    /// @notice Ticks that involve positions must be a multiple of tick spacing
    tick_spacing: i32,
    /// @notice The hooks of the pool
    hooks: [u8; 20],
}

impl PoolKey {
    pub fn new(
        currency0: [u8; 20],
        currency1: [u8; 20],
        fee: u32,
        tick_spacing: i32,
        hooks: [u8; 20],
    ) -> Self {
        Self {
            currency0,
            currency1,
            fee,
            tick_spacing,
            hooks,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut serialized = Vec::with_capacity(160);
        // Adding padding for currency0
        serialized.extend_from_slice(&[0u8; 12]); // padding to make it 32 bytes
        serialized.extend_from_slice(&self.currency0);

        // Adding padding for currency1
        serialized.extend_from_slice(&[0u8; 12]); // padding to make it 32 bytes
        serialized.extend_from_slice(&self.currency1);

        // Adding padding for fee
        serialized.extend_from_slice(&[0u8; 28]); // padding to make it 32 bytes
        serialized.extend_from_slice(&self.fee.to_be_bytes());

        // Adding padding for tick_spacing
        serialized.extend_from_slice(&[0u8; 28]); // padding to make it 32 bytes
        serialized.extend_from_slice(&self.tick_spacing.to_be_bytes());

        // Adding padding for hooks
        serialized.extend_from_slice(&[0u8; 12]); // padding to make it 32 bytes
        serialized.extend_from_slice(&self.hooks);

        serialized
    }
}

pub fn compute_sqrt_price_x96_int(price: f64) -> i128 {
    let sqrt_price = price.sqrt();
    let scale_factor: f64 = 2_f64.powi(96);
    let sqrt_price_x96_int = sqrt_price * scale_factor;
    sqrt_price_x96_int as i128
}

pub fn compose_erc20_approve(
    sender: &String,
    erc20_addr: &[u8; 20],
    delegator: &[u8; 20],
    amount: Uint256,
) -> Result<FISInstruction, StdError> {
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

    Ok(FISInstruction {
        plane: PLANE_EVM.to_string(),
        action: ACTION_VM_INVOKE.to_string(),
        address: "".to_string(),
        msg: to_json_vec(&msg)?,
    })
}

pub fn initilize(
    deps: Deps,
    sender: String,
    fee: u32,
    price: f64,
    denom_0: String,
    denom_1: String,
) -> Result<FISInstruction, StdError> {
    let signature: [u8; 4] = (0x695c5bf5u32).to_be_bytes();
    let tick_spacing = 60;

    let pool_key = PoolKey::new(
        parse_addr(&denom_0), 
        parse_addr(&denom_1),
        fee,
        tick_spacing,
        [0; 20],
    );
    let sqrt_price_x96_int = compute_sqrt_price_x96_int(price);

    let empty_hook_data = Binary::from_base64(
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAASAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==",
    )
    .unwrap();

    let mut calldata = Vec::new();
    calldata.extend(signature);
    calldata.extend(pool_key.serialize());
    calldata.extend(sqrt_price_x96_int.to_be_bytes());
    calldata.extend(empty_hook_data.iter());

    let msg = MsgExecuteContract::new(
        sender.to_string(),
        Binary::from(hex::decode(POOL_MANAGER).unwrap()),
        Binary::from(calldata),
        Binary::from(vec![]),
    );

    Ok(FISInstruction {
        plane: PLANE_EVM.to_string(),
        action: ACTION_VM_INVOKE.to_string(),
        address: "".to_string(),
        msg: to_json_vec(&msg).unwrap(),
    })
}

// pub fn provide_liquidity(
//     fee: u32,
//     price: f64,
//     sender: String,
//     denom_0: String,
//     denom_1: String,
// ) -> Result<FISInstruction, StdError> {
//     let signature: [u8; 4] = (0x568846efu32).to_be_bytes();
//     let tick_spacing = 60;
//     let pool_key = PoolKey::new(
//         parse_addr(&denom_0),
//         parse_addr(&denom_1),
//         fee,
//         tick_spacing,
//         [0; 20],
//     );
//     let salt: [u8; 32] = [
//         1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0,
//         1, 2,
//     ];

//     let lower_price = price * 0.8;
//     let upper_price = price * 1.2;

//     let tick_lower = compute_tick(lower_price, tick_spacing.into());
//     let tick_upper = compute_tick(upper_price, tick_spacing.into());

//     let modify_liquidity_params = ModifyLiquidityParams::new(
//         tick_lower,
//         tick_upper,
//         1000000000,
//         salt,
//     );
//     let empty_hook_data = Binary::from_base64(
//         "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAASAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==",
//     )
//     .unwrap();

//     let mut calldata = Vec::new();
//     calldata.extend(signature);
//     calldata.extend(pool_key.serialize());
//     calldata.extend(modify_liquidity_params.serialize());
//     calldata.extend(empty_hook_data.iter());

//     let msg = MsgExecuteContract::new(
//         sender.to_string(),
//         Binary::from(hex::decode(POOL_ACTION).unwrap()),
//         Binary::from(calldata),
//         Binary::from(vec![]),
//     );

//     Ok(FISInstruction {
//         plane: PLANE_EVM.to_string(),
//         action: ACTION_VM_INVOKE.to_string(),
//         address: "".to_string(),
//         msg: to_json_vec(&msg).unwrap(),
//     })
// }

fn handle_create_token(
    deps: Deps,
    env: Env,
    name: String,
    description: String,
    uri: String,
    target_vm: String,
    bot_id: String,
    fis_input: &Vec<FISInput>,
) -> StdResult<Vec<FISInstruction>> {
    let creator = env.contract.address.to_string();
    let acc_info = from_json::<AccountResponse>(fis_input.get(0).unwrap().data.get(0).unwrap())?;
    let (_, creator_bz) =
        bech32::decode(&creator).map_err(|e| StdError::generic_err(e.to_string()))?;
    let pool_id_seed = &[
        "pool".as_bytes(),
        creator_bz.as_slice(),
        (acc_info.account.sequence.u64() + 1)
            .to_le_bytes()
            .as_slice(),
    ]
    .concat();
    deps.api.debug(
        format!(
            "pool id seed: {}",
            HexBinary::from(pool_id_seed.as_slice()).to_string()
        )
        .as_str(),
    );

    let pool_id = &keccak256(&pool_id_seed)[12..];
    // TODO: Check cosmwasm std Addr (it used callback/FFI to parse addr, save some gas/perf)
    let pool_address = bech32::encode::<Bech32>(Hrp::parse("lux").unwrap(), pool_id)
        .map_err(|e| StdError::generic_err(e.to_string()))?;

    let denom_base = format!("astromesh/{}/{}", creator.clone(), name);
    let denom_display = name.to_uppercase();
    let denom_symbol = name.to_uppercase();

    // create pool
    let create_pool_msg = MsgCreatePool::new(
        creator.clone(),
        Some(CommissionConfig::new(0i64, 0i64, 0i64)),
    );
    // update the pool input the target vm
    // create denom and mint for the pool
    let create_denom_msg = MsgCreateBankDenom::new(
        creator.clone(),
        DenomMetadata {
            description: description.clone(),
            denom_units: vec![
                DenomUnit {
                    denom: denom_base.clone(),
                    exponent: 0,
                    aliases: vec![name.clone()],
                },
                DenomUnit {
                    denom: denom_display.clone(),
                    exponent: 9,
                    aliases: vec![],
                },
            ],
            base: denom_base.clone(),
            display: denom_display.clone(),
            name: name.clone(),
            symbol: denom_symbol.clone(),
            uri: uri.clone(),
            uri_hash: "".to_string(),
        },
        "".to_string(), // only do initial mint, cannot mint more
        vec![InitialMint {
            address: pool_address.clone(),
            amount: INITIAL_AMOUNT.clone(),
        }],
    );

    deps.api
        .debug(format!("pool_address {}, denom_base {}", pool_address, denom_base).as_str());

    // TODO: Use static sum to save some gas
    let cron_binary_checksum = sha256(EMBEDDED_CRON_BINARY);
    let cron_id = keccak256(
        &[
            creator_bz.as_slice(),
            cron_binary_checksum.as_slice(),
            &(acc_info.account.sequence.u64() + 1).to_le_bytes(),
        ]
        .concat(),
    );
    let create_graduate_cron_msg = MsgConfigStrategy::new(
        creator.clone(),
        strategy::Config::Deploy,
        "".to_string(),
        EMBEDDED_CRON_BINARY.to_vec(),
        Some(FISQueryRequest::new(vec![
            FISQueryInstruction::new(
                PLANE_COSMOS.to_string(),
                QUERY_ACTION_COSMOS_BANK_BALANCE.to_string(),
                vec![],
                vec![
                    format!("{},{}", pool_address, pool_address)
                        .as_bytes()
                        .to_vec(),
                    format!("sol,{}", denom_base).as_bytes().to_vec(),
                ],
            ),
            FISQueryInstruction::new(
                PLANE_COSMOS.to_string(),
                QUERY_ACTION_COSMOS_KVSTORE.to_string(),
                vec![],
                vec![
                    "wasm".as_bytes().to_vec(),
                    [&[4u8], "lastContractId".as_bytes()].concat().to_vec(),
                ],
            ),
            FISQueryInstruction::new(
                PLANE_COSMOS.to_string(),
                QUERY_ACTION_COSMOS_QUERY.to_string(),
                vec![],
                vec![
                    format!("/flux/svm/v1beta1/account_link/cosmos/{}", pool_address)
                        .as_bytes()
                        .to_vec(),
                ],
            ),
            FISQueryInstruction::new(
                PLANE_COSMOS.to_string(),
                QUERY_ACTION_COSMOS_ASTROMESH_BALANCE.to_string(),
                vec![],
                vec![
                    pool_address.clone().as_bytes().to_vec(),
                    denom_base.as_bytes().to_vec(),
                ],
            ),
            FISQueryInstruction::new(
                PLANE_COSMOS.to_string(),
                QUERY_ACTION_COSMOS_QUERY.to_string(),
                vec![],
                vec![format!(
                    "/flux/astromesh/v1beta1/denom_link/{}/{}/{}",
                    PLANE_COSMOS, PLANE_EVM, denom_base
                )
                .as_bytes()
                .to_vec()],
            ),
        ])),
        Some(PermissionConfig::new("anyone".to_string(), vec![])),
        Some(StrategyMetadata {
            name: "graduate cron".to_string(),
            logo: "".to_string(),
            description: "graduate meme coin".to_string(),
            website: "".to_string(),
            ty: "CRON".to_string(),
            tags: vec![],
            schema: "{}".to_string(),
            cron_gas_price: Uint128::from(500_000_000u128),
            aggregated_query_keys: vec![],
            cron_input: format!(
                r#"{{"vm":"{}","pool_address":"{}"}}"#,
                target_vm, pool_address
            ),
            cron_interval: 2,
            supported_apps: vec![],
        }),
    );

    deps.api.debug(
        format!(
            "pool id to update: {}",
            HexBinary::from(pool_id).to_string()
        )
        .as_str(),
    );
    let update_pool_msg = MsgUpdatePool::new(
        creator.clone(),
        HexBinary::from(pool_id).to_string(),
        target_vm.as_bytes().to_vec(),
        vec![],
        false,
        vec![],
        HexBinary::from(cron_id).to_string(),
        bot_id,
    );

    let mut instructions = vec![
        FISInstruction {
            plane: "COSMOS".to_string(),
            action: "COSMOS_INVOKE".to_string(),
            address: "".to_string(),
            msg: to_json_vec(&create_pool_msg)?,
        },
        // FISInstruction {
        //     plane: "COSMOS".to_string(),
        //     action: "COSMOS_INVOKE".to_string(),
        //     address: "".to_string(),
        //     msg: to_json_vec(&create_graduate_cron_msg)?, // cron
        // },
        FISInstruction {
            plane: "COSMOS".to_string(),
            action: "COSMOS_INVOKE".to_string(),
            address: "".to_string(),
            msg: to_json_vec(&create_denom_msg)?,
        },
        FISInstruction {
            plane: "COSMOS".to_string(),
            action: "COSMOS_INVOKE".to_string(),
            address: "".to_string(),
            msg: to_json_vec(&update_pool_msg)?, // TODO: should disable admin permission for creator from this part, as owner can set another bot as driver
        },
    ];

    if target_vm == PLANE_EVM {
        let init_transfer_msg = MsgAstroTransfer::new(
            pool_address.clone(),
            pool_address.to_string(),
            PLANE_COSMOS.to_string(),
            PLANE_EVM.to_string(),
            Coin::new(Uint128::one(), denom_base.clone()),
        );
        instructions.push(FISInstruction {
            plane: "COSMOS".to_string(),
            action: "COSMOS_INVOKE".to_string(),
            address: "".to_string(),
            msg: to_json_vec(&init_transfer_msg)?,
        });
    }

    if target_vm == PLANE_SVM {
        let init_transfer_msg = MsgAstroTransfer::new(
            pool_address.clone(),
            pool_address,
            PLANE_COSMOS.to_string(),
            PLANE_SVM.to_string(),
            Coin::new(Uint128::one(), denom_base),
        );
        instructions.push(FISInstruction {
            plane: "COSMOS".to_string(),
            action: "COSMOS_INVOKE".to_string(),
            address: "".to_string(),
            msg: to_json_vec(&init_transfer_msg)?,
        });
    }
    Ok(instructions)
}

fn handle_buy(
    _deps: Deps,
    env: Env,
    denom: String,
    amount: Uint128,
    slippage: Uint128,
    pool_address: String, // TODO: where can frontend get this pool address?
    fis_input: &Vec<FISInput>,
) -> StdResult<Vec<FISInstruction>> {
    let trader = env.contract.address.clone();
    // load quote amount
    let quote_coin = from_json::<Coin>(fis_input.get(0).unwrap().data.get(0).unwrap())?;
    let meme_coin = from_json::<Coin>(fis_input.get(0).unwrap().data.get(1).unwrap())?;

    // calculate the delta Y
    let mut curve = BondingCurve::default(quote_coin.amount, INITIAL_AMOUNT - meme_coin.amount);
    let pre_price = curve.price();
    let worst_price = pre_price
        .checked_mul(slippage.checked_add(Uint128::new(PERCENTAGE_BPS))?)?
        .checked_div(Uint128::new(PERCENTAGE_BPS))?;

    let bought_amount = curve.buy(amount);
    let post_price = curve.price();
    assert!(
        post_price.lt(&worst_price),
        "slippage exceeds, pre price: {}, worst price: {}, post price: {}",
        pre_price,
        worst_price,
        post_price
    );

    // send quote to vault
    let trader_send_quote = FISInstruction {
        plane: PLANE_COSMOS.to_string(),
        action: "COSMOS_INVOKE".to_string(),
        address: "".to_string(),
        msg: to_json_vec(&MsgAstroTransfer::new(
            trader.to_string(),
            pool_address.clone(),
            PLANE_COSMOS.to_string(),
            PLANE_COSMOS.to_string(),
            Coin {
                denom: quote_coin.denom,
                amount,
            },
        ))?,
    };

    // send meme to trader
    let pool_send_meme = FISInstruction {
        plane: PLANE_COSMOS.to_string(),
        action: "COSMOS_INVOKE".to_string(),
        address: "".to_string(),
        msg: to_json_vec(&MsgAstroTransfer::new(
            pool_address,
            trader.to_string(),
            PLANE_COSMOS.to_string(),
            PLANE_COSMOS.to_string(),
            Coin {
                denom: meme_coin.denom,
                amount: bought_amount,
            },
        ))?,
    };

    Ok(vec![trader_send_quote, pool_send_meme])
}

fn handle_sell(
    deps: Deps,
    env: Env,
    denom: String,
    amount: Uint128,
    slippage: Uint128,
    pool_address: String, // TODO: where can frontend get this pool address?
    fis_input: &Vec<FISInput>,
) -> StdResult<Vec<FISInstruction>> {
    let trader = env.contract.address.clone();
    // Load quote and meme amounts from input
    let quote_coin = from_json::<Coin>(fis_input.get(0).unwrap().data.get(0).unwrap())?;
    let meme_coin = from_json::<Coin>(fis_input.get(0).unwrap().data.get(1).unwrap())?;

    // Initialize bonding curve
    let mut curve = BondingCurve::default(quote_coin.amount, INITIAL_AMOUNT - meme_coin.amount);
    let pre_price = curve.price();
    let worst_price = pre_price
        .checked_mul(slippage.checked_add(Uint128::new(PERCENTAGE_BPS))?)?
        .checked_div(Uint128::new(PERCENTAGE_BPS))?;

    // Calculate sold amount and verify slippage
    let sold_amount = curve.sell(amount);
    let post_price = curve.price();
    assert!(
        post_price >= worst_price,
        "slippage exceeds, pre price: {}, post price: {}",
        pre_price,
        post_price
    );

    // Transfer instructions
    let trader_send_meme = FISInstruction {
        plane: PLANE_COSMOS.to_string(),
        action: "COSMOS_INVOKE".to_string(),
        address: "".to_string(),
        msg: to_json_vec(&MsgAstroTransfer::new(
            trader.to_string(),
            pool_address.clone(),
            PLANE_COSMOS.to_string(),
            PLANE_COSMOS.to_string(),
            Coin {
                denom: meme_coin.denom,
                amount,
            },
        ))?,
    };

    let pool_send_quote = FISInstruction {
        plane: PLANE_COSMOS.to_string(),
        action: "COSMOS_INVOKE".to_string(),
        address: "".to_string(),
        msg: to_json_vec(&MsgAstroTransfer::new(
            pool_address,
            trader.to_string(),
            PLANE_COSMOS.to_string(),
            PLANE_COSMOS.to_string(),
            Coin {
                denom: quote_coin.denom,
                amount: sold_amount,
            },
        ))?,
    };

    Ok(vec![trader_send_meme, pool_send_quote])
}

pub fn handle_create_pool(
    deps: Deps,
    pool_address: String,
    denom_2: String,
) -> StdResult<Vec<FISInstruction>> {
    let sender = pool_address.clone();

    let mut denom_0 = "eef74ab95099c8d1ad8de02ba6bdab9cbc9dbf93".to_string(); // sol
    let mut amount_0: u128 = 2000000000;

    let mut denom_1 = denom_2;
    let mut amount_1: u128 = 932937488062500000;

    let quote_coin = Coin {
        denom: "sol".to_string(),
        amount: amount_0.into(),
    };

    let meme_coin = Coin {
        denom: "astromesh/lux1jcltmuhplrdcwp7stlr4hlhlhgd4htqhu86cqx/huhu-guy".to_string(),
        amount: amount_1.into(),
    };

    if denom_0 > denom_1 {
        (denom_0, denom_1) = (denom_1, denom_0);
        (amount_0, amount_1) = (amount_1, amount_0);
    }

    let fee = 3000;
    let price = amount_1 as f64 / amount_0 as f64;

    let mut instructions = vec![];
    instructions.extend(vec![
        FISInstruction {
            plane: PLANE_COSMOS.to_string(),
            action: ACTION_COSMOS_INVOKE.to_string(),
            address: "".to_string(),
            msg: to_json_vec(&MsgAstroTransfer::new(
                pool_address.to_string(),
                pool_address.to_string(),
                PLANE_COSMOS.to_string(),
                PLANE_EVM.to_string(),
                Coin {
                    denom: quote_coin.denom,
                    amount: quote_coin.amount,
                },
            ))?,
        },
        FISInstruction {
            plane: PLANE_COSMOS.to_string(),
            action: ACTION_COSMOS_INVOKE.to_string(),
            address: "".to_string(),
            msg: to_json_vec(&MsgAstroTransfer::new(
                pool_address.to_string(),
                pool_address.to_string(),
                PLANE_COSMOS.to_string(),
                PLANE_EVM.to_string(),
                Coin {
                    denom: meme_coin.denom,
                    amount: meme_coin.amount,
                },
            ))?,
        },
    ]);

    instructions.push(compose_erc20_approve(
        &pool_address.to_string(),
        &parse_addr(&denom_1),
        &parse_addr(&POOL_ACTION),
        amount_0.into(),
    )?);

    instructions.push(
        initilize(
            deps,
            sender.to_string(),
            fee,
            price,
            denom_0.to_string(),
            denom_1.to_string(),
        )
        .unwrap(),
    );

    Ok(instructions)
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let nexus_action: NexusAction = from_json(&msg.msg)?;
    let instructions = match nexus_action {
        NexusAction::CreateToken {
            name,
            description,
            uri,
            target_vm,
            bot_id,
        } => handle_create_token(
            deps,
            env,
            name,
            description,
            uri,
            target_vm,
            bot_id,
            &msg.fis_input,
        ),
        NexusAction::Buy {
            denom,
            amount,
            slippage,
            pool_address,
        } => handle_buy(
            deps,
            env,
            denom,
            amount,
            slippage,
            pool_address,
            &msg.fis_input,
        ),
        NexusAction::Sell {
            denom,
            amount,
            slippage,
            pool_address,
        } => handle_sell(
            deps,
            env,
            denom,
            amount,
            slippage,
            pool_address,
            &msg.fis_input,
        ),
        NexusAction::CreatePool {
            pool_address,
            denom_1,
        } => handle_create_pool(deps, pool_address, denom_1),
    }?;

    Ok(to_json_binary(&StrategyOutput { instructions }).unwrap())
}
