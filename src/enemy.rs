use crate::{ms::Loadable, prelude::*, thing_def::EnemyType};
use anyhow::anyhow;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    io::{Cursor, Read, Write},
};

fn think_chase(_thing: &mut Enemy, _map_dynamic: &MapDynamic) {

    // thing.direction
}

// fn think_path(thing: &mut Thing) {}
fn think_path(thing: &mut Enemy, map_dynamic: &MapDynamic) {
    let (dx, dy) = thing.direction.tile_offset();
    thing.x += crate::fp16::FP16_FRAC_128 * dx;
    thing.y += crate::fp16::FP16_FRAC_128 * dy;
    let xaligned = thing.x.fract() == FP16_HALF;
    let yaligned = thing.y.fract() == FP16_HALF;
    // println!("chase {:?} {:?}", xaligned, yaligned);
    if xaligned && yaligned {
        if let MapTile::Walkable(_, Some(path_direction)) =
            map_dynamic.lookup_tile(thing.x.get_int(), thing.y.get_int())
        {
            thing.direction = path_direction;
        }
    }
}

pub struct Enemy {
    exec_ctx: ExecCtx,
    enemy_type: EnemyType,
    direction: Direction,
    health: i32,
    x: Fp16,
    y: Fp16,
}

impl ms::Loadable for Enemy {
    fn read_from(r: &mut dyn Read) -> Result<Self> {
        let exec_ctx = ExecCtx::read_from(r)?;
        let enemy_type = EnemyType::read_from(r)?;
        let direction = Direction::read_from(r)?;
        let health = r.read_i32::<LittleEndian>()?;
        let x = Fp16::read_from(r)?;
        let y = Fp16::read_from(r)?;
        Ok(Enemy {
            exec_ctx,
            enemy_type,
            direction,
            health,
            x,
            y,
        })
    }
}

impl ms::Writable for Enemy {
    fn write(&self, w: &mut dyn Write) -> Result<()> {
        self.exec_ctx.write(w)?;
        self.enemy_type.write(w)?;
        self.direction.write(w)?;
        w.write_i32::<LittleEndian>(self.health)?;
        self.x.write(w)?;
        self.y.write(w)?;
        Ok(())
    }
}

trait LabelMapper {
    fn map_label(&self, label: &str) -> String;
}

impl LabelMapper for EnemyType {
    fn map_label(&self, name: &str) -> String {
        match self {
            EnemyType::Brown => format!("brown::{name}"),
            EnemyType::Blue => format!("blue::{name}"),
            EnemyType::White => format!("white::{name}"),
            EnemyType::Rotten => format!("rotten::{name}"),
            EnemyType::Woof => format!("furry::{name}"),
        }
    }
}

impl Enemy {
    // pub fn set_state(&mut self, states: &'static [State]) {
    //     self.states = states;
    //     self.cur = 0;
    //     self.timeout = self.states[0].1;
    // }
    pub fn set_state(&mut self, name: &str) {
        let label = self.enemy_type.map_label(name);
        self.exec_ctx
            .jump_label(&label)
            .unwrap_or_else(|err| panic!("failed to jump to state {label}: {err:?}"));
        println!("state: {self:?}");
    }
    pub fn update(&mut self, map_dynamic: &MapDynamic) {
        if self.exec_ctx.state.ticks <= 0 {
            self.exec_ctx.jump(self.exec_ctx.state.next).unwrap();
        }

        match self.exec_ctx.state.think {
            Think::None => (),
            Think::Stand => (),
            Think::Path => think_path(self, map_dynamic),
            Think::Chase => think_chase(self, map_dynamic),
        }
        // self.states[self.cur].2();

        self.exec_ctx.state.ticks -= 1;
    }
    pub fn hit(&mut self) {
        self.health -= 7;

        if self.health > 10 {
            self.set_state("pain1");
        } else if self.health > 0 {
            self.set_state("pain2");
        } else {
            self.set_state("die");
        }
    }
    pub fn get_sprite(&self) -> (SpriteIndex, Fp16, Fp16) {
        let id = if self.exec_ctx.state.directional {
            SpriteIndex::Directional(self.exec_ctx.state.id, self.direction)
        } else {
            SpriteIndex::Undirectional(self.exec_ctx.state.id)
        };
        (id, self.x, self.y)
    }

    pub fn spawn(
        direction: Direction,
        _difficulty: crate::thing_def::Difficulty,
        enemy_type: EnemyType,
        state: crate::thing_def::EnemyState,
        thing_def: &ThingDef,
    ) -> Enemy {
        let start_label = match state {
            crate::thing_def::EnemyState::Standing => "stand",
            crate::thing_def::EnemyState::Patrolling => "path",
        };
        let exec_ctx = ExecCtx::new(&enemy_type.map_label(start_label)).unwrap();

        Enemy {
            direction,
            exec_ctx,
            enemy_type,
            health: 25,
            x: thing_def.x,
            y: thing_def.y,
        }
    }
}

impl std::fmt::Debug for Enemy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Enemy")
            .field("state", &self.exec_ctx.state)
            .field("direction", &self.direction)
            .field("health", &self.health)
            .finish()
    }
}
