// the only clippy lints that are really annying and unnecessary
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::collapsible_if)]

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

pub use crate::prelude::*;

use self::ms::{Loadable, Writable};

#[derive(Debug)]
pub struct Player {
    pub x: Fp16,
    pub y: Fp16,
    pub rot: i32,
    pub trigger: bool,
    pub shoot: bool,
    pub shoot_timeout: i32,
    pub weapon: Weapon,
    pub health: i32,
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
            shoot: false,
            shoot_timeout: 0,
            weapon: Default::default(),
            health: 100,
        }
    }
}

impl Writable for Player {
    fn write(&self, w: &mut dyn std::io::Write) -> Result<()> {
        w.writei32(self.x.v)?;
        w.writei32(self.y.v)?;
        w.writei32(self.rot)?;
        w.writeu8(if self.trigger { 1 } else { 0 })?;
        w.writeu8(if self.shoot { 1 } else { 0 })?;
        w.writei32(self.shoot_timeout)?;
        self.weapon.write(w)?;
        w.writei32(self.health)?;
        Ok(())
    }
}

impl Loadable for Player {
    fn read_from(r: &mut dyn std::io::Read) -> Result<Self> {
        let x = Fp16 {
            v: r.read_i32::<LittleEndian>()?,
        };
        let y = Fp16 {
            v: r.read_i32::<LittleEndian>()?,
        };
        let rot = r.read_i32::<LittleEndian>()?;
        let trigger = r.read_u8()? != 0;
        let shoot = r.read_u8()? != 0;
        let shoot_timeout = r.read_i32::<LittleEndian>()?;
        let weapon = Weapon::read_from(r)?;
        let health = r.read_i32::<LittleEndian>()?;
        Ok(Self {
            x,
            y,
            rot,
            trigger,
            shoot,
            shoot_timeout,
            weapon,
            health,
        })
    }
}

impl Player {
    pub fn int_pos(&self) -> (i32, i32) {
        (self.x.get_int(), self.y.get_int())
    }
    pub fn frac_pos(&self) -> (Fp16, Fp16) {
        (self.x.fract(), self.y.fract())
    }

    pub fn apply_vel(
        &mut self,
        player_vel: &PlayerVel,
        dt: Fp16,
        map_dynamic: &MapDynamic,
        collisions: bool,
    ) {
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
        let max_loops = if collisions { 2 } else { 0 };
        for _ in 0..max_loops {
            let (tx, ty) = self.get_corners();

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

                // if !map.can_walk(x.get_int(), y.get_int()) {
                // println!("collision {}", ti + 1);

                let test_x = map_dynamic.can_walk(x.get_int(), ty[ti].get_int());
                let test_y = map_dynamic.can_walk(tx[ti].get_int(), y.get_int());

                can_move_x &= test_x;
                can_move_y &= test_y;
                // }
            }

            let doorsnap_threshold: Fp16 = 0.05.into();
            if dx.v.abs() > dy.v.abs() {
                if !can_move_x
                    && (self.y.fract() > FP16_HALF - doorsnap_threshold
                        || self.y.fract() < FP16_HALF + doorsnap_threshold)
                {
                    match map_dynamic
                        .lookup_tile(self.x.get_int() + dx.v.signum(), self.y.get_int())
                    {
                        MapTile::Door(_, _, door_id)
                            if map_dynamic.door_states[door_id].open_f > FP16_HALF =>
                        {
                            self.y = FP16_HALF + self.y.get_int().into();
                            continue;
                        }
                        _ => (),
                    }
                }
            } else {
                if !can_move_y
                    && (self.x.fract() > FP16_HALF - doorsnap_threshold
                        || self.x.fract() < FP16_HALF + doorsnap_threshold)
                {
                    match map_dynamic
                        .lookup_tile(self.x.get_int(), self.y.get_int() + dy.v.signum())
                    {
                        MapTile::Door(_, _, door_id)
                            if map_dynamic.door_states[door_id].open_f > FP16_HALF =>
                        {
                            self.x = FP16_HALF + self.x.get_int().into();
                            continue;
                        }
                        _ => (),
                    }
                }
            }

            if can_move_x {
                self.x += dx;
            }
            if can_move_y {
                self.y += dy;
            }
            break;
        }
    }

    pub fn get_corners(&self) -> ([Fp16; 4], [Fp16; 4]) {
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
        (tx, ty)
    }

    pub fn draw<D: Draw + ?Sized>(&self, buffer: &mut D) {
        buffer.point_world(self.x, self.y, 0);

        for angle in COL_ANGLE.chunks(10) {
            let angle = angle[0];
            let dx = fa_cos(fa_fix_angle(self.rot + angle)) * 2;
            let dy = fa_sin(fa_fix_angle(self.rot + angle)) * 2;
            buffer.point_world(self.x + dx, self.y + dy, 2);
        }
    }
}
