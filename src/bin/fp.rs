use std::{
    ops::{Add, AddAssign, Mul, Neg, Range, Sub, SubAssign},
    time::Instant,
};

use lazy_static::lazy_static;
use minifb::{Key, Window, WindowOptions};
use raycast::Resources;

const WIDTH: usize = 320;
const HEIGHT: usize = 200;
const COLORS: [u32; 16] = [
    0xFFFFFF, 0xFF0000, 0x00FF00, 0x0000FF, 0xFFFF00, 0x00FFFF, 0xFF00FF, 0xFF8000, 0x808080,
    0x800000, 0x008000, 0x000080, 0x808000, 0x008080, 0x800080, 0x804000,
];

const MAP_SIZE: usize = 64;

#[rustfmt::skip]
const MAP: [&[u8]; MAP_SIZE] = [
    b"1111111111345671723456717234567172345671723456717234567172345671",
    b"1000000000000002700000000000000270000000000000027000000000000002",
    b"1000000000000003600000000000000360000000000000036000000000000003",
    b"1000550000000004500000000000000450000000000000045000000000000004",
    b"1000553000000030000000300000003540000030000000300000003000000035",
    b"1000554000000040000000400000004000000040000000400000004000000046",
    b"1000025000000250000002500000025000000250000002500000025000000257",
    b"1000067002000670020006700200067002000670020006700200067002000677",
    b"1000567002005670020056700200567002005670020056700200567002005671",
    b"1000000000000000000000000000000000000000000000000000000000000002",
    b"1000000000000000000000000000000000000000000000000000000000000003",
    b"5000000000000000000000000000000000000000000000000000000000000004",
    b"4000003000000030000000300000003000000030000000300000003000000035",
    b"3000004000000040000000400000004000000040000000400000004000000046",
    b"2000025000000250000002500000025000000250000002500000025000000257",
    b"2000025000000250000002500000025000000250000002500000025000000257",
    b"2000025000000257200002500000025000000250000002572000025000000257",
    b"7000000000000002700000000000000000000000000000027000000000000002",
    b"6000000000000000000000000000000000000000000000000000000000000003",
    b"5000000000000000000000000000000000000000000000000000000000000004",
    b"4000003000000030000000300000003000000030000000300000003000000035",
    b"3000004000000040000000400000004000000040000000400000004000000046",
    b"2000025000000250000002500000025000000250000002500000025000000257",
    b"1200067002000670020006700200067002000670020006700200067002000677",
    b"7200567002005670020056700200567002005670020056700200567002005671",
    b"7000000000000000000000000000000000000000000000000000000000000002",
    b"6000000000000000000000000000000000000000000000000000000000000003",
    b"5000000000000000000000000000000000000000000000000000000000000004",
    b"4000003000000030000000300000003540000030000000300000003000000035",
    b"3000004000000040000000400000004630000040000000400000004000000046",
    b"2000025000000257200002500000025720000250000002572000025000000257",
    b"5000000000000000000000000000000450000000000000000000000000000004",
    b"4000003000000030000000300000003540000030000000300000003000000035",
    b"7000000000000002700000000000000270000000000000027000000000000002",
    b"6000000000000003600000000000000360000000000000036000000000000003",
    b"5000000000000004500000000000000450000000000000045000000000000004",
    b"4000003000000030000000300000003000000030000000300000003000000035",
    b"3000004000000040000000400000004000000040000000400000004000000046",
    b"2000025000000250000002500000025000000250000002500000025000000257",
    b"1200067002000670020006700200067002000670020006700200067002000677",
    b"7200567002005670020056700200567002005670020056700200567002005671",
    b"7000000000000000000000000000000000000000000000000000000000000002",
    b"6000000000000000000000000000000000000000000000000000000000000003",
    b"5000000000000000000000000000000000000000000000000000000000000004",
    b"4000003000000030000000300000003000000030000000300000003000000035",
    b"3000004000000040000000400000004000000040000000400000004000000046",
    b"2000025000000250000002500000025000000250000002500000025000000257",
    b"2000025000000250000002500000025000000250000002500000025000000257",
    b"2000025000000257200002500000025000000250000002572000025000000257",
    b"7000000000000002700000000000000000000000000000027000000000000002",
    b"6000000000000000000000000000000000000000000000000000000000000003",
    b"5000000000000000000000000000000000000000000000000000000000000004",
    b"4000003000000030000000300000003000000030000000300000003000000035",
    b"3000004000000040000000400000004000000040000000400000004000000046",
    b"2000025000000250000002500000025000000250000002500000025000000257",
    b"1200067002000670020006700200067002000670020006700200067002000677",
    b"7200567002005670020056700200567002005670020056700200567002005671",
    b"7000000000000000000000000000000000000000000000000000000000000002",
    b"6000000000000000000000000000000000000000000000000000000000000003",
    b"5000000000000000000000000000000000000000000000000000000000000004",
    b"4000003000000030000000300000003540000030000000300000003000000035",
    b"3000004000000040000000400000004630000040000000400000004000000046",
    b"2000025000000257200002500000025720000250000002572000025000000257",
    b"1234567712345677123456771234567712345677123456771234567712345677",
];

const FP16_ZERO: Fp16 = Fp16 { v: 0 };
const FP16_ONE: Fp16 = Fp16 { v: 1 << FP16_SCALE };
const FP16_TAU: Fp16 = Fp16 {
    v: (std::f32::consts::TAU * FP16_F) as i32,
};
const FP16_PI: Fp16 = Fp16 {
    v: (std::f32::consts::PI * FP16_F) as i32,
};

const FP16_FRAC_PI_2: Fp16 = Fp16 {
    v: (std::f32::consts::FRAC_PI_2 * FP16_F) as i32,
};

const FP16_PI_FRAC_PI_2: Fp16 = Fp16 {
    v: ((std::f32::consts::PI + std::f32::consts::FRAC_PI_2) * FP16_F) as i32,
};

const QUADRANT_1: std::ops::Range<Fp16> = FP16_ZERO..FP16_FRAC_PI_2;
const QUADRANT_2: std::ops::Range<Fp16> = FP16_FRAC_PI_2..FP16_PI;
const QUADRANT_3: std::ops::Range<Fp16> = FP16_PI..(FP16_PI_FRAC_PI_2);
const QUADRANT_4: std::ops::Range<Fp16> = (FP16_PI_FRAC_PI_2)..(FP16_TAU);

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
struct Fp16 {
    v: i32,
}

const FP16_SCALE: i32 = 16;
const FP16_F: f32 = (1 << FP16_SCALE) as f32;

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

// const TAN_CLAMP: f32 = 16.0;
const TAN_CLAMP: f32 = 128.0;

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

    pub fn sin(&self) -> Fp16 {
        ((self.v as f32) / FP16_F).sin().into()
    }

    pub fn cos(&self) -> Fp16 {
        ((self.v as f32) / FP16_F).cos().into()
    }
    pub fn tan(&self) -> Fp16 {
        ((self.v as f32) / FP16_F)
            .tan()
            .clamp(-TAN_CLAMP, TAN_CLAMP)
            .into()
    }
    pub fn cot(&self) -> Fp16 {
        (1.0 / ((self.v as f32) / FP16_F).tan())
            .clamp(-TAN_CLAMP, TAN_CLAMP)
            .into()
    }

    // pub fn as_f32(&self) -> f32 {
    //     self.v as f32 / FP16_F
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

lazy_static! {
    static ref COL_ANGLE: [Fp16; WIDTH] = {
        let mut col_angle = [Fp16::default(); WIDTH];
        for i in 0..WIDTH / 2 {
            let f = (i as f32) / (WIDTH as f32 * 0.5);
            col_angle[WIDTH / 2 + i] = f.atan().into();
            col_angle[WIDTH / 2 - i - 1] = (-f.atan()).into();
        }
        col_angle
    };
}

trait Draw {
    fn point(&mut self, x: i32, y: i32, c: i32);
    fn point_world(&mut self, x: Fp16, y: Fp16, c: i32);
    fn point_rgb(&mut self, x: i32, y: i32, c: u32);
}

impl Draw for Vec<u32> {
    fn point(&mut self, x: i32, y: i32, c: i32) {
        if x < 0
            || y < 0
            || (x as usize) >= WIDTH
            || (y as usize) >= HEIGHT
            || c < 0
            || (c as usize) > COLORS.len()
        {
            return;
        }

        self[(x as usize) + (y as usize) * WIDTH] = COLORS[c as usize];
    }

    fn point_rgb(&mut self, x: i32, y: i32, c: u32) {
        if x < 0 || y < 0 || (x as usize) >= WIDTH || (y as usize) >= HEIGHT {
            return;
        }

        self[(x as usize) + (y as usize) * WIDTH] = c;
    }

    fn point_world(&mut self, x: Fp16, y: Fp16, c: i32) {
        let scale = 4;
        let xp = (x * scale).get_int();
        let yp = (y * scale).get_int();
        // let xp = x.get_int();
        // let yp = y.get_int();

        // println!("point {:?} {:?} -> {} {}", x, y, xp, yp);

        self.point(xp, yp, c);
    }
}

#[derive(Debug)]
struct Player {
    x: Fp16,
    y: Fp16,
    rot: Fp16,
}

#[derive(Debug)]
struct PlayerVel {
    forward: i32,
    rot: i32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            x: 1.1.into(),
            y: 1.1.into(),
            rot: 0.0.into(),
        }
    }
}

impl Player {
    pub fn int_pos(&self) -> (i32, i32) {
        (self.x.get_int(), self.y.get_int())
    }
    pub fn frac_pos(&self) -> (Fp16, Fp16) {
        (self.x.fract(), self.y.fract())
    }

    pub fn apply_vel(&mut self, player_vel: &PlayerVel, dt: Fp16) {
        self.rot += dt * player_vel.rot;
        while self.rot < FP16_ZERO {
            self.rot += FP16_TAU;
        }

        while self.rot >= FP16_TAU {
            self.rot -= FP16_TAU;
        }
        println!("cos sin {:?} {:?}", self.rot.cos(), self.rot.sin());
        let dx = self.rot.cos() * player_vel.forward * dt;
        let dy = self.rot.sin() * player_vel.forward * dt;
        self.x += dx;
        self.y += dy;
    }

    pub(crate) fn draw(&self, buffer: &mut Vec<u32>) {
        buffer.point_world(self.x, self.y, 0);

        for angle in COL_ANGLE.chunks(10) {
            let angle = angle[0];
            let dx = (self.rot + angle).cos() * 2;
            let dy = (self.rot + angle).sin() * 2;
            buffer.point_world(self.x + dx, self.y + dy, 2);
        }
    }
}

struct Map {
    pub map: [[i32; MAP_SIZE]; MAP_SIZE],
}

impl Default for Map {
    fn default() -> Self {
        let mut map = [[0; MAP_SIZE]; MAP_SIZE];
        for (in_line, out_line) in MAP.iter().zip(map.iter_mut()) {
            for (c, out) in in_line.iter().zip(out_line.iter_mut()) {
                *out = (*c - b'0').into();
            }
        }

        Self { map }
    }
}

impl Map {
    fn lookup(&self, x: i32, y: i32) -> i32 {
        if x < 0 || y < 0 || (x as usize) >= MAP_SIZE || (y as usize) >= MAP_SIZE {
            return 1; // solid outer
        }
        self.map[y as usize][x as usize]
    }
    pub fn sweep_raycast(
        &self,
        screen: &mut Vec<u32>,
        player: &Player,
        columns: Range<usize>,
        resources: &Resources,
    ) {
        let (x, y) = player.int_pos();
        if self.lookup(x, y) != 0 {
            return;
        }
        let (dx, dy) = player.frac_pos();
        let (ex, ey) = (FP16_ONE - dx, FP16_ONE - dy);
        for column in columns {
            let alpha_full = player.rot + COL_ANGLE[column];
            let alpha_full = if alpha_full < FP16_ZERO {
                alpha_full + FP16_TAU
            } else if alpha_full >= FP16_TAU {
                alpha_full - FP16_TAU
            } else {
                alpha_full
            };

            let hstep_x;
            let hstep_y;
            let tx;
            let ty;

            let mut nx;
            let mut ny;

            if QUADRANT_1.contains(&alpha_full) {
                let alpha = alpha_full - QUADRANT_1.start;
                hstep_x = 1;
                hstep_y = 1;
                tx = alpha.cot();
                ty = alpha.tan();
                nx = player.x + (tx * ey);
                ny = player.y + (ty * ex);
            } else if QUADRANT_2.contains(&alpha_full) {
                let alpha = alpha_full - QUADRANT_2.start;
                hstep_x = -1;
                hstep_y = 1;
                tx = -alpha.tan();
                ty = alpha.cot();
                nx = player.x + (tx * ey);
                ny = player.y + (ty * dx); // - 1.into();
            } else if QUADRANT_3.contains(&alpha_full) {
                let alpha = alpha_full - QUADRANT_3.start;
                hstep_x = -1;
                hstep_y = -1;
                tx = -alpha.cot();
                ty = -alpha.tan();
                // let m1 = (tx * dy).as_f32();
                // let m2 = (-alpha.cot()).as_f32() * dy.as_f32();
                nx = player.x + (tx * dy);
                ny = player.y + (ty * dx);
                // let nxf = nx.as_f32();
                // let nyf = ny.as_f32();
            } else if QUADRANT_4.contains(&alpha_full) {
                let alpha = alpha_full - QUADRANT_4.start;
                hstep_x = 1;
                hstep_y = -1;
                tx = alpha.tan();
                ty = -alpha.cot();
                nx = player.x + (tx * dy);
                ny = player.y + (ty * ex);
            } else {
                continue;
            };

            // TODO: can't we just shift hx/nx (or hy/ny) both by -1 to get rid of lookup correction?
            // hx/hy is the x/y boundary of the next block we enter. Depending on the hstep direction this is either the left or right (upper or lower)
            // boundary.
            let mut hx = x + hstep_x.max(0);
            let mut hy = y + hstep_y.max(0);

            let mut hit_tile;
            // let mut hit_dist = 0.0;
            let dx;
            let dy;
            // let mut loop_count = 0;
            // 'outer: loop {
            //     while (hstep_y > 0 && ny <= hy.into()) || (hstep_y < 0 && ny >= hy.into()) {
            //         // when hstep_x is negative, hx needs to be corrected by -1 (enter block from right / below). Inverse of the correction during hx initialization.
            //         hit_tile = self.lookup(hx + hstep_x.min(0), ny.get_int());

            //         if hit_tile != 0 {
            //             screen.pointWorld(hx.into(), ny, hit_tile);
            //             dx = Fp16::from(hx) - player.x;
            //             dy = ny - player.y;
            //             break 'outer;
            //         }

            //         hx += hstep_x;
            //         ny += ty;
            //     }

            //     while (hstep_x > 0 && nx <= hx.into()) || (hstep_x < 0 && nx >= hx.into()) {
            //         // when hstep_y is negative, hy needs to be corrected by -1 (enter block from right / below). Inverse of the correction during hx initialization.
            //         hit_tile = self.lookup(nx.get_int(), hy + hstep_y.min(0));
            //         if hit_tile != 0 {
            //             hit_tile += 8;
            //             screen.pointWorld(nx, hy.into(), hit_tile);
            //             dx = nx - player.x;
            //             dy = Fp16::from(hy) - player.y;
            //             break 'outer;
            //         }
            //         hy += hstep_y;
            //         nx += tx;
            //     }
            //     loop_count += 1;
            //     if loop_count > 32 {
            //         println!("x: {} {}", nx.as_f32(), hx);
            //         println!("y: {} {}", ny.as_f32(), hy);
            //         panic!("{:?} col: {}", player, column);
            //     }
            // }
            let tex_u;
            'outer: loop {
                if (hstep_y > 0 && ny <= hy.into()) || (hstep_y < 0 && ny >= hy.into()) {
                    // when hstep_x is negative, hx needs to be corrected by -1 (enter block from right / below). Inverse of the correction during hx initialization.
                    hit_tile = self.lookup(hx + hstep_x.min(0), ny.get_int());

                    if hit_tile != 0 {
                        screen.point_world(hx.into(), ny, hit_tile);
                        dx = Fp16::from(hx) - player.x;
                        dy = ny - player.y;
                        tex_u = (ny.get_fract() >> 10) as i32;
                        break 'outer;
                    }

                    hx += hstep_x;
                    ny += ty;
                }
                // while (hstep_x > 0 && nx <= hx.into()) || (hstep_x < 0 && nx >= hx.into()) {
                else {
                    // when hstep_y is negative, hy needs to be corrected by -1 (enter block from right / below). Inverse of the correction during hx initialization.
                    hit_tile = self.lookup(nx.get_int(), hy + hstep_y.min(0));
                    if hit_tile != 0 {
                        // hit_tile += 8;
                        screen.point_world(nx, hy.into(), hit_tile);
                        dx = nx - player.x;
                        dy = Fp16::from(hy) - player.y;
                        tex_u = (nx.get_fract() >> 10) as i32;
                        break 'outer;
                    }
                    hy += hstep_y;
                    nx += tx;
                }
                // loop_count += 1;
                // if loop_count > 64 {
                //     println!("x: {} {}", nx.as_f32(), hx);
                //     println!("y: {} {}", ny.as_f32(), hy);
                //     panic!("{:?} col: {}", player, column);
                // }
            }
            const MID: i32 = 100;
            const C: i32 = 100;
            let beta = player.rot;
            let p = beta.cos() * dx + beta.sin() * dy;
            let offs = if p > FP16_ZERO {
                (C << FP16_SCALE) / p.v
            } else {
                C
            };

            let line_range = (MID - offs)..(MID + offs);

            if false {
                let step = 64.0 / line_range.len() as f32;
                let mut tex_v = (line_range.start as f32 - (HEIGHT / 2) as f32
                    + line_range.len() as f32 / 2.0)
                    * step;
                let tex_col = resources.get_texture(hit_tile)[(tex_u % 64) as usize];
                for row in line_range {
                    tex_v += step;
                    let tex_v = (tex_v - 0.5) as usize;
                    // let color = if ((tex_v >> 2) ^ (tex_u >> 2)) % 2 == 0 {
                    //     hit_tile + 8
                    // } else {
                    //     hit_tile
                    // };
                    // screen.point(column as i32, row, color);
                    screen.point_rgb(column as i32, row, tex_col[tex_v % 64]);
                }
            } else {
                let (tex_clip, line_range_clamped) = if line_range.start < 0 {
                    let h = 100 - line_range.start; // overall (half) height of drawn column (partially offscreen)
                    let x = (32 * 100) / h; // (half) number of texture pixels inside visible range
                    let tex_clip = 32 - x;
                    (tex_clip, 0..(HEIGHT as i32))
                } else {
                    (0, line_range)
                };
                // do not include 'endpoints' in bresenham. -1 on d_screen and d_tex to account for this.
                let d_screen = line_range_clamped.end - line_range_clamped.start - 1;
                let d_tex = 64 - 2 * tex_clip - 1;
                let mut d = 2 * d_tex - d_screen;
                let mut row_tex = 0;
                let tex_col = resources.get_texture(hit_tile)[(tex_u % 64) as usize];

                for row_screen in line_range_clamped {
                    if !true {
                        let tex_v = (row_tex + tex_clip) % 64;
                        let color = if ((tex_v >> 1) ^ (tex_u >> 1)) % 2 == 0 {
                            hit_tile + 8
                        } else {
                            hit_tile
                        };
                        screen.point(column as i32, row_screen, color);
                    } else {
                        screen.point_rgb(
                            column as i32,
                            row_screen,
                            tex_col[(row_tex + tex_clip) as usize],
                        );
                    }
                    while d > 0 {
                        row_tex += 1;
                        d -= 2 * d_screen;
                    }
                    d += 2 * d_tex
                }
            }
            // screen.point(column as i32, MID + offs, 0);
            // screen.point(column as i32, MID - offs, 0);
        }
    }
}

#[test]
fn raycast_test() {
    let map = Map::default();
    let resources = Resources::default();
    let mut screen = vec![0; WIDTH * HEIGHT];
    let player = Player {
        x: Fp16 { v: 72090 },
        y: Fp16 { v: 72090 },
        rot: Fp16 { v: 0 },
    };
    let col = 10;
    map.sweep_raycast(&mut screen, &player, col..(col + 1), &resources);
}

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let resources = Resources::load_textures("textures.txt");

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions {
            scale: minifb::Scale::X4,
            ..Default::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let dt: Fp16 = (1.0f32 / 60.0f32).into();
    let fwd_speed: i32 = 10;
    let rot_speed: i32 = 5;
    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut player_vel = PlayerVel { forward: 0, rot: 0 };
    let mut player = Player::default();
    let map = Map::default();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for i in buffer.iter_mut() {
            *i = 0x40404040; // write something more funny here!
        }

        for i in 0..16 {
            buffer.point(10 + i, 10 + i, i);
        }

        player_vel.forward = 0;
        player_vel.rot = 0;
        if window.is_key_down(Key::W) {
            player_vel.forward += fwd_speed;
        }
        if window.is_key_down(Key::S) {
            player_vel.forward -= fwd_speed;
        }
        if window.is_key_down(Key::D) {
            player_vel.rot += rot_speed;
        }
        if window.is_key_down(Key::A) {
            player_vel.rot -= rot_speed;
        }

        player.apply_vel(&player_vel, dt);
        // println!("player: {:?} {:?} {:?}", player_vel, player.x, player.y);
        println!("player: {:?}", player);

        player.draw(&mut buffer);

        let start = Instant::now();
        // for _ in 0..1000 {
        map.sweep_raycast(&mut buffer, &player, 0..WIDTH, &resources);
        // }
        println!("time: {}us", start.elapsed().as_micros());

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
