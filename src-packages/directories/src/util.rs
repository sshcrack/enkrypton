use std::time::SystemTime;

pub fn now_millis() -> u128 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u128
}