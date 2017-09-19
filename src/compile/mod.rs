pub mod bc;
pub mod ir;
pub mod bake;
pub mod graph;
pub mod optimize;

use errors::*;

/// A general compiler trait.
pub trait Compile {
    type Out;

    fn compile(self) -> Result<Self::Out>;
}

pub use self::bc::*;
pub use self::ir::*;
pub use self::bake::*;
pub use self::graph::*;
pub use self::optimize::*;
