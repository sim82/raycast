use crate::prelude::*;
use crate::sprite::SpriteSceenSetup;

#[derive(Debug)]
pub enum WeaponType {
    Knife,
    Gun,
    Machinegun,
    Chaingun,
}

impl WeaponType {
    pub fn get_timeout(&self) -> i32 {
        match self {
            WeaponType::Knife => todo!(),
            WeaponType::Gun => 30,
            WeaponType::Machinegun => 10,
            WeaponType::Chaingun => 1,
        }
    }
}
#[derive(Debug)]
pub struct Weapon {
    pub timeout: i32,
    pub selected_weapon: WeaponType,
    pub ammo: i32,
}

impl Default for Weapon {
    fn default() -> Self {
        Self {
            timeout: 0,
            // selected_weapon: WeaponType::Gun,
            selected_weapon: WeaponType::Machinegun,
            ammo: 9999,
        }
    }
}

impl Weapon {
    pub fn run(&mut self, fire: bool) -> bool {
        if fire && self.timeout <= 0 && self.ammo > 0 {
            self.ammo -= 1;
            self.timeout = self.selected_weapon.get_timeout();
            return true;
        }

        if self.timeout > 0 {
            self.timeout -= 1;
        }
        false
    }

    pub fn get_sprite(&self) -> SpriteSceenSetup {
        let id = match self.selected_weapon {
            WeaponType::Knife => todo!(),
            WeaponType::Gun => {
                422 + if self.timeout == 0 {
                    0
                } else {
                    4 - (self.timeout / 7).min(4)
                }
            }
            WeaponType::Machinegun => {
                427 + if self.timeout == 0 {
                    0
                } else {
                    4 - (self.timeout / 2).min(431)
                }
            }
            WeaponType::Chaingun => todo!(),
        };
        println!("weapon: {id}");
        SpriteSceenSetup {
            z: FP16_ZERO,
            screen_x: WIDTH as i32 / 2,
            id,
            owner: 0,
        }
    }
}
