use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;

pub const PLANE_EVM: &str = "EVM";
pub const PLANE_COSMOS: &str = "COSMOS";

pub const ACTION_COSMOS_INVOKE: &str = "COSMOS_INVOKE";
pub const ACTION_VM_INVOKE: &str = "VM_INVOKE";

#[cw_serde]
pub struct MsgAstroTransfer {
    #[serde(rename = "@type")]
    pub ty: String,
    pub sender: String,
    pub receiver: String,
    pub src_plane: String,
    pub dst_plane: String,
    pub coin: Coin,
}

impl MsgAstroTransfer {
    pub fn new(
        sender: String,
        receiver: String,
        src_plane: String,
        dst_plane: String,
        coin: Coin,
    ) -> Self {
        MsgAstroTransfer {
            ty: "/flux.astromesh.v1beta1.MsgAstroTransfer".to_string(),
            sender,
            receiver,
            src_plane,
            dst_plane,
            coin,
        }
    }
}
