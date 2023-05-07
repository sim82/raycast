use crate::prelude::Result;
use std::io;

pub trait WriteExt: io::Write {
    #[inline]
    fn writeu8(&mut self, v: u8) -> Result<()> {
        Ok(self.write_all(&v.to_le_bytes())?)
    }
    #[inline]
    fn writeu16(&mut self, v: u16) -> Result<()> {
        Ok(self.write_all(&v.to_le_bytes())?)
    }
    #[inline]
    fn writei32(&mut self, v: i32) -> Result<()> {
        Ok(self.write_all(&v.to_le_bytes())?)
    }
    #[inline]
    fn writeu32(&mut self, v: u32) -> Result<()> {
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
    fn readu16(&mut self) -> Result<u16> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }
    #[inline]
    fn readi32(&mut self) -> Result<i32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(i32::from_le_bytes(buf))
    }
    #[inline]
    fn readu32(&mut self) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }
}
impl<R: io::Read + ?Sized> ReadExt for R {}
