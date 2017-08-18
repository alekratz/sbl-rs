pub mod bytes;
// TODO(IR)
//pub mod bake;
pub mod ir;

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
// TODO(IR)
//pub use self::bake::*;
pub use self::ir::*;
