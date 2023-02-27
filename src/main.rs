use std::f32::consts::{FRAC_PI_2, PI};

use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 320;
const HEIGHT: usize = 200;
const COLORS: [u32; 16] = [
    0xFFFFFF, 0xFF0000, 0x00FF00, 0x0000FF, 0xFFFF00, 0x00FFFF, 0xFF00FF, 0xFF8000, 0x808080,
    0x800000, 0x008000, 0x000080, 0x808000, 0x008080, 0x800080, 0x804000,
];

const MAP: [[i32; 8]; 8] = [
    [1, 2, 3, 4, 5, 6, 7, 8],
    [1, 0, 0, 0, 0, 0, 0, 8],
    [1, 0, 0, 0, 0, 0, 0, 8],
    [1, 0, 0, 0, 0, 0, 0, 8],
    [1, 0, 0, 0, 0, 0, 0, 8],
    [1, 0, 0, 0, 0, 0, 0, 8],
    [1, 0, 0, 0, 0, 0, 0, 8],
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
            || (y as usize) > HEIGHT
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
            let alpha = player.rot + player.col_angle[column];

            if !(0.0..FRAC_PI_2).contains(&alpha) {
                continue;
            }

            let tx = 1.0 / alpha.tan();
            let ty = alpha.tan();
            let sx = tx * ex;
            let sy = ty * ey;
            let mut hx = x + 1.0;
            let mut hy = y + 1.0;
            let mut nx = player.x + sx;
            let mut ny = player.y + sy;
            'outer: loop {
                // 'outer: for _ in 0..32 {
                while ny < hy {
                    let tile = self.lookup(hx, ny);

                    if tile != 0 {
                        screen.pointWorld(hx, ny, tile);
                        break 'outer;
                    }

                    hx += 1.0;
                    ny += ty;
                }

                while nx < hx {
                    let tile = self.lookup(nx, hy);
                    if tile != 0 {
                        screen.pointWorld(nx, hy, tile);
                        break 'outer;
                    }
                    hy += 1.0;
                    nx += tx;
                }
            }
        }
    }
}

#[test]
fn test_raycast() {
    let (xp, yp) = (1.16643822f32, 1.00872266f32);
    let (x, y) = (xp.floor(), yp.floor());
    // if self.lookup(x, y) != 0 {
    //     return;
    // }

    let (dx, dy) = (xp.fract(), yp.fract());
    let (ex, ey) = (1.0 - dx, 1.0 - dy);
    let alpha = 0.927944362;

    if !(0.0..FRAC_PI_2).contains(&alpha) {
        panic!();
    }

    let tx = 1.0 / alpha.tan();
    let ty = alpha.tan();
    let sx = tx * ex;
    let sy = ty * ey;
    let mut hx = x + 1.0;
    let mut hy = y + 1.0;
    let mut nx = xp + sx;
    let mut ny = yp + sy;
    'outer: loop {
        // 'outer: for _ in 0..32 {
        while ny < hy {
            let tile = if hx >= 7.0 || ny >= 7.0 { 1 } else { 0 };

            if tile != 0 {
                // screen.pointWorld(hx, ny, tile);
                break 'outer;
            }

            hx += 1.0;
            ny += ty;
        }

        while nx < hx {
            let tile = if nx >= 7.0 || hy >= 7.0 { 1 } else { 0 };
            if tile != 0 {
                // screen.pointWorld(nx, hy, tile);
                break 'outer;
            }
            hy += 1.0;
            nx += tx;
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
        forward: 10.0,
        rot: PI,
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

        player.apply_vel(&player_vel, DT);
        player.draw(&mut buffer);

        map.sweep_raycast(&mut buffer, &player);

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
