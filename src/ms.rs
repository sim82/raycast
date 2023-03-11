use std::io::{Read, Write};

pub trait Writable {
    fn write(&self, w: &mut dyn Write);
}

pub trait Loadable {
    fn read_from(r: &mut dyn Read) -> Self;
}
