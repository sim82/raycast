use crate::prelude::Result;
use anyhow::anyhow;
use std::io::{Read, Write};

use self::endian::{ReadExt, WriteExt};

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
                w.writeu8(1)?;
                x.write(w)?;
            }
            None => w.writeu8(0)?,
        }
        Ok(())
    }
}

impl<T: Loadable> Loadable for Option<T> {
    fn read_from(r: &mut dyn Read) -> Result<Self> {
        Ok(match r.readu8()? {
            0 => None,
            1 => Some(T::read_from(r)?),
            x => return Err(anyhow!("unhandled Option<T> discriminator {x}")),
        })
    }
}

impl Writable for String {
    fn write(&self, w: &mut dyn Write) -> Result<()> {
        let bytes = self.as_bytes();
        assert!(bytes.len() <= 255);
        w.writeu8(bytes.len() as u8)?;
        w.write_all(bytes)?;
        Ok(())
    }
}

impl Loadable for String {
    fn read_from(r: &mut dyn Read) -> Result<Self> {
        let len = r.readu8()? as usize;
        let mut name = vec![0u8; len];
        r.read_exact(&mut name)?;
        Ok(String::from_utf8(name)?)
    }
}

pub mod endian;
