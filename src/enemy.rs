use crate::{prelude::*, thing_def::EnemyType};
use anyhow::anyhow;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

fn think_chase(_thing: &mut Enemy, _map_dynamic: &MapDynamic) {

    // thing.direction
}

// fn think_path(thing: &mut Thing) {}
fn think_path(thing: &mut Enemy, map_dynamic: &mut MapDynamic, things: &Things, static_index: usize) {
    match &mut thing.path_action {
        Some(PathAction::Move { dist }) if *dist > FP16_ZERO => {
            let (dx, dy) = thing.direction.tile_offset();

            if *dist == FP16_ONE {
                // check if we would bump into door
                let enter_x = thing.x.get_int() + dx;
                let enter_y = thing.y.get_int() + dy;
                match map_dynamic.lookup_tile(enter_x, enter_y) {
                    MapTile::Door(_, _, door_id) => {
                        thing.path_action = Some(PathAction::WaitForDoor { door_id });
                        // FIXME: maybe directly continue with next state
                        return;
                    }
                    MapTile::Blocked(_) | MapTile::Wall(_) | MapTile::PushWall(_, _) => {
                        println!("path blocked. waiting");
                        // thing.set_state("stand");
                        // return;
                        // thing.direction = thing.direction.opposite();
                        *dist = FP16_ZERO;
                        return;
                    }
                    MapTile::Walkable(_, _) => {
                        if things.blockmap.is_occupied(enter_x, enter_y) {
                            println!("path occupied in blockmap. waiting");
                            // thing.set_state("stand");
                            // return;
                            // thing.direction = thing.direction.opposite();
                            *dist = FP16_ZERO;
                            return;
                        }
                    }
                    _ => (),
                }
            }

            // still some way to go on old action
            *dist -= crate::fp16::FP16_FRAC_128;
            thing.x += crate::fp16::FP16_FRAC_128 * dx;
            thing.y += crate::fp16::FP16_FRAC_128 * dy;
        }
        Some(PathAction::Move { dist: _ }) => {
            // check how to continue
            let xaligned = thing.x.fract() == FP16_HALF;
            let yaligned = thing.y.fract() == FP16_HALF;
            if !(xaligned && yaligned) {
                // TODO: recover, e.g. teleport to next tile center. This should not happen in a fixedpoint world, but who knows...
                println!("PathAction::Move ended not on tile center. Aborting.");
                thing.path_action = None;
            }
            match map_dynamic.lookup_tile(thing.x.get_int(), thing.y.get_int()) {
                MapTile::Walkable(_, Some(path_direction)) => {
                    // change direction
                    println!("change path direction {path_direction:?}");
                    thing.direction = path_direction;
                    thing.path_action = Some(PathAction::Move { dist: FP16_ONE })
                }
                MapTile::Walkable(_, None) => {
                    // continue in same direction
                    thing.path_action = Some(PathAction::Move { dist: FP16_ONE })
                }
                x => {
                    println!("hit non-walkable {x:?}");
                    thing.path_action = None;
                }
            }
        }
        Some(PathAction::WaitForDoor { door_id }) => {
            if map_dynamic.try_open_and_block_door(*door_id, static_index as i32) {
                thing.path_action = Some(PathAction::MoveThroughDoor {
                    dist: FP16_ONE,
                    door_id: *door_id,
                })
            }
        }
        Some(PathAction::MoveThroughDoor { dist, door_id }) => {
            if *dist > FP16_ZERO {
                *dist -= crate::fp16::FP16_FRAC_128;
                let (dx, dy) = thing.direction.tile_offset();
                thing.x += crate::fp16::FP16_FRAC_128 * dx;
                thing.y += crate::fp16::FP16_FRAC_128 * dy;
            } else {
                // unblock door and keep moving in same direction
                map_dynamic.unblock_door(*door_id, static_index as i32);
                thing.path_action = Some(PathAction::Move { dist: FP16_ONE });
            }
        }
        None => {
            // println!("no PathAction.")
        }
    }
}

pub enum PathAction {
    Move { dist: Fp16 },
    WaitForDoor { door_id: usize },
    MoveThroughDoor { dist: Fp16, door_id: usize },
}

impl ms::Loadable for PathAction {
    fn read_from(r: &mut dyn Read) -> Result<Self> {
        Ok(match r.read_u8()? {
            0 => PathAction::Move {
                dist: Fp16::read_from(r)?,
            },
            1 => PathAction::WaitForDoor {
                door_id: r.read_u32::<LittleEndian>()? as usize,
            },
            2 => PathAction::MoveThroughDoor {
                dist: Fp16::read_from(r)?,
                door_id: r.read_u32::<LittleEndian>()? as usize,
            },
            x => return Err(anyhow!("unhandled PathAction discriminator: {x}")),
        })
    }
}

impl ms::Writable for PathAction {
    fn write(&self, w: &mut dyn Write) -> Result<()> {
        match self {
            PathAction::Move { dist } => {
                w.write_u8(0)?;
                dist.write(w)?;
            }
            PathAction::WaitForDoor { door_id } => {
                w.write_u8(1)?;
                w.write_u32::<LittleEndian>(*door_id as u32)?
            }
            PathAction::MoveThroughDoor { dist, door_id } => {
                w.write_u8(2)?;
                dist.write(w)?;
                w.write_u32::<LittleEndian>(*door_id as u32)?
            }
        }
        Ok(())
    }
}

pub struct Enemy {
    exec_ctx: ExecCtx,
    enemy_type: EnemyType,
    direction: Direction,
    path_action: Option<PathAction>,
    health: i32,
    pub x: Fp16,
    pub y: Fp16,
}

impl ms::Loadable for Enemy {
    fn read_from(r: &mut dyn Read) -> Result<Self> {
        let exec_ctx = ExecCtx::read_from(r)?;
        let enemy_type = EnemyType::read_from(r)?;
        let direction = Direction::read_from(r)?;
        let path_action = Option::<PathAction>::read_from(r)?;
        let health = r.read_i32::<LittleEndian>()?;
        let x = Fp16::read_from(r)?;
        let y = Fp16::read_from(r)?;
        Ok(Enemy {
            exec_ctx,
            enemy_type,
            direction,
            path_action,
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
        self.path_action.write(w)?;
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
    pub fn update(&mut self, map_dynamic: &mut MapDynamic, things: &Things, static_index: usize) {
        if self.exec_ctx.state.ticks <= 0 {
            self.exec_ctx.jump(self.exec_ctx.state.next).unwrap();
        }

        match self.exec_ctx.state.think {
            Think::None => (),
            Think::Stand => (),
            Think::Path => think_path(self, map_dynamic, things, static_index),
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
        let (start_label, path_action) = match state {
            crate::thing_def::EnemyState::Standing => ("stand", None),
            crate::thing_def::EnemyState::Patrolling => ("path", Some(PathAction::Move { dist: FP16_ONE })),
        };
        let exec_ctx = ExecCtx::new(&enemy_type.map_label(start_label)).unwrap();

        Enemy {
            direction,
            path_action,
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
