use std::f32::consts::{FRAC_PI_2, PI};

use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 320;
const HEIGHT: usize = 200;
const COLORS: [u32; 16] = [
    0xFFFFFF, 0xFF0000, 0x00FF00, 0x0000FF, 0xFFFF00, 0x00FFFF, 0xFF00FF, 0xFF8000, 0x808080,
    0x800000, 0x008000, 0x000080, 0x808000, 0x008080, 0x800080, 0x804000,
];

const MAP: [[i32; 8]; 8] = [
    [8, 2, 3, 4, 5, 6, 7, 1],
    [7, 0, 0, 0, 0, 0, 0, 2],
    [6, 0, 0, 0, 0, 0, 0, 3],
    [5, 0, 0, 0, 0, 0, 0, 4],
    [4, 0, 0, 0, 0, 0, 3, 5],
    [3, 0, 0, 0, 0, 0, 4, 6],
    [2, 0, 0, 0, 0, 2, 5, 7],
    [1, 2, 3, 4, 5, 6, 7, 8],
];

trait Draw {
    fn point(&mut self, x: i32, y: i32, c: i32);
    fn pointWorld(&mut self, x: f32, y: f32, c: i32);
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

    fn pointWorld(&mut self, x: f32, y: f32, c: i32) {
        let scale = 16.0;
        self.point((x * scale) as i32, (y * scale) as i32, c);
    }
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
            col_angle[WIDTH / 2 + i] = f.tan() * 0.75;
            col_angle[WIDTH / 2 - i - 1] = -f.tan() * 0.75;
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
            let hstep_x; // = 1.0;
            let hstep_y; // = 1.0;
            let tx; // = 1.0 / alpha.tan();
            let ty; // = alpha.tan();
            let sx; // = tx * ey;
            let sy; // = ty * ex;
                    // let (comp_x: Box<dyn Fn(f32, f32) -> bool>, comp_y: Box<dyn Fn(f32, f32) -> bool>)
            let (comp_x, comp_y): (Box<dyn Fn(f32, f32) -> bool>, Box<dyn Fn(f32, f32) -> bool>) =
                if (0.0..FRAC_PI_2).contains(&alpha_full) {
                    alpha = alpha_full;
                    hstep_x = 1.0f32;
                    hstep_y = 1.0f32;
                    tx = 1.0 / alpha.tan();
                    ty = alpha.tan();
                    sx = tx * ey;
                    sy = ty * ex;
                    (
                        Box::new(|a: f32, b: f32| a < b),
                        Box::new(|a: f32, b: f32| a < b),
                    )
                } else if (FRAC_PI_2..PI).contains(&alpha_full) {
                    alpha = alpha_full - FRAC_PI_2;
                    hstep_x = -1.0f32;
                    hstep_y = 1.0f32;
                    tx = -alpha.tan();
                    ty = 1.0 / alpha.tan();
                    sx = tx * ey;
                    sy = ty * dx;
                    (
                        Box::new(|a: f32, b: f32| a > b),
                        Box::new(|a: f32, b: f32| a < b),
                    )
                } else if (PI..(PI + FRAC_PI_2)).contains(&alpha_full) {
                    alpha = alpha_full - PI;
                    hstep_x = -1.0f32;
                    hstep_y = -1.0f32;
                    tx = -1.0 / alpha.tan();
                    ty = -alpha.tan();
                    sx = tx * dy;
                    sy = ty * dx;
                    (
                        Box::new(|a: f32, b: f32| a > b),
                        Box::new(|a: f32, b: f32| a > b),
                    )
                } else if ((PI + FRAC_PI_2)..(2.0 * PI)).contains(&alpha_full) {
                    alpha = alpha_full - (PI + FRAC_PI_2);
                    hstep_x = 1.0f32;
                    hstep_y = -1.0f32;
                    tx = alpha.tan();
                    ty = -1.0 / alpha.tan();
                    sx = tx * dy;
                    sy = ty * ex;
                    (
                        Box::new(|a: f32, b: f32| a < b),
                        Box::new(|a: f32, b: f32| a > b),
                    )
                } else {
                    continue;
                };

            let mut hx = x + hstep_x;
            let mut hy = y + hstep_y;
            let mut nx = player.x + sx;
            let mut ny = player.y + sy;
            let mut hit_tile = 15;
            let mut hit_dist = 0.0;
            'outer: loop {
                // 'outer: for _ in 0..32 {
                const E: f32 = 1e-3;
                let equal = (hy - ny).abs() < E && (hx - nx).abs() < E;
                while comp_y(ny, hy) || equal {
                    hit_tile = self.lookup(hx, ny);

                    if hit_tile != 0 {
                        screen.pointWorld(hx, ny, hit_tile);
                        hit_dist = ((hx - player.x).powf(2.0) + (ny - player.y).powf(2.0)).sqrt();
                        break 'outer;
                    }

                    hx += hstep_x;
                    ny += ty;
                }

                while comp_x(nx, hx) {
                    hit_tile = self.lookup(nx, hy);
                    if hit_tile != 0 {
                        screen.pointWorld(nx, hy, hit_tile);
                        hit_dist = ((nx - player.x).powf(2.0) + (hy - player.y).powf(2.0)).sqrt();
                        break 'outer;
                    }
                    hy += hstep_y;
                    nx += tx;
                }
            }
            let mid = 100;
            let c = 100.0;
            let offs = if hit_dist > 0.0 { c / hit_dist } else { c };
            // for row in (mid - offs as i32)..(mid + offs as i32) {
            //     screen.point(column as i32, row, hit_tile);
            // }
            screen.point(column as i32, mid + offs as i32, hit_tile);
            screen.point(column as i32, mid - offs as i32, hit_tile);
        }
    }
}

#[test]
fn test_raycast() {
    let yrange = 0..1024;
    for yi in yrange {
        let xrange = 0..1024;
        for xi in xrange {
            let arange = 0..1024;
            for ai in arange {
                let (xp, yp) = (1.0 + (xi as f32 / 1024.0), 1.0 + (yi as f32 / 1024.0));
                let (x, y) = (xp.floor(), yp.floor());
                // if self.lookup(x, y) != 0 {
                //     return;
                // }

                let (dx, dy) = (xp.fract(), yp.fract());
                let (ex, ey) = (1.0 - dx, 1.0 - dy);
                let alpha = (ai as f32) / 1024.0;

                if !(0.0..FRAC_PI_2).contains(&alpha) {
                    panic!();
                }

                let tx = 1.0 / alpha.tan();
                let ty = alpha.tan();
                let sx = tx * ey;
                let sy = ty * ex;
                let mut hx = x + 1.0;
                let mut hy = y + 1.0;
                let mut nx = xp + sx;
                let mut ny = yp + sy;
                let mut hit = false;
                'outer:
                // 'outer: loop {
                for _ in 0..32 {
                    const E: f32 = 1e-3;
                    let equal = (hy - ny).abs() < E && (hx - nx).abs() < E;
                    // 'outer: for _ in 0..32 {
                    while ny < hy || equal {
                        let tile = if hx >= 7.0 || ny >= 7.0 { 1 } else { 0 };

                        if tile != 0 {
                            // screen.pointWorld(hx, ny, tile);
                            hit = true;
                            break 'outer;
                        }

                        hx += 1.0;
                        ny += ty;
                    }

                    while nx < hx {
                        let tile = if nx >= 7.0 || hy >= 7.0 { 1 } else { 0 };
                        if tile != 0 {
                            // screen.pointWorld(nx, hy, tile);
                            hit = true;
                            break 'outer;
                        }
                        hy += 1.0;
                        nx += tx;
                    }
                }
                assert!(hit);
            }
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

    let mut player_vel = PlayerVel {
        forward: 0.0,
        rot: 0.0,
    };
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

        map.sweep_raycast(&mut buffer, &player);

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
