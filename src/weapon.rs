use std::io::Cursor;

use crate::prelude::*;
use crate::sprite::SpriteSceenSetup;
use anyhow::anyhow;
use state_bc::opcode::Value;

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
        WeaponType::from_weapon_id(r.readi32()?)
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
        w.writei32(v)?;
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
    shoot: bool, // transitional: set during function dispatch
}

impl Default for Weapon {
    fn default() -> Self {
        Self {
            // selected_weapon: WeaponType::Gun,
            selected_weapon: WeaponType::Machinegun,
            ammo: 99,
            exec_ctx: ExecCtx::new("weapon_gun::ready", &IMG_WL6).unwrap(),
            shoot: false,
        }
    }
}

impl Weapon {
    fn exec_code(
        &mut self,
        code_offs: i32,
        fire: bool,
        audio_service: &mut dyn AudioService,
    ) -> Result<()> {
        let mut env = opcode::Env::default();
        let mut cursor = Cursor::new(&self.exec_ctx.image.code[code_offs as usize..]);
        loop {
            let state = opcode::exec(&mut cursor, &mut env).expect("error on bytecode exec");
            match state {
                opcode::Event::Stop => break,
                opcode::Event::Call(function) => {
                    self.dispatch_call(function, fire);
                }
                opcode::Event::Trap => {
                    match env.stack.pop() {
                        Some(Value::U8(0)) => env.stack.push(opcode::Value::Bool(fire)),
                        Some(Value::U8(1)) => env.stack.push(opcode::Value::Bool(
                            fire && self.selected_weapon != WeaponType::Knife && self.ammo <= 0,
                        )),
                        Some(Value::U8(2)) => {
                            if let Some(offs) = self.exec_ctx.image.get_state_offs_by_label(
                                &self.selected_weapon.map_state_label("ready"),
                            ) {
                                env.stack.push(Value::I32(offs));
                            } else {
                                panic!("could not find ready state for current weapon");
                            }
                        }
                        Some(Value::U8(4)) => {
                            if let Some(Value::U8(snd_id)) = env.stack.pop() {
                                audio_service.play_sound(snd_id as i32);
                            }
                        }
                        x => panic!("unexpected trap code {x:?}"),
                    }
                }
                opcode::Event::GoState => {
                    match env.stack.pop() {
                        Some(Value::I32(offs)) => {
                            self.exec_ctx
                                .jump(offs)
                                .unwrap_or_else(|_| panic!("jump to {offs} failed."));
                        }
                        Some(x) => return Err(anyhow!("unhandled opcode::Value {x:?}")),
                        None => return Err(anyhow!("stack underflow")),
                    };
                    break;
                }
            }
        }
        Ok(())
    }
    fn dispatch_call(&mut self, function: Function, _fire: bool) {
        match function {
            // Function::ThinkStand => {
            //     self.exec_ctx
            //         .jump_label(&self.selected_weapon.map_state_label("attack"))
            //         .unwrap_or_else(|err| panic!("failed to jump to state attack: {err:?}"));
            // }
            // Function::ThinkPath => {
            //     self.exec_ctx
            //         .jump_label(&self.selected_weapon.map_state_label("ready"))
            //         .unwrap_or_else(|err| panic!("failed to jump to state ready: {err:?}"));
            // }
            // Function::ThinkChase => {
            //     self.exec_ctx
            //         .jump_label(&self.selected_weapon.map_state_label("lower"))
            //         .unwrap_or_else(|err| panic!("failed to jump to state attack: {err:?}"));
            // }
            Function::ActionShoot => self.shoot = true,
            _ => todo!(),
        }
    }
    pub fn run(
        &mut self,
        fire: bool,
        new_weapon_type: Option<i32>,
        audio_service: &mut dyn AudioService,
    ) -> bool {
        self.shoot = false;
        if let Some(new_weapon_type) = new_weapon_type {
            if let Ok(weapon_type) = WeaponType::from_weapon_id(new_weapon_type) {
                self.selected_weapon = weapon_type;
            }
        }

        if self.exec_ctx.state.ticks <= 0 {
            self.exec_code(self.exec_ctx.state.action_offs, fire, audio_service)
                .expect("exec_code failed.");
            if self.exec_ctx.state.ticks <= 0 {
                self.exec_ctx.jump(self.exec_ctx.state.next).unwrap();
            }
        }
        self.exec_ctx.state.ticks -= 1;

        if self.shoot && self.selected_weapon != WeaponType::Knife {
            self.ammo -= 1;
        }
        if self.shoot {
            println!("player shoot");
        }
        self.shoot
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
            ammo: r.readi32()?,
            selected_weapon: WeaponType::read_from(r)?,
            exec_ctx: ExecCtx::read_from(r, &IMG_WL6)?,
            shoot: false,
        })
    }
}

impl ms::Writable for Weapon {
    fn write(&self, w: &mut dyn std::io::Write) -> Result<()> {
        w.writei32(self.ammo)?;
        self.selected_weapon.write(w)?;
        self.exec_ctx.write(w)?;
        Ok(())
    }
}
