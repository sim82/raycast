use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::{ms, Result};

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Fp16 {
    pub v: i32,
}

pub const FP16_SCALE: i32 = 16;
pub const FP16_F: f32 = (1 << FP16_SCALE) as f32;

pub const FP16_ZERO: Fp16 = Fp16 { v: 0 };
pub const FP16_ONE: Fp16 = Fp16 { v: 1 << FP16_SCALE };
pub const FP16_HALF: Fp16 = Fp16 {
    v: (1 << FP16_SCALE) / 2,
};

pub const FP16_FRAC_64: Fp16 = Fp16 {
    v: 1 << (FP16_SCALE - 6),
};

impl From<f32> for Fp16 {
    fn from(f: f32) -> Self {
        let e = if f >= 0.0 { 0.5 } else { -0.5 };
        Self {
            v: (f * FP16_F + e) as i32,
        }
    }
}

impl From<i32> for Fp16 {
    fn from(f: i32) -> Self {
        Self { v: f << FP16_SCALE }
    }
}

impl ms::Writable for Fp16 {
    fn write(&self, w: &mut dyn std::io::Write) -> Result<()> {
        w.write_i32::<LittleEndian>(self.v)?;
        Ok(())
    }
}

impl ms::Loadable for Fp16 {
    fn read_from(r: &mut dyn std::io::Read) -> Result<Self> {
        let v = r.read_i32::<LittleEndian>()?;
        Ok(Self { v })
    }
}

impl Fp16 {
    pub fn get_int(&self) -> i32 {
        self.v >> FP16_SCALE
    }
    pub fn get_fract(&self) -> u32 {
        (self.v as u32) & 0xFFFF // TODO: mask from SCALE
    }

    pub fn fract(&self) -> Fp16 {
        Self { v: self.v & 0xFFFF }
    }
    // pub fn as_f32(&self) -> f32 {
    //     (self.v as f32) / FP16_F
    // }
}

impl Add<Fp16> for Fp16 {
    type Output = Fp16;

    fn add(self, rhs: Fp16) -> Self::Output {
        Self { v: self.v + rhs.v }
    }
}

impl AddAssign<Fp16> for Fp16 {
    fn add_assign(&mut self, rhs: Fp16) {
        self.v += rhs.v;
    }
}

impl Sub<Fp16> for Fp16 {
    type Output = Fp16;

    fn sub(self, rhs: Fp16) -> Self::Output {
        Self { v: self.v - rhs.v }
    }
}
impl SubAssign<Fp16> for Fp16 {
    fn sub_assign(&mut self, rhs: Fp16) {
        self.v -= rhs.v;
    }
}
impl Neg for Fp16 {
    type Output = Fp16;

    fn neg(self) -> Self::Output {
        Self { v: -self.v }
    }
}

impl Mul<Fp16> for Fp16 {
    type Output = Fp16;

    fn mul(self, rhs: Fp16) -> Self::Output {
        // assert!(self.v.abs() <= 0xFFFF || rhs.v.abs() <= 0xFFFF);
        // let sign =
        // let v = (((self.v as i64) * (rhs.v as i64)) >> FP16_SCALE) as i32;
        let v = ((self.v >> 4) * (rhs.v >> 4)) >> 8;
        Self { v }
    }
}

impl Div<Fp16> for Fp16 {
    type Output = Fp16;

    fn div(self, rhs: Fp16) -> Self::Output {
        // FIXME: the scaling factors are completely empirical for the sprite size 1/z division
        let a = self.v << 8;
        let b = rhs.v;
        let c = a / b;
        let v = c << 8;
        Self { v }
    }
}

impl Mul<i32> for Fp16 {
    type Output = Fp16;

    fn mul(self, rhs: i32) -> Self::Output {
        Self { v: self.v * rhs }
    }
}

#[test]
fn test_fp16() {
    let v1: Fp16 = 123.456.into();

    assert_eq!(v1.get_int(), 123);
    assert_eq!(v1.get_fract(), (0.456 * FP16_F) as u32);

    let v2: Fp16 = (-123.456).into();

    assert_eq!(v2.get_int(), -124);
    assert_eq!(v2.get_fract(), ((1.0 - 0.456) * FP16_F) as u32 + 1);
}
