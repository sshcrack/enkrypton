mod check;
mod hostname;
mod alive;
mod splashscreen_closed;

pub use hostname::tor_hostname;
pub use check::tor_check;
pub use alive::*;
pub use splashscreen_closed::*;