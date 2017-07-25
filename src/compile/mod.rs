pub mod bytes;
pub mod optimize;

use errors::*;

/// A general compiler trait.
pub trait Compile {
    type Out;

    fn compile(self) -> Result<Self::Out>;
}

pub use bytes::*;
pub use optimize::*;
