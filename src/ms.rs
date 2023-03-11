use std::io::Write;

pub trait Writable {
    fn write(&self, w: &mut dyn Write);
}
