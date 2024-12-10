use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub enum Config {
    Deploy = 0,
    Update = 1,
    Disable = 2,
    Enable = 3,
    Revoke = 4,
}

#[cw_serde]
pub struct MsgConfigStrategy {
    #[serde(rename = "@type")]
    pub ty: String,
    pub sender: String,
    pub config: Config,
    pub id: String,
    pub strategy: Vec<u8>,
    pub query: Option<FISQueryRequest>,
    pub trigger_permission: Option<PermissionConfig>,
    pub metadata: Option<StrategyMetadata>,
}

impl MsgConfigStrategy {
    pub fn new(
        sender: String,
        config: Config,
        id: String,
        strategy: Vec<u8>,
        query: Option<FISQueryRequest>,
        trigger_permission: Option<PermissionConfig>,
        metadata: Option<StrategyMetadata>,
    ) -> Self {
        MsgConfigStrategy {
            ty: "/flux.strategy.v1beta1.MsgConfigStrategy".to_string(),
            sender,
            config,
            id,
            strategy,
            query,
            trigger_permission,
            metadata,
        }
    }
}

#[cw_serde]
pub struct FISQueryRequest {
    pub instructions: Vec<FISQueryInstruction>,
}

impl FISQueryRequest {
    pub fn new(instructions: Vec<FISQueryInstruction>) -> Self {
        FISQueryRequest { instructions }
    }
}

#[cw_serde]
pub struct FISQueryInstruction {
    pub plane: String,
    pub action: String,
    pub address: Vec<u8>,
    pub input: Vec<Vec<u8>>,
}

impl FISQueryInstruction {
    pub fn new(plane: String, action: String, address: Vec<u8>, input: Vec<Vec<u8>>) -> Self {
        FISQueryInstruction {
            plane,
            action,
            address,
            input,
        }
    }
}

#[cw_serde]
pub struct PermissionConfig {
    #[serde(rename = "type")]
    pub ty: String,
    pub addresses: Vec<String>,
}

impl PermissionConfig {
    pub fn new(access_type: String, addresses: Vec<String>) -> Self {
        PermissionConfig {
            ty: access_type,
            addresses,
        }
    }
}

#[cw_serde]
pub struct StrategyMetadata {
    pub name: String,
    pub description: String,
    pub logo: String,
    pub website: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub tags: Vec<String>,
    pub schema: String,
    pub aggregated_query_keys: Vec<String>,
    pub cron_gas_price: Uint128,
    pub cron_input: String,
    pub cron_interval: u64,
    pub supported_apps: Vec<SupportedApp>,
}

impl StrategyMetadata {
    pub fn new(
        name: String,
        description: String,
        logo: String,
        website: String,
        ty: String,
        tags: Vec<String>,
        schema: String,
        aggregated_query_keys: Vec<String>,
        cron_gas_price: Uint128,
        cron_input: String,
        cron_interval: u64,
        supported_apps: Vec<SupportedApp>,
    ) -> Self {
        StrategyMetadata {
            name,
            description,
            logo,
            website,
            ty,
            tags,
            schema,
            aggregated_query_keys,
            cron_gas_price,
            cron_input,
            cron_interval,
            supported_apps,
        }
    }
}

#[cw_serde]
pub struct SupportedApp {
    pub name: String,
}

impl SupportedApp {
    pub fn new(name: String) -> Self {
        SupportedApp { name }
    }
}
