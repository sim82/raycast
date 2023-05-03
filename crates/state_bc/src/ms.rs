use crate::prelude::Result;
use anyhow::anyhow;
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

use self::we::WriteExt;

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
        Ok(match r.read_u8()? {
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
        let len = r.read_u8()? as usize;
        let mut name = vec![0u8; len];
        r.read_exact(&mut name)?;
        Ok(String::from_utf8(name)?)
    }
}

pub mod we {
    use crate::prelude::Result;
    use std::{fmt::Write, io};

    pub trait WriteExt: io::Write {
        #[inline]
        fn writeu8(&mut self, v: u8) -> Result<()> {
            Ok(self.write_all(&v.to_le_bytes())?)
        }
        #[inline]
        fn writei32(&mut self, v: i32) -> Result<()> {
            Ok(self.write_all(&v.to_le_bytes())?)
        }
    }
    impl<W: io::Write + ?Sized> WriteExt for W {}
    pub trait ReadExt: io::Read {
        #[inline]
        fn readu8(&mut self) -> Result<u8> {
            let mut buf = [0u8; 1];
            self.read_exact(&mut buf)?;
            Ok(u8::from_le_bytes(buf))
        }
        #[inline]
        fn readi32(&mut self) -> Result<i32> {
            let mut buf = [0u8; 4];
            self.read_exact(&mut buf)?;
            Ok(i32::from_le_bytes(buf))
        }
    }
    impl<R: io::Read + ?Sized> ReadExt for R {}
}
