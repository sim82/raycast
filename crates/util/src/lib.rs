pub mod fa;
pub mod fp16;
pub mod ms;
pub use anyhow::Result;

pub mod prelude {
    pub use crate::fp16::Fp16;
    pub use crate::Result;
}
