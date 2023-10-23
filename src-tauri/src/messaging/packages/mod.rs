pub enum Packets {
    DisconnectMultipleConnections,
    VerifyIdentity(Vec<u8>),
    IdentityVerified,
    PublicKey,
    Message
}

impl Packets {
    pub fn id(&self) -> u8 {
        match self {
            Packets::DisconnectMultipleConnections => todo!(),
            Packets::VerifyIdentity(_) => todo!(),
            Packets::IdentityVerified => todo!(),
            Packets::PublicKey => todo!(),
            Packets::Message => todo!(),
        }
    }
}

impl From<&[u8]> for Packets {
    fn from(value: &[u8]) -> Self {
        let id = value[0];
        
    }
}