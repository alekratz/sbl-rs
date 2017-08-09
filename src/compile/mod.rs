pub mod bytes;
pub mod bake;

use errors::*;

/// A general compiler trait.
pub trait Compile {
    type Out;

    fn compile(self) -> Result<Self::Out>;
}

pub trait Optimize {
    type Out;

    fn optimize(self) -> Self::Out;
}

pub use self::bytes::*;
pub use self::bake::*;
