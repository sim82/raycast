use crate::prelude::*;

pub const COLORS: [u32; 16] = [
    0xFFFFFF, 0xFF0000, 0x00FF00, 0x0000FF, 0xFFFF00, 0x00FFFF, 0xFF00FF, 0xFF8000, 0x808080,
    0x800000, 0x008000, 0x000080, 0x808000, 0x008080, 0x800080, 0x804000,
];
pub trait Draw {
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
