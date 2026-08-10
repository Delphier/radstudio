pub mod consts;
pub mod msbuild;

mod discovery;

pub use discovery::find;
pub use discovery::{Architecture, Platform};
pub use discovery::{Installation, Installations};
