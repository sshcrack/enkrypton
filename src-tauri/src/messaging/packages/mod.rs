use actix_web_actors::ws::Message as ActixMessage;
use bincode::ErrorKind;
use duplicate::duplicate_item;
use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;

mod client_2_server;
mod server_2_client;

pub use client_2_server::*;
pub use server_2_client::*;


#[duplicate_item(name; [C2SPacket]; [S2CPacket])]
impl TryInto<Vec<u8>> for name {
    type Error = Box<ErrorKind>;

    fn try_into(self) -> std::result::Result<Vec<u8>, Self::Error> {
        bincode::serialize(&self)
    }
}

#[duplicate_item(name; [C2SPacket]; [S2CPacket])]
impl TryFrom<&Vec<u8>> for name {
    type Error = Box<ErrorKind>;

    fn try_from(bin: &Vec<u8>) -> std::result::Result<Self, Self::Error> {
        bincode::deserialize(bin)
    }
}

impl TryInto<TungsteniteMessage> for C2SPacket {
    type Error = Box<ErrorKind>;

    fn try_into(self) -> std::result::Result<TungsteniteMessage, Self::Error> {
        let res = TungsteniteMessage::Binary(self.try_into()?);
        Ok(res)
    }
}

impl TryInto<ActixMessage> for S2CPacket {
    type Error = Box<ErrorKind>;

    fn try_into(self) -> Result<ActixMessage, Self::Error> {
        let res: Vec<u8> = self.try_into()?;

        Ok(ActixMessage::Binary(res.into()))
    }
}