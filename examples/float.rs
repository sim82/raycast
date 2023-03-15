use std::{
    f32::consts::{FRAC_PI_2, PI},
    time::Instant,
};

use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 320;
const HEIGHT: usize = 200;
const COLORS: [u32; 16] = [
    0xFFFFFF, 0xFF0000, 0x00FF00, 0x0000FF, 0xFFFF00, 0x00FFFF, 0xFF00FF, 0xFF8000, 0x808080, 0x800000, 0x008000,
    0x000080, 0x808000, 0x008080, 0x800080, 0x804000,
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

trait Draw {
    fn point(&mut self, x: i32, y: i32, c: i32);
    fn pointWorld(&mut self, x: f32, y: f32, c: i32);
}

impl Draw for Vec<u32> {
    fn point(&mut self, x: i32, y: i32, c: i32) {
        if x < 0 || y < 0 || (x as usize) >= WIDTH || (y as usize) >= HEIGHT || c < 0 || (c as usize) > COLORS.len() {
            return;
        }

        self[(x as usize) + (y as usize) * WIDTH] = COLORS[c as usize];
    }

    fn pointWorld(&mut self, x: f32, y: f32, c: i32) {
        let scale = 16.0;
        self.point((x * scale) as i32, (y * scale) as i32, c);
    }
}

struct Fp16 {
    v: i32,
}

const FP16_F: f32 = 256.0 * 256.0;

impl From<f32> for Fp16 {
    fn from(f: f32) -> Self {
        Self { v: (f * FP16_F) as i32 }
    }
}

impl Fp16 {
    pub fn get_int(&self) -> i32 {
        self.v >> 16
    }
    pub fn get_fract(&self) -> u32 {
        (self.v as u32) & 0xFFFF
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

struct Player {
    x: f32,
    y: f32,
    rot: f32,
    col_angle: [f32; WIDTH],
}

struct PlayerVel {
    forward: f32,
    rot: f32,
}

impl Default for Player {
    fn default() -> Self {
        let mut col_angle = [0.0; WIDTH];
        for i in 0..WIDTH / 2 {
            let f = (i as f32) / (WIDTH as f32 * 0.5);
            col_angle[WIDTH / 2 + i] = f.atan();
            col_angle[WIDTH / 2 - i - 1] = -f.atan();
        }

        Self {
            x: 1.0,
            y: 1.0,
            rot: 0.0,
            col_angle,
        }
    }
}

impl Player {
    pub fn int_pos(&self) -> (f32, f32) {
        (self.x.floor(), self.y.floor())
    }
    pub fn frac_pos(&self) -> (f32, f32) {
        (self.x.fract(), self.y.fract())
    }

    pub fn apply_vel(&mut self, player_vel: &PlayerVel, dt: f32) {
        self.rot += player_vel.rot * dt;
        while self.rot < 0.0 {
            self.rot += std::f32::consts::TAU;
        }

        while self.rot >= std::f32::consts::TAU {
            self.rot -= std::f32::consts::TAU;
        }

        let dx = self.rot.cos() * player_vel.forward * dt;
        let dy = self.rot.sin() * player_vel.forward * dt;
        self.x += dx;
        self.y += dy;
    }

    pub(crate) fn draw(&self, buffer: &mut Vec<u32>) {
        buffer.pointWorld(self.x, self.y, 0);

        for angle in self.col_angle.chunks(10) {
            let angle = angle[0];
            let dx = (self.rot + angle).cos() * 2.0;
            let dy = (self.rot + angle).sin() * 2.0;
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
    fn lookup(&self, x: f32, y: f32) -> i32 {
        if x < 0.0 || y < 0.0 || (x as usize) >= MAP_SIZE || (y as usize) >= MAP_SIZE {
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
        let (ex, ey) = (1.0 - dx, 1.0 - dy);
        for column in 0..WIDTH {
            let alpha_full = player.rot + player.col_angle[column];
            let alpha_full = if alpha_full < 0.0 {
                alpha_full + 2.0 * PI
            } else if alpha_full >= 2.0 * PI {
                alpha_full - 2.0 * PI
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
            let (comp_x, comp_y): (Box<dyn Fn(f32, f32) -> bool>, Box<dyn Fn(f32, f32) -> bool>) =
                if (0.0..FRAC_PI_2).contains(&alpha_full) {
                    quad_alpha = 0.0;
                    alpha = alpha_full - quad_alpha;
                    hstep_x = 1.0f32;
                    hstep_y = 1.0f32;
                    tx = 1.0 / alpha.tan();
                    ty = alpha.tan();
                    sx = tx * ey;
                    sy = ty * ex;
                    hx = x + hstep_x;
                    hy = y + hstep_y;
                    (Box::new(|a: f32, b: f32| a < b), Box::new(|a: f32, b: f32| a < b))
                } else if (FRAC_PI_2..PI).contains(&alpha_full) {
                    quad_alpha = FRAC_PI_2;
                    alpha = alpha_full - quad_alpha;
                    hstep_x = -1.0f32;
                    hstep_y = 1.0f32;
                    tx = -alpha.tan();
                    ty = 1.0 / alpha.tan();
                    sx = tx * ey;
                    sy = ty * dx;
                    hx = x;
                    hy = y + hstep_y;
                    xcor = -0.01;

                    (Box::new(|a: f32, b: f32| a > b), Box::new(|a: f32, b: f32| a < b))
                } else if (PI..(PI + FRAC_PI_2)).contains(&alpha_full) {
                    quad_alpha = PI;
                    alpha = alpha_full - quad_alpha;
                    hstep_x = -1.0f32;
                    hstep_y = -1.0f32;
                    tx = -1.0 / alpha.tan();
                    ty = -alpha.tan();
                    sx = tx * dy;
                    sy = ty * dx;
                    hx = x;
                    hy = y;
                    xcor = -0.01;
                    ycor = -0.01;
                    (Box::new(|a: f32, b: f32| a > b), Box::new(|a: f32, b: f32| a > b))
                } else if ((PI + FRAC_PI_2)..(2.0 * PI)).contains(&alpha_full) {
                    quad_alpha = PI + FRAC_PI_2;
                    alpha = alpha_full - quad_alpha;
                    hstep_x = 1.0f32;
                    hstep_y = -1.0f32;
                    tx = alpha.tan();
                    ty = -1.0 / alpha.tan();
                    sx = tx * dy;
                    sy = ty * ex;
                    hx = x + hstep_x;
                    hy = y;
                    ycor = -0.01;
                    (Box::new(|a: f32, b: f32| a < b), Box::new(|a: f32, b: f32| a > b))
                } else {
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
                let equal = (hy - ny).abs() < E && (hx - nx).abs() < E;
                while comp_y(ny, hy) || equal {
                    hit_tile = self.lookup(hx + xcor, ny + ycor);

                    if hit_tile != 0 {
                        screen.pointWorld(hx, ny, hit_tile);
                        // hit_dist = ((hx - player.x).powf(2.0) + (ny - player.y).powf(2.0)).sqrt();
                        dx = (hx - player.x) * hstep_x;
                        dy = (ny - player.y) * hstep_y;
                        break 'outer;
                    }

                    hx += hstep_x;
                    ny += ty;
                }

                while comp_x(nx, hx) {
                    hit_tile = self.lookup(nx + xcor, hy + ycor);
                    if hit_tile != 0 {
                        hit_tile += 8;
                        screen.pointWorld(nx, hy, hit_tile);
                        //hit_dist = ((nx - player.x).powf(2.0) + (hy - player.y).powf(2.0)).sqrt();
                        dx = (nx - player.x) * hstep_x;
                        dy = (hy - player.y) * hstep_y;
                        break 'outer;
                    }
                    hy += hstep_y;
                    nx += tx;
                }
            }
            let mid = 100;
            let c = 100.0;
            let beta = player.rot - quad_alpha;
            // let p = beta.cos() * dx + beta.sin() * dy;
            // let p = 2.0 / p;
            let p = (dx * dx + dy * dy).sqrt();
            let p = p * (beta - alpha).cos();
            // screen.pointWorld(
            //     player.x + p * alpha.cos(),
            //     player.y + p * alpha.sin(),
            //     hit_tile,
            // );
            let offs = if p > 0.0 { c / p } else { c };

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

    const DT: f32 = 1.0 / 60.0;

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut player_vel = PlayerVel { forward: 0.0, rot: 0.0 };
    let mut player = Player::default();
    let map = Map::default();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for i in buffer.iter_mut() {
            *i = 0; // write something more funny here!
        }

        for i in 0..16 {
            buffer.point(10 + i, 10 + i, i);
        }
        const FWD_SPEED: f32 = 10.0;
        const ROT_SPEED: f32 = 5.0;
        player_vel.forward = 0.0;
        player_vel.rot = 0.0;
        if window.is_key_down(Key::W) {
            player_vel.forward += FWD_SPEED;
        }
        if window.is_key_down(Key::S) {
            player_vel.forward -= FWD_SPEED;
        }
        if window.is_key_down(Key::D) {
            player_vel.rot += ROT_SPEED;
        }
        if window.is_key_down(Key::A) {
            player_vel.rot -= ROT_SPEED;
        }
        player.apply_vel(&player_vel, DT);
        player.draw(&mut buffer);

        let start = Instant::now();
        map.sweep_raycast(&mut buffer, &player);
        println!("time: {}ns", start.elapsed().as_nanos());

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
