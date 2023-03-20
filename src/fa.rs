use crate::prelude::{Fp16, WIDTH};
use lazy_static::lazy_static;
use std;

pub const FA_SCALEF: f32 = 10.0;
pub const PIS_IN_180: f32 = 57.295_78_f32;
pub const FA_TAU: i32 = (std::f32::consts::TAU * PIS_IN_180 * FA_SCALEF) as i32; // .to_degrees() method is not constexpr
pub const FA_PI: i32 = (std::f32::consts::PI * PIS_IN_180 * FA_SCALEF) as i32;
pub const FA_FRAC_PI_2: i32 = (std::f32::consts::FRAC_PI_2 * PIS_IN_180 * FA_SCALEF) as i32;
pub const FA_FRAC_PI_4: i32 = (std::f32::consts::FRAC_PI_4 * PIS_IN_180 * FA_SCALEF) as i32;
pub const FA_PI_FRAC_PI_2: i32 = ((std::f32::consts::PI + std::f32::consts::FRAC_PI_2) * PIS_IN_180 * FA_SCALEF) as i32;
pub const QUADRANT_1: std::ops::Range<i32> = 0..FA_FRAC_PI_2;
pub const QUADRANT_2: std::ops::Range<i32> = FA_FRAC_PI_2..FA_PI;
pub const QUADRANT_3: std::ops::Range<i32> = FA_PI..FA_PI_FRAC_PI_2;
pub const QUADRANT_4: std::ops::Range<i32> = (FA_PI_FRAC_PI_2)..(FA_TAU);
pub const TAN_CLAMP: f32 = 128.0;
pub const FA_STEPS: usize = 3600;

lazy_static! {
    pub static ref COL_ANGLE: [i32; WIDTH] = {
        let mut col_angle = [0; WIDTH];
        for i in 0..WIDTH / 2 {
            let f = (i as f32) / (WIDTH as f32 * 0.50);
            col_angle[WIDTH / 2 + i] = (f.atan().to_degrees() * FA_SCALEF) as i32;
            col_angle[WIDTH / 2 - i - 1] = ((-f.atan()).to_degrees() * FA_SCALEF) as i32;
        }
        col_angle
    };
    static ref SIN_TABLE: [Fp16; FA_STEPS] = {
        let mut t = [Fp16::default(); FA_STEPS];
        #[allow(clippy::needless_range_loop)]
        for v in 0..FA_STEPS {
            t[v] = (v as f32 / FA_SCALEF).to_radians().sin().into();
        }
        t
    };
    static ref COS_TABLE: [Fp16; FA_STEPS] = {
        let mut t = [Fp16::default(); FA_STEPS];
        #[allow(clippy::needless_range_loop)]
        for v in 0..FA_STEPS {
            t[v] = (v as f32 / FA_SCALEF).to_radians().cos().into();
        }
        t
    };
    static ref TAN_TABLE: [Fp16; FA_STEPS] = {
        let mut t = [Fp16::default(); FA_STEPS];
        #[allow(clippy::needless_range_loop)]
        for v in 0..FA_STEPS {
            t[v] = (v as f32 / FA_SCALEF)
                .to_radians()
                .tan()
                .clamp(-TAN_CLAMP, TAN_CLAMP)
                .into();
        }
        t
    };
    static ref COT_TABLE: [Fp16; FA_STEPS] = {
        let mut t = [Fp16::default(); FA_STEPS];
        #[allow(clippy::needless_range_loop)]
        for v in 0..FA_STEPS {
            t[v] = (1.0 / (v as f32 / FA_SCALEF).to_radians().tan())
                .clamp(-TAN_CLAMP, TAN_CLAMP)
                .into()
        }
        t
    };
}

pub fn fa_sin(v: i32) -> Fp16 {
    SIN_TABLE[v as usize]
}

pub fn fa_cos(v: i32) -> Fp16 {
    COS_TABLE[v as usize]
}

pub fn fa_tan(v: i32) -> Fp16 {
    TAN_TABLE[v as usize]
}

pub fn fa_cot(v: i32) -> Fp16 {
    COT_TABLE[v as usize]
}

pub fn fa_fix_angle(mut v: i32) -> i32 {
    while v < 0 {
        v += FA_TAU;
    }

    while v >= FA_TAU {
        v -= FA_TAU;
    }
    v
}
