pub mod bytes;
pub mod bake;
pub mod ir;
pub mod graph;

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
pub use self::ir::*;
pub use self::graph::*;
