use crate::prelude::Result;
use anyhow::anyhow;
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

pub trait Writable {
    fn write(&self, w: &mut dyn Write) -> Result<()>;
}

pub trait Loadable: Sized {
    fn read_from(r: &mut dyn Read) -> Result<Self>;
}

impl<T: Writable> Writable for Option<T> {
    fn write(&self, w: &mut dyn Write) -> Result<()> {
        match self {
            Some(x) => {
                w.write_u8(1)?;
                x.write(w)?;
            }
            None => w.write_u8(0)?,
        }
        Ok(())
    }
}

impl<T: Loadable> Loadable for Option<T> {
    fn read_from(r: &mut dyn Read) -> Result<Self> {
        Ok(match r.read_u8()? {
            0 => None,
            1 => Some(T::read_from(r)?),
            x => return Err(anyhow!("unhandled Option<T> discriminator {x}")),
        })
    }
}
