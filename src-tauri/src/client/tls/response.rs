use anyhow::Result;

pub struct Response {
    buffer: Vec<u8>,
}

impl Response {
    pub(super) fn from_bytes(bytes: Vec<u8>) -> Self {
        return Response { buffer: bytes };
    }

    pub fn text(&self) -> Result<String> {
        let str_res = String::from_utf8(self.buffer)?;

        return Ok(str_res);
    }
}
