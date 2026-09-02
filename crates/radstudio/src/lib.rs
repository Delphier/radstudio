pub mod bds;
pub mod brcc;
pub mod consts;
pub mod dcc;
pub mod msbuild;

mod discovery;

pub use discovery::find;
pub use discovery::{Architecture, Architectures, Platform, Platforms};
pub use discovery::{CommandLineTool, Edition, Personality};
pub use discovery::{Installation, Installations};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    XmlParse(#[from] xmltree::ParseError),
    #[error(transparent)]
    XmlWrite(#[from] xmltree::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
