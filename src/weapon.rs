use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

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
    pub selected_weapon: WeaponType,
    pub ammo: i32,
    pub exec_ctx: ExecCtx,
}

impl Default for Weapon {
    fn default() -> Self {
        Self {
            // selected_weapon: WeaponType::Gun,
            selected_weapon: WeaponType::Machinegun,
            ammo: 9999,
            exec_ctx: ExecCtx::new("weapon_chaingun::ready").unwrap(),
        }
    }
}

impl Weapon {
    pub fn run(&mut self, fire: bool) -> bool {
        let shoot = match self.exec_ctx.state.take_action() {
            Action::None => false,
            Action::Die => true,
        };

        if self.exec_ctx.state.ticks <= 0 {
            self.exec_ctx.jump(self.exec_ctx.state.next).unwrap();
        }
        self.exec_ctx.state.ticks -= 1;

        match self.exec_ctx.state.think {
            Think::Stand if fire => {
                self.exec_ctx
                    .jump_label("weapon_chaingun::attack")
                    .unwrap_or_else(|err| panic!("failed to jump to state attack: {err:?}"));
            }
            _ => (),
        }
        shoot
    }

    pub fn get_sprite(&self) -> SpriteSceenSetup {
        SpriteSceenSetup {
            z: FP16_ZERO,
            screen_x: WIDTH as i32 / 2,
            id: self.exec_ctx.state.id,
            owner: 0,
        }
    }
}

impl ms::Loadable for Weapon {
    fn read_from(r: &mut dyn std::io::Read) -> Result<Self> {
        Ok(Self {
            ammo: r.read_i32::<LittleEndian>()?,
            selected_weapon: WeaponType::Gun,
            exec_ctx: ExecCtx::read_from(r)?,
        })
    }
}

impl ms::Writable for Weapon {
    fn write(&self, w: &mut dyn std::io::Write) -> Result<()> {
        w.write_i32::<LittleEndian>(self.ammo)?;
        self.exec_ctx.write(w)?;
        Ok(())
    }
}
