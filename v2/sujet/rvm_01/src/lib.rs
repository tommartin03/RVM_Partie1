pub mod cli;
pub mod error;
pub mod instruction;
pub mod value;
pub mod vm;
pub mod memory;

#[allow(unused_imports)]
mod inner_prelude {
    pub use crate::error::*;
    pub use crate::instruction::*;
    pub use crate::value::*;
    pub use crate::vm::*;
    pub use crate::memory::*;
}
