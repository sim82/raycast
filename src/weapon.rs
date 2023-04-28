use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::prelude::*;
use crate::sprite::SpriteSceenSetup;
use anyhow::anyhow;

#[derive(Debug, PartialEq, Eq)]
pub enum WeaponType {
    Knife,
    Gun,
    Machinegun,
    Chaingun,
}

impl WeaponType {
    pub fn from_weapon_id(id: i32) -> Result<WeaponType> {
        Ok(match id {
            1 => WeaponType::Knife,
            2 => WeaponType::Gun,
            3 => WeaponType::Machinegun,
            4 => WeaponType::Chaingun,
            x => return Err(anyhow!("unhandled weapon type {x}")),
        })
    }

    pub fn map_state_label(&self, state: &str) -> String {
        match self {
            WeaponType::Knife => format!("weapon_knife::{state}"),
            WeaponType::Gun => format!("weapon_gun::{state}"),
            WeaponType::Machinegun => format!("weapon_machinegun::{state}"),
            WeaponType::Chaingun => format!("weapon_chaingun::{state}"),
        }
    }
}

impl ms::Loadable for WeaponType {
    fn read_from(r: &mut dyn std::io::Read) -> Result<Self> {
        WeaponType::from_weapon_id(r.read_i32::<LittleEndian>()?)
    }
}

impl ms::Writable for WeaponType {
    fn write(&self, w: &mut dyn std::io::Write) -> Result<()> {
        let v = match self {
            WeaponType::Knife => 1,
            WeaponType::Gun => 2,
            WeaponType::Machinegun => 3,
            WeaponType::Chaingun => 4,
        };
        w.write_i32::<LittleEndian>(v)?;
        Ok(())
    }
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
            ammo: 99,
            exec_ctx: ExecCtx::new("weapon_gun::ready", &IMG_WL6).unwrap(),
        }
    }
}

impl Weapon {
    pub fn run(&mut self, fire: bool, new_weapon_type: Option<i32>) -> bool {
        if let Some(new_weapon_type) = new_weapon_type {
            if let Ok(weapon_type) = WeaponType::from_weapon_id(new_weapon_type) {
                self.selected_weapon = weapon_type;
            }
        }

        let shoot = matches!(self.exec_ctx.state.take_action(), Function::ActionShoot);

        if shoot && self.selected_weapon != WeaponType::Knife {
            self.ammo -= 1;
        }

        // weapons only evaluate think action on state end (they do not need to 'think' constantly like enemies)
        if self.exec_ctx.state.ticks <= 0 {
            match self.exec_ctx.state.think {
                // weapon idle + fire -> attack state
                Function::ThinkStand if fire => {
                    self.exec_ctx
                        .jump_label(&self.selected_weapon.map_state_label("attack"))
                        .unwrap_or_else(|err| panic!("failed to jump to state attack: {err:?}"));
                }
                // weapon idle + no fire -> restart ready state of selected weapon (i.e. change weapon if requested)
                Function::ThinkStand if !fire => {
                    self.exec_ctx
                        .jump_label(&self.selected_weapon.map_state_label("ready"))
                        .unwrap_or_else(|err| panic!("failed to jump to state ready: {err:?}"));
                }
                // weapon in attack state + ammo depleted -> restart attack state (keep weapon raised, but don't proceed further)
                Function::ThinkPath
                    if fire && self.selected_weapon != WeaponType::Knife && self.ammo <= 0 =>
                {
                    self.exec_ctx
                        .jump_label(&self.selected_weapon.map_state_label("attack"))
                        .unwrap_or_else(|err| panic!("failed to jump to state attack: {err:?}"));
                }
                // weapon in attack state + no fire -> lower weapon, proceed to idle
                Function::ThinkPath if !fire => {
                    self.exec_ctx
                        .jump_label(&self.selected_weapon.map_state_label("lower"))
                        .unwrap_or_else(|err| panic!("failed to jump to state attack: {err:?}"));
                }
                _ => self.exec_ctx.jump(self.exec_ctx.state.next).unwrap(),
            }
        }
        self.exec_ctx.state.ticks -= 1;

        // match self.exec_ctx.state.think {
        //     // weapon idle + fire -> attack state
        //     Think::Stand if fire => {
        //         self.exec_ctx
        //             .jump_label(&self.selected_weapon.map_state_label("attack"))
        //             .unwrap_or_else(|err| panic!("failed to jump to state attack: {err:?}"));
        //     }
        //     // weapon idle + no fire -> restart ready state of selected weapon (i.e. change weapon if requested)
        //     Think::Stand if !fire => {
        //         self.exec_ctx
        //             .jump_label(&self.selected_weapon.map_state_label("ready"))
        //             .unwrap_or_else(|err| panic!("failed to jump to state ready: {err:?}"));
        //     }
        //     // weapon in attack state + ammo depleted -> restart attack state (keep weapon raised, but don't proceed further)
        //     Think::Path if fire && self.selected_weapon != WeaponType::Knife && self.ammo <= 0 => {
        //         self.exec_ctx
        //             .jump_label(&self.selected_weapon.map_state_label("attack"))
        //             .unwrap_or_else(|err| panic!("failed to jump to state attack: {err:?}"));
        //     }
        //     // weapon in attack state + no fire -> lower weapon, proceed to idle
        //     Think::Path if !fire => {
        //         self.exec_ctx
        //             .jump_label(&self.selected_weapon.map_state_label("lower"))
        //             .unwrap_or_else(|err| panic!("failed to jump to state attack: {err:?}"));
        //     }
        //     _ => (),
        // }
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
            selected_weapon: WeaponType::read_from(r)?,
            exec_ctx: ExecCtx::read_from(r, &IMG_WL6)?,
        })
    }
}

impl ms::Writable for Weapon {
    fn write(&self, w: &mut dyn std::io::Write) -> Result<()> {
        w.write_i32::<LittleEndian>(self.ammo)?;
        self.selected_weapon.write(w)?;
        self.exec_ctx.write(w)?;
        Ok(())
    }
}
