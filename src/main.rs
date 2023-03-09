use std::{
    ops::{Add, AddAssign, Div, Mul, Neg, Range, Sub, SubAssign},
    time::Instant,
};

use lazy_static::lazy_static;
use minifb::{Key, Window, WindowOptions};
use raycast::{wl6, Resources};

const WIDTH: usize = 320;
const HEIGHT: usize = 200;
const COLORS: [u32; 16] = [
    0xFFFFFF, 0xFF0000, 0x00FF00, 0x0000FF, 0xFFFF00, 0x00FFFF, 0xFF00FF, 0xFF8000, 0x808080,
    0x800000, 0x008000, 0x000080, 0x808000, 0x008080, 0x800080, 0x804000,
];

const MAP_SIZE: usize = 64;

const MAP: [&[u8]; MAP_SIZE] = include!("maps/map2.txt");
// const MAP: [&[u8]; MAP_SIZE] = include!("maps/map_empty.txt");

const FP16_ZERO: Fp16 = Fp16 { v: 0 };
const FP16_ONE: Fp16 = Fp16 { v: 1 << FP16_SCALE };
const FP16_HALF: Fp16 = Fp16 {
    v: (1 << FP16_SCALE) / 2,
};

const FA_SCALEF: f32 = 10.0;

const PIS_IN_180: f32 = 57.295_78_f32; // .to_degrees() method is not constexpr
const FA_TAU: i32 = (std::f32::consts::TAU * PIS_IN_180 * FA_SCALEF) as i32;
const FA_PI: i32 = (std::f32::consts::PI * PIS_IN_180 * FA_SCALEF) as i32;

const FA_FRAC_PI_2: i32 = (std::f32::consts::FRAC_PI_2 * PIS_IN_180 * FA_SCALEF) as i32;

const FA_PI_FRAC_PI_2: i32 =
    ((std::f32::consts::PI + std::f32::consts::FRAC_PI_2) * PIS_IN_180 * FA_SCALEF) as i32;

const QUADRANT_1: std::ops::Range<i32> = 0..FA_FRAC_PI_2;
const QUADRANT_2: std::ops::Range<i32> = FA_FRAC_PI_2..FA_PI;
const QUADRANT_3: std::ops::Range<i32> = FA_PI..FA_PI_FRAC_PI_2;
const QUADRANT_4: std::ops::Range<i32> = (FA_PI_FRAC_PI_2)..(FA_TAU);

// const TAN_CLAMP: f32 = 16.0;
const TAN_CLAMP: f32 = 128.0;
const FA_STEPS: usize = 3600;

lazy_static! {
    static ref COL_ANGLE: [i32; WIDTH] = {
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

fn fa_sin(v: i32) -> Fp16 {
    SIN_TABLE[v as usize]
}

fn fa_cos(v: i32) -> Fp16 {
    COS_TABLE[v as usize]
}

fn fa_tan(v: i32) -> Fp16 {
    TAN_TABLE[v as usize]
}

fn fa_cot(v: i32) -> Fp16 {
    COT_TABLE[v as usize]
}

fn fa_fix_angle(mut v: i32) -> i32 {
    while v < 0 {
        v += FA_TAU;
    }

    while v >= FA_TAU {
        v -= FA_TAU;
    }
    v
}

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
    pub fn as_f32(&self) -> f32 {
        (self.v as f32) / FP16_F
    }
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
        // if x < 0 || y < 0 || (x as usize) >= WIDTH || (y as usize) >= HEIGHT {
        //     return;
        // }
        if c & 0xff000000 == 0 {
            return;
        }
        self[(x as usize) + (y as usize) * WIDTH] = c;
    }

    fn point_world(&mut self, x: Fp16, y: Fp16, c: i32) {
        let scale = 3;
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
    rot: i32,
}

#[derive(Debug)]
struct PlayerVel {
    forward: i32,
    right: i32,
    rot: i32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            x: 3.1.into(),
            y: 3.1.into(),
            rot: 0,
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

    pub fn apply_vel(&mut self, player_vel: &PlayerVel, dt: Fp16, map: &Map) {
        self.rot += (dt * player_vel.rot).get_int();
        while self.rot < 0 {
            self.rot += FA_TAU;
        }

        while self.rot >= FA_TAU {
            self.rot -= FA_TAU;
        }
        // println!("cos sin {:?} {:?}", fa_cos(self.rot), fa_sin(self.rot));
        let sin = fa_sin(self.rot);
        let cos = fa_cos(self.rot);
        // right direction is forward + 90deg
        let dx = (cos * player_vel.forward + sin * player_vel.right) * dt;
        let dy = (sin * player_vel.forward - cos * player_vel.right) * dt;

        let player_width: Fp16 = 0.4.into();

        let tx = [
            self.x + player_width,
            self.x - player_width,
            self.x - player_width,
            self.x + player_width,
        ];
        let ty = [
            self.y + player_width,
            self.y + player_width,
            self.y - player_width,
            self.y - player_width,
        ];

        // select the three corners of the player box that need to be checked (depends on the quadrant of the actual movement)
        let tis = if dx >= FP16_ZERO && dy >= FP16_ZERO {
            [0, 1, 3]
        } else if dx < FP16_ZERO && dy >= FP16_ZERO {
            [1, 2, 0]
        } else if dx < FP16_ZERO && dy < FP16_ZERO {
            [1, 2, 3]
        } else {
            [0, 2, 3]
        };

        let mut can_move_x = true;
        let mut can_move_y = true;
        for ti in tis {
            let x = tx[ti] + dx;
            let y = ty[ti] + dy;

            if !map.can_walk(x.get_int(), y.get_int()) {
                println!("collision {}", ti + 1);
                can_move_x &= map.can_walk(x.get_int(), ty[ti].get_int());
                can_move_y &= map.can_walk(tx[ti].get_int(), y.get_int());
            }
        }
        if can_move_x {
            self.x += dx;
        }
        if can_move_y {
            self.y += dy;
        }
    }

    pub(crate) fn draw(&self, buffer: &mut Vec<u32>) {
        buffer.point_world(self.x, self.y, 0);

        for angle in COL_ANGLE.chunks(10) {
            let angle = angle[0];
            let dx = fa_cos(fa_fix_angle(self.rot + angle)) * 2;
            let dy = fa_sin(fa_fix_angle(self.rot + angle)) * 2;
            buffer.point_world(self.x + dx, self.y + dy, 2);
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum MapTile {
    Wall(i32),
    Walkable(i32),
}

struct Map {
    pub map: [[MapTile; MAP_SIZE]; MAP_SIZE],
}

impl Default for Map {
    fn default() -> Self {
        let mut map = [[MapTile::Walkable(0); MAP_SIZE]; MAP_SIZE];
        for (in_line, out_line) in MAP.iter().zip(map.iter_mut()) {
            for (c, out) in in_line.iter().zip(out_line.iter_mut()) {
                // FIXME: crappy harcoded
                if *c >= b'1' && *c <= b'9' {
                    let tile: i32 = (*c - b'0').into();
                    *out = MapTile::Wall(tile - 1 + 10);
                } else if *c == b'8' || *c == b':' {
                    // *out = (0, false);
                }
            }
        }

        Self { map }
    }
}

impl Map {
    fn from_map_plane(plane: &[u16]) -> Self {
        assert_eq!(plane.len(), 64 * 64);
        let mut plane_iter = plane.iter();
        let mut map = [[MapTile::Walkable(0); MAP_SIZE]; MAP_SIZE];
        for line in &mut map {
            for out in line.iter_mut() {
                let c = (*plane_iter.next().unwrap() - 1) * 2;
                if (0..64).contains(&c) {
                    *out = MapTile::Wall(c as i32);
                }
            }
        }

        Self { map }
    }

    fn lookup_wall(&self, x: i32, y: i32) -> Option<i32> {
        if x < 0 || y < 0 || (x as usize) >= MAP_SIZE || (y as usize) >= MAP_SIZE {
            return Some(1); // solid outer
        }
        match self.map[y as usize][x as usize] {
            MapTile::Wall(id) => Some(id),
            MapTile::Walkable(_) => None,
        }
    }
    fn can_walk(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || (x as usize) >= MAP_SIZE || (y as usize) >= MAP_SIZE {
            return false; // solid outer
        }
        matches!(self.map[y as usize][x as usize], MapTile::Walkable(_))
    }

    pub fn sweep_raycast(
        &self,
        screen: &mut Vec<u32>,
        zbuffer: &mut [Fp16; WIDTH],
        player: &Player,
        columns: Range<usize>,
        resources: &Resources,
    ) {
        let (x, y) = player.int_pos();
        if !self.can_walk(x, y) {
            return;
        }
        let (dx, dy) = player.frac_pos();
        let (ex, ey) = (FP16_ONE - dx, FP16_ONE - dy);
        for column in columns {
            let alpha_full = player.rot + COL_ANGLE[column];
            let alpha_full = if alpha_full < 0 {
                alpha_full + FA_TAU
            } else if alpha_full >= FA_TAU {
                alpha_full - FA_TAU
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
                tx = fa_cot(alpha);
                ty = fa_tan(alpha);
                nx = player.x + (tx * ey);
                ny = player.y + (ty * ex);
            } else if QUADRANT_2.contains(&alpha_full) {
                let alpha = alpha_full - QUADRANT_2.start;
                hstep_x = -1;
                hstep_y = 1;
                tx = -fa_tan(alpha);
                ty = fa_cot(alpha);
                nx = player.x + (tx * ey);
                ny = player.y + (ty * dx); // - 1.into();
            } else if QUADRANT_3.contains(&alpha_full) {
                let alpha = alpha_full - QUADRANT_3.start;
                hstep_x = -1;
                hstep_y = -1;
                tx = -fa_cot(alpha);
                ty = -fa_tan(alpha);
                nx = player.x + (tx * dy);
                ny = player.y + (ty * dx);
            } else if QUADRANT_4.contains(&alpha_full) {
                let alpha = alpha_full - QUADRANT_4.start;
                hstep_x = 1;
                hstep_y = -1;
                tx = fa_tan(alpha);
                ty = -fa_cot(alpha);
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
            let tex_u;
            'outer: loop {
                if (hstep_y > 0 && ny <= hy.into()) || (hstep_y < 0 && ny >= hy.into()) {
                    // when hstep_x is negative, hx needs to be corrected by -1 (enter block from right / below). Inverse of the correction during hx initialization.
                    let lookup_tile = self.lookup_wall(hx + hstep_x.min(0), ny.get_int());

                    if let Some(tile) = lookup_tile {
                        screen.point_world(hx.into(), ny, (tile) % 16);
                        hit_tile = tile + 1;
                        dx = Fp16::from(hx) - player.x;
                        dy = ny - player.y;
                        tex_u = if hstep_x > 0 {
                            (ny.get_fract() >> 10) as i32
                        } else {
                            63 - (ny.get_fract() >> 10) as i32
                        };
                        break 'outer;
                    }

                    hx += hstep_x;
                    ny += ty;
                } else {
                    // when hstep_y is negative, hy needs to be corrected by -1 (enter block from right / below). Inverse of the correction during hx initialization.
                    let lookup_tile = self.lookup_wall(nx.get_int(), hy + hstep_y.min(0));
                    if let Some(tile) = lookup_tile {
                        // hit_tile += 8;
                        hit_tile = tile;
                        screen.point_world(nx, hy.into(), (tile) % 16);
                        dx = nx - player.x;
                        dy = Fp16::from(hy) - player.y;
                        tex_u = if hstep_y < 0 {
                            (nx.get_fract() >> 10) as i32
                        } else {
                            63 - (nx.get_fract() >> 10) as i32
                        };

                        break 'outer;
                    }
                    hy += hstep_y;
                    nx += tx;
                }
            }
            const MID: i32 = (HEIGHT / 2) as i32;
            const C: i32 = MID;
            let beta = player.rot;
            let p = fa_cos(beta) * dx + fa_sin(beta) * dy;
            let offs = if p > FP16_ZERO {
                (C << FP16_SCALE) / p.v
            } else {
                C
            };
            zbuffer[column] = p;
            let line_range = (MID - offs)..(MID + offs);

            // TODO: pre-compute LUTs for the non clipped cases?
            const TEXEL_SCALE: i32 = 1; // NOTE: this currently influences the performance of the bresenham loop
            let (tex_clip, line_range_clamped) = if line_range.start < 0 {
                const HALF_HEIGHT: i32 = (HEIGHT / 2) as i32;
                const HALF_TEX: i32 = 32 << TEXEL_SCALE;
                let h = HALF_HEIGHT - line_range.start; // overall (half) height of drawn column (partially offscreen)
                let x = (HALF_TEX * HALF_HEIGHT) / h; // (half) number of texture pixels inside visible range (derived via HALF_HEIGHT / h == x / HALF_TEX)
                let tex_clip = HALF_TEX - x;
                (tex_clip, 0..(HEIGHT as i32))
            } else {
                (0, line_range)
            };
            // do not include 'endpoints' in bresenham. -1 on d_screen and d_tex to account for this.
            let d_screen = line_range_clamped.end - line_range_clamped.start - 1;
            let d_tex = (64 << TEXEL_SCALE) - 2 * tex_clip - 1;
            let mut d = 2 * d_tex - d_screen;
            let mut row_tex = 0;
            let tex_col = &resources.get_texture(hit_tile)[(tex_u) as usize];

            for row_screen in line_range_clamped {
                if !true {
                    let tex_v = (row_tex + tex_clip) >> TEXEL_SCALE;
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
                        tex_col[(row_tex + tex_clip) as usize >> TEXEL_SCALE],
                    );
                }
                while d > 0 {
                    row_tex += 1;
                    d -= 2 * d_screen;
                }
                d += 2 * d_tex
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
    let mut zbuffer = [Fp16::default(); WIDTH];
    let player = Player {
        x: Fp16 { v: 72090 },
        y: Fp16 { v: 72090 },
        rot: 0,
    };
    let col = 10;
    map.sweep_raycast(
        &mut screen,
        &mut zbuffer,
        &player,
        col..(col + 1),
        &resources,
    );
}

fn draw_sprite(
    screen: &mut Vec<u32>,
    zbuffer: &[Fp16],
    resources: &Resources,
    id: i32,
    x_mid: i32,
    z: Fp16,
) {
    // FIXME: what is going on with the sprite scaling? at least pillars seem to be meant larger than walls at the same z distance
    const C: i32 = MID;
    let offs = if z > FP16_ZERO {
        (C << FP16_SCALE) / z.v
    } else {
        C
    };
    let line_range = (MID - offs)..(MID + offs);

    let tex = resources.get_sprite(id);

    const MID: i32 = HEIGHT as i32 / 2;
    let target_size = 2 * offs;
    // TODO: re-think the whole thing... it kind of works without explicit clipping to screen bounds, but the redundant in-loop bounds checks are weird and inefficient

    let texcoord = {
        // pre-calc screen -> tex coords
        let mut texcoord = [0u8; WIDTH];

        let d_screen = (line_range.end - line_range.start) - 1;
        let d_tex = 64 - 1;
        let mut d = 2 * d_tex - d_screen;
        let mut tex = 0;
        // only fill first (2 * offs, screen-space target size) entries of the texcoord array, using bresenham-style interpolator
        for tx in texcoord.iter_mut().take(target_size as usize) {
            *tx = tex;

            while d > 0 {
                tex += 1;
                d -= 2 * d_screen;
            }
            d += 2 * d_tex;
        }
        texcoord
    };
    for x in 0..(target_size.min(WIDTH as i32)) {
        let column = x + (x_mid - offs);
        // TODO: clip to screen bounds
        if column < 0 || column >= WIDTH as i32 || zbuffer[column as usize] <= z {
            continue;
        }
        let tex_col = tex[texcoord[x as usize] as usize];
        for y in 0..(target_size.min(WIDTH as i32)) {
            let row = y + (MID - offs);
            if row < 0 || row >= HEIGHT as i32 {
                continue;
            }
            if row >= 0 && row < HEIGHT as i32 {
                let c = tex_col[texcoord[y as usize] as usize];
                if c != 0 {
                    screen.point_rgb(column, row, c);
                }
            }
        }
    }
}

struct Sprite {
    x: Fp16,
    y: Fp16,
    id: i32,
    directional: bool,
}

struct Sprites {
    sprites: Vec<Sprite>,
    screen_pos: Vec<(Fp16, i32, i32)>,
}

impl Default for Sprites {
    fn default() -> Self {
        let sprites = MAP
            .iter()
            .enumerate()
            .flat_map(|(y, row)| {
                row.iter().enumerate().filter_map(move |(x, c)| {
                    if *c == b'9' {
                        Some(Sprite {
                            x: FP16_HALF + (x as i32).into(),
                            y: FP16_HALF + (y as i32).into(),
                            id: 51, //(*c - b'0') as i32,
                            directional: true,
                        })
                    } else {
                        None
                    }
                })
            })
            .collect();

        Self {
            sprites,
            screen_pos: Default::default(),
        }
    }
}

impl Sprites {
    pub fn from_map_plane(plane: &[u16]) -> Sprites {
        assert_eq!(plane.len(), 64 * 64);
        let mut plane_iter = plane.iter();
        let props_range = 23..=70;
        let props_sprite_offset = 2;
        let mut sprites = Vec::new();
        for y in 0..64 {
            for x in 0..64 {
                let c = (*plane_iter.next().unwrap()) + 1;

                if props_range.contains(&c) {
                    sprites.push(Sprite {
                        x: FP16_HALF + x.into(),
                        y: FP16_HALF + y.into(),
                        id: (c - 21) as i32,
                        directional: false,
                    })
                }
            }
        }
        Sprites {
            sprites,
            ..Default::default()
        }
    }

    pub fn setup_screen_pos_for_player(&mut self, player: &Player) {
        self.screen_pos.clear();
        self.screen_pos.reserve(self.sprites.len());
        let inv_sin = fa_sin(fa_fix_angle(-player.rot));
        let inv_cos = fa_cos(fa_fix_angle(-player.rot));

        for sprite in &self.sprites {
            let x = sprite.x - player.x;
            let y = sprite.y - player.y;

            let tx = x * inv_cos - y * inv_sin;
            let ty = y * inv_cos + x * inv_sin;

            if tx <= FP16_HALF
            /*|| tx.get_int() < 1 */
            {
                continue;
            }

            let z = tx;
            // let screen_x = (WIDTH as i32 / 2) + (ty * (WIDTH as i32 / 2)).get_int();
            let screen_x = ((FP16_ONE + (ty / z)) * (WIDTH as i32 / 2)).get_int();

            let id = if sprite.directional {
                // calculate sprite orientation from player view angle
                // 1. rotate sprite by inverse of camera rotation
                // 2. turn by 'half an octant' to align steps to expected position
                let mut viewangle = -player.rot + (FA_STEPS as i32) / 16;
                // 3. approximate angle offset according to screen x position (who needs trigonometry whan you can fake it...)
                // 4. turn by 180 deg to look along x axis
                let cor = (160 - screen_x) * 2;
                viewangle += cor + 1800;

                while viewangle < 0 {
                    viewangle += FA_STEPS as i32;
                }
                while viewangle >= FA_STEPS as i32 {
                    viewangle -= FA_STEPS as i32;
                }
                viewangle /= FA_STEPS as i32 / 8;
                sprite.id + viewangle
            } else {
                sprite.id
            };
            // println!("viewangle: {viewangle}");
            self.screen_pos.push((z, screen_x, id));
        }
        self.screen_pos.sort_by_key(|(z, _, _)| -(*z));
    }
    pub fn draw(&self, screen: &mut Vec<u32>, zbuffer: &[Fp16], resources: &Resources, frame: i32) {
        for (z, mid, id) in &self.screen_pos {
            draw_sprite(
                screen, zbuffer, resources, *id, /*+ ((frame / 30) % 8)*/
                *mid, *z,
            );
        }
    }
}

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut zbuffer = [Fp16::default(); WIDTH];

    // let resources = Resources::load_textures("textures.txt");
    let resources = Resources::load_wl6("vswap.wl6");
    let mut maps = wl6::MapsFile::open("maphead.wl6", "gamemaps.wl6");

    let (plane0, plane1) = maps.get_map_planes(0);

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

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut player_vel = PlayerVel {
        forward: 0,
        right: 0,
        rot: 0,
    };
    let mut player = Player {
        x: Fp16 { v: 1933113 },
        y: Fp16 { v: 3793254 },
        rot: 3476,
    };
    // let mut player = Player::default();

    let mut sprites = Sprites::from_map_plane(&plane1);
    // let mut sprites = Sprites::default();

    let map = Map::from_map_plane(&plane0);
    // let map = Map::default();

    let mut frame = 0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for (i, chunk) in buffer.chunks_mut(320 * 100).enumerate() {
            if i == 0 {
                chunk.fill(0x38383838);
            } else {
                chunk.fill(0x64646464);
            }
        }

        // for i in buffer.iter_mut() {
        //     // *i = 0x40404040; // write something more funny here!

        //     *i = 0x84848484; // write something more funny here!
        // }

        for i in 0..16 {
            buffer.point(10 + i, 10 + i, i);
        }

        player_vel.forward = 0;
        player_vel.right = 0;
        player_vel.rot = 0;

        let (fwd_speed, rot_speed) = if window.is_key_down(Key::LeftShift) {
            (2, 360)
        } else {
            (7, 3 * 360)
        };

        if window.is_key_down(Key::W) {
            player_vel.forward += fwd_speed;
        }
        if window.is_key_down(Key::S) {
            player_vel.forward -= fwd_speed;
        }

        if window.is_key_down(Key::Q) {
            player_vel.right += fwd_speed;
        }
        if window.is_key_down(Key::E) {
            player_vel.right -= fwd_speed;
        }
        if window.is_key_down(Key::D) {
            player_vel.rot += rot_speed;
        }
        if window.is_key_down(Key::A) {
            player_vel.rot -= rot_speed;
        }
        if window.is_key_released(Key::F1) {
            player = Player::default();
        }

        player.apply_vel(&player_vel, dt, &map);
        // println!("player: {:?} {:?} {:?}", player_vel, player.x, player.y);
        println!("player: {:?}", player);

        player.draw(&mut buffer);
        let start = Instant::now();

        // for _ in 0..1000 {
        map.sweep_raycast(&mut buffer, &mut zbuffer, &player, 0..WIDTH, &resources);

        let sprite_start = Instant::now();

        // draw_sprite(&mut buffer, &zbuffer, &resources, 8, 100, sprite_z.into());
        // if frame % 4 == 0 {
        sprites.setup_screen_pos_for_player(&player);
        // }
        sprites.draw(&mut buffer, &zbuffer, &resources, frame);

        // }
        // println!(
        //     "time: {}us\t({}us sprite)",
        //     start.elapsed().as_micros(),
        //     sprite_start.elapsed().as_micros()
        // );

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
        frame += 1;
    }
}
