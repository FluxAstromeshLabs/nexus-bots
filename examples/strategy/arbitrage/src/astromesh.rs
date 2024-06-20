use cosmwasm_std::Coin;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    pub fn new(sender: String, receiver: String, src_plane: String, dst_plane: String, coin: Coin) -> Self {
        MsgAstroTransfer {
            ty: "flux.astromesh.v1beta1.AstroTransfer".to_string(),
            sender,
            receiver,
            src_plane,
            dst_plane,
            coin,
        }
    }
}
