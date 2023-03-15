use crate::prelude::Result;
use std::io::{Read, Write};

pub trait Writable {
    fn write(&self, w: &mut dyn Write) -> Result<()>;
}

pub trait Loadable: Sized {
    fn read_from(r: &mut dyn Read) -> Result<Self>;
}
