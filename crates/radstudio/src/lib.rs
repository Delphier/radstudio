pub mod brcc;
pub mod consts;
pub mod msbuild;

mod discovery;

pub use discovery::find;
pub use discovery::{Architecture, Architectures, Platform, Platforms};
pub use discovery::{CommandLineTool, Edition, Personality};
pub use discovery::{Installation, Installations};
