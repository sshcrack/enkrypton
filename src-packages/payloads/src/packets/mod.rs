use actix_web::web::Bytes;
use actix_web_actors::ws::Message as ActixMessage;
use bincode::ErrorKind;
use duplicate::duplicate_item;
//noinspection SpellCheckingInspection
use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;

mod client_2_server;
mod server_2_client;
mod identity;

pub use identity::*;
pub use client_2_server::*;
pub use server_2_client::*;


/// A trait that allows for easy conversion between the packets and bytes
#[duplicate_item(name; [C2SPacket]; [S2CPacket])]
impl TryInto<Vec<u8>> for name {
    type Error = Box<ErrorKind>;

    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        bincode::serialize(&self)
    }
}


/// A trait that allows for easy conversion between the packets and bytes
#[duplicate_item(name; [C2SPacket]; [S2CPacket])]
impl TryInto<Bytes> for name {
    type Error = Box<ErrorKind>;

    fn try_into(self) -> Result<Bytes, Self::Error> {
        let b: Vec<u8> = self.try_into()?;
        Ok(Bytes::from(b))
    }
}

/// A trait that allows for easy conversion between the packets and bytes
#[duplicate_item(name; [C2SPacket]; [S2CPacket])]
impl TryFrom<&Vec<u8>> for name {
    type Error = Box<ErrorKind>;

    fn try_from(bin: &Vec<u8>) -> Result<Self, Self::Error> {
        bincode::deserialize(bin)
    }
}

/// A trait that allows for easy conversion between the packets and bytes
impl TryInto<TungsteniteMessage> for C2SPacket {
    type Error = Box<ErrorKind>;

    fn try_into(self) -> Result<TungsteniteMessage, Self::Error> {
        let res = TungsteniteMessage::Binary(self.try_into()?);
        Ok(res)
    }
}

/// A trait that allows for easy conversion between the packets and `ActixMessages`
impl TryInto<ActixMessage> for S2CPacket {
    type Error = Box<ErrorKind>;

    fn try_into(self) -> Result<ActixMessage, Self::Error> {
        let res: Vec<u8> = self.try_into()?;

        Ok(ActixMessage::Binary(res.into()))
    }
}