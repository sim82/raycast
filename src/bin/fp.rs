use std::{
    f32::consts::{FRAC_PI_2, PI},
    ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign},
    time::Instant,
};

use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 320;
const HEIGHT: usize = 200;
const COLORS: [u32; 16] = [
    0xFFFFFF, 0xFF0000, 0x00FF00, 0x0000FF, 0xFFFF00, 0x00FFFF, 0xFF00FF, 0xFF8000, 0x808080,
    0x800000, 0x008000, 0x000080, 0x808000, 0x008080, 0x800080, 0x804000,
];

const MAP: [[i32; 8]; 8] = [
    [7, 2, 3, 4, 5, 6, 7, 1],
    [7, 0, 0, 0, 0, 0, 0, 2],
    [6, 0, 0, 0, 0, 0, 0, 3],
    [5, 0, 0, 0, 0, 0, 0, 4],
    [4, 0, 0, 0, 0, 0, 3, 5],
    [3, 0, 0, 0, 0, 0, 4, 6],
    [2, 0, 0, 0, 0, 2, 5, 7],
    [1, 2, 3, 4, 5, 6, 7, 7],
];

const FP16_ZERO: Fp16 = Fp16 { v: 0 };
const FP16_ONE: Fp16 = Fp16 { v: 1 << 16 };
const FP16_TAU: Fp16 = Fp16 {
    v: (std::f32::consts::TAU * FP16_F) as i32,
};
const FP16_PI: Fp16 = Fp16 {
    v: (std::f32::consts::PI * FP16_F) as i32,
};

const FP16_FRAC_PI_2: Fp16 = Fp16 {
    v: (std::f32::consts::FRAC_PI_2 * FP16_F) as i32,
};

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
struct Fp16 {
    v: i32,
}

const FP16_F: f32 = 256.0 * 256.0;

impl From<f32> for Fp16 {
    fn from(f: f32) -> Self {
        Self {
            v: (f * FP16_F) as i32,
        }
    }
}

impl From<i32> for Fp16 {
    fn from(f: i32) -> Self {
        Self { v: f << 16 }
    }
}

impl Fp16 {
    pub fn get_int(&self) -> i32 {
        self.v >> 16
    }
    pub fn get_fract(&self) -> u32 {
        (self.v as u32) & 0xFFFF
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
        ((self.v as f32) / FP16_F).tan().clamp(-1.0, 1.0).into()
    }
    pub fn cot(&self) -> Fp16 {
        (1.0 / ((self.v as f32) / FP16_F).tan())
            .clamp(-1.0, 1.0)
            .into()
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
        let v = (((self.v as i64) * (rhs.v as i64)) >> 16) as i32;
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
    fn pointWorld(&mut self, x: Fp16, y: Fp16, c: i32);
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

    fn pointWorld(&mut self, x: Fp16, y: Fp16, c: i32) {
        let scale = 16;
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
    col_angle: [Fp16; WIDTH],
}

#[derive(Debug)]
struct PlayerVel {
    forward: i32,
    rot: i32,
}

impl Default for Player {
    fn default() -> Self {
        let mut col_angle = [Fp16::default(); WIDTH];
        for i in 0..WIDTH / 2 {
            let f = (i as f32) / (WIDTH as f32 * 0.5);
            col_angle[WIDTH / 2 + i] = f.atan().into();
            col_angle[WIDTH / 2 - i - 1] = (-f.atan()).into();
        }

        Self {
            x: 1.1.into(),
            y: 1.1.into(),
            rot: 0.0.into(),
            col_angle,
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
        buffer.pointWorld(self.x, self.y, 0);

        for angle in self.col_angle.chunks(10) {
            let angle = angle[0];
            let dx = (self.rot + angle).cos() * 2;
            let dy = (self.rot + angle).sin() * 2;
            buffer.pointWorld(self.x + dx, self.y + dy, 2);
        }
    }
}

const MAP_SIZE: usize = 8;
struct Map {
    pub map: [[i32; MAP_SIZE]; MAP_SIZE],
}

impl Default for Map {
    fn default() -> Self {
        Self { map: MAP }
    }
}

impl Map {
    fn lookup(&self, x: i32, y: i32) -> i32 {
        if x < 0 || y < 0 || (x as usize) >= MAP_SIZE || (y as usize) >= MAP_SIZE {
            return 1; // solid outer
        }
        self.map[y as usize][x as usize]
    }
    pub fn sweep_raycast(&self, screen: &mut Vec<u32>, player: &Player) {
        let (x, y) = player.int_pos();
        if self.lookup(x, y) != 0 {
            return;
        }
        let (dx, dy) = player.frac_pos();
        let (ex, ey) = (FP16_ONE - dx, FP16_ONE - dy);
        for column in 0..WIDTH {
            let alpha_full = player.rot + player.col_angle[column];
            let alpha_full = if alpha_full < FP16_ZERO {
                alpha_full + FP16_TAU
            } else if alpha_full >= FP16_TAU {
                alpha_full - FP16_TAU
            } else {
                alpha_full
            };

            let alpha;
            let hstep_x;
            let hstep_y;
            let tx;
            let ty;
            let sx;
            let sy;
            let mut hx;
            let mut hy;
            let quad_alpha;
            let mut xcor = 0.0;
            let mut ycor = 0.0;
            let (comp_x, comp_y): (Box<dyn Fn(i32, i32) -> bool>, Box<dyn Fn(i32, i32) -> bool>) =
                if (FP16_ZERO..FP16_FRAC_PI_2).contains(&alpha_full) {
                    quad_alpha = FP16_ZERO;
                    alpha = alpha_full - quad_alpha;
                    hstep_x = 1;
                    hstep_y = 1;
                    tx = alpha.cot();
                    ty = alpha.tan();
                    sx = tx * ey;
                    sy = ty * ex;
                    hx = x + hstep_x;
                    hy = y + hstep_y;
                    (
                        Box::new(|a: i32, b: i32| a < b),
                        Box::new(|a: i32, b: i32| a < b),
                    )
                }
                //  else if (FP16_FRAC_PI_2..FP16_PI).contains(&alpha_full) {
                //     quad_alpha = FP16_FRAC_PI_2;
                //     alpha = alpha_full - quad_alpha;
                //     hstep_x = -1;
                //     hstep_y = 1;
                //     tx = -alpha.tan();
                //     ty = alpha.cot();
                //     sx = tx * ey;
                //     sy = ty * dx;
                //     hx = x;
                //     hy = y + hstep_y;
                //     xcor = -0.01;

                //     (
                //         Box::new(|a: i32, b: i32| a > b),
                //         Box::new(|a: i32, b: i32| a < b),
                //     )
                // } else if (FP16_PI..(FP16_PI + FP16_FRAC_PI_2)).contains(&alpha_full) {
                //     quad_alpha = FP16_PI;
                //     alpha = alpha_full - quad_alpha;
                //     hstep_x = 1;
                //     hstep_y = 1;
                //     tx = -alpha.cot();
                //     ty = -alpha.tan();
                //     sx = tx * dy;
                //     sy = ty * dx;
                //     hx = x;
                //     hy = y;
                //     xcor = -0.01;
                //     ycor = -0.01;
                //     (
                //         Box::new(|a: i32, b: i32| a > b),
                //         Box::new(|a: i32, b: i32| a > b),
                //     )
                // } else if ((FP16_PI + FP16_FRAC_PI_2)..(FP16_TAU)).contains(&alpha_full) {
                //     quad_alpha = FP16_PI + FP16_FRAC_PI_2;
                //     alpha = alpha_full - quad_alpha;
                //     hstep_x = 1;
                //     hstep_y = -1;
                //     tx = alpha.tan();
                //     ty = -alpha.cot();
                //     sx = tx * dy;
                //     sy = ty * ex;
                //     hx = x + hstep_x;
                //     hy = y;
                //     ycor = -0.01;
                //     (
                //         Box::new(|a: i32, b: i32| a < b),
                //         Box::new(|a: i32, b: i32| a > b),
                //     )
                // }
                else {
                    continue;
                };

            let mut nx = player.x + sx;
            let mut ny = player.y + sy;
            let mut hit_tile;
            // let mut hit_dist = 0.0;
            let dx;
            let dy;
            'outer: loop {
                // 'outer: for _ in 0..32 {
                const E: f32 = 1e-3;
                // let equal = (hy - ny).abs() < E && (hx - nx).abs() < E;
                while comp_y(ny.get_int(), hy) {
                    hit_tile = self.lookup(hx, ny.get_int());

                    if hit_tile != 0 {
                        screen.pointWorld(hx.into(), ny, hit_tile);
                        // hit_dist = ((hx - player.x).powf(2.0) + (ny - player.y).powf(2.0)).sqrt();
                        dx = Fp16::from(hx) - player.x;
                        dy = ny - player.y;
                        break 'outer;
                    }

                    hx += hstep_x;
                    ny += ty;
                }

                while comp_x(nx.get_int(), hx) {
                    hit_tile = self.lookup(nx.get_int(), hy);
                    if hit_tile != 0 {
                        hit_tile += 8;
                        screen.pointWorld(nx, hy.into(), hit_tile);
                        //hit_dist = ((nx - player.x).powf(2.0) + (hy - player.y).powf(2.0)).sqrt();
                        dx = nx - player.x;
                        dy = Fp16::from(hy) - player.y;
                        break 'outer;
                    }
                    hy += hstep_y;
                    nx += tx;
                }
            }
            let mid = 100;
            let c = 100;
            let beta = player.rot;
            let p = beta.cos() * dx + beta.sin() * dy;
            // let p = 2.0 / p;
            // let p = (dx * dx + dy * dy).sqrt();
            // let p = p * (beta - alpha).cos();
            // screen.pointWorld(
            //     player.x + p * alpha.cos(),
            //     player.y + p * alpha.sin(),
            //     hit_tile,
            // );
            let offs = if p > FP16_ZERO { c / p.get_int() } else { c };

            for row in (mid - offs as i32)..(mid + offs as i32) {
                screen.point(column as i32, row, hit_tile);
            }
            screen.point(column as i32, mid + offs as i32, 0);
            screen.point(column as i32, mid - offs as i32, 0);
        }
    }
}
fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

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
            *i = 0; // write something more funny here!
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
        println!("player: {:?} {:?} {:?}", player_vel, player.x, player.y);

        player.draw(&mut buffer);

        // let start = Instant::now();
        map.sweep_raycast(&mut buffer, &player);
        // println!("time: {}ns", start.elapsed().as_nanos());

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
