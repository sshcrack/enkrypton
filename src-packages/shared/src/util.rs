use std::time::SystemTime;

/// Returns the current time in milliseconds since the unix epoch, equal to `Date.now()` in JS
pub fn now_millis() -> u128 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u128
}


pub fn _get_name<T>(_: &T) -> &'static str {
    std::any::type_name::<T>()
}

/// Used to get the name of a struct
#[macro_export]
macro_rules! name_struct {
    ($e:expr) => {
        _get_name(&$e)
    }
}