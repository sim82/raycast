pub use crate::prelude::*;

#[derive(Debug)]
pub struct Player {
    pub x: Fp16,
    pub y: Fp16,
    pub rot: i32,
    pub trigger: bool,
}

#[derive(Debug)]
pub struct PlayerVel {
    pub forward: i32,
    pub right: i32,
    pub rot: i32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            x: 3.1.into(),
            y: 3.1.into(),
            rot: 0,
            trigger: false,
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

        let player_width: Fp16 = 0.40.into();

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

    pub fn draw(&self, buffer: &mut Vec<u32>) {
        buffer.point_world(self.x, self.y, 0);

        for angle in COL_ANGLE.chunks(10) {
            let angle = angle[0];
            let dx = fa_cos(fa_fix_angle(self.rot + angle)) * 2;
            let dy = fa_sin(fa_fix_angle(self.rot + angle)) * 2;
            buffer.point_world(self.x + dx, self.y + dy, 2);
        }
    }
}
