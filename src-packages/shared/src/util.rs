use std::time::SystemTime;

pub fn now_millis() -> u128 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u128
}


pub fn _get_name<T>(_: &T) -> &'static str {
    std::any::type_name::<T>()
}

#[macro_export]
macro_rules! name_struct {
    ($e:expr) => {
        _get_name(&$e)
    }
}