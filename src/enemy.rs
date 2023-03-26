use crate::{fp16::FP16_FRAC_64, prelude::*, thing_def::EnemyType};
use anyhow::anyhow;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

fn check_player_sight(thing: &mut Enemy, things: &Things, map_dynamic: &mut MapDynamic, _static_index: usize) -> bool {
    let dx = things.player_x - thing.x.get_int();
    let dy = things.player_y - thing.y.get_int();

    let in_front = match thing.direction {
        Direction::East => dx > 0,
        Direction::SouthEast => dx + dy > 0,
        Direction::South => dy > 0,
        Direction::SouthWest => dx - dy < 0,
        Direction::West => dx < 0,
        Direction::NorthWest => dx + dy < 0,
        Direction::North => dy < 0,
        Direction::NorthEast => dx - dy > 0,
    };

    if !in_front {
        return false;
    }

    bresenham_trace(
        thing.x.get_int(),
        thing.y.get_int(),
        things.player_x,
        things.player_y,
        |x, y| match map_dynamic.lookup_tile(x, y) {
            MapTile::Walkable(_, _) => true,
            MapTile::Door(_, _, door_id) => map_dynamic.door_states[door_id].open_f > FP16_HALF,
            _ => false,
        },
    )
}

fn try_update_pathdir(thing: &Enemy, map_dynamic: &mut MapDynamic) -> Option<Direction> {
    // check how to continue
    let xaligned = thing.x.fract() == FP16_HALF;
    let yaligned = thing.y.fract() == FP16_HALF;
    if !(xaligned && yaligned) {
        // TODO: recover, e.g. teleport to next tile center. This should not happen in a fixedpoint world, but who knows...
        println!("PathAction::Move ended not on tile center. Aborting.");
        return None;
    }
    if let MapTile::Walkable(_, Some(path_direction)) = map_dynamic.lookup_tile(thing.x.get_int(), thing.y.get_int()) {
        Some(path_direction)
    } else {
        None
    }
}

fn try_find_pathaction(thing: &Enemy, map_dynamic: &mut MapDynamic, things: &Things) -> Option<PathAction> {
    let (dx, dy) = thing.direction.tile_offset();
    // check the block we are about to enter
    let enter_x = thing.x.get_int() + dx;
    let enter_y = thing.y.get_int() + dy;
    match map_dynamic.lookup_tile(enter_x, enter_y) {
        MapTile::Door(_, _, door_id) if !thing.direction.is_diagonal() => {
            println!("open door");
            Some(PathAction::WaitForDoor { door_id })
        }
        // MapTile::Door(_, _, door_id) => {
        //     println!("door diagonal");
        //     None
        // }
        MapTile::Walkable(_, _) => {
            if things.blockmap.is_occupied(enter_x, enter_y) {
                println!("path occupied in blockmap. waiting");
                None
            } else {
                Some(PathAction::Move { dist: FP16_ONE, dx, dy })
            }
        }
        MapTile::Blocked(_) | MapTile::Wall(_) | MapTile::PushWall(_, _) => {
            // fixup path direction pointing diagonally into wall. The expectation seems to be that the actor continues going in the
            // diagonal direction but 'slide' along walls (e.g. the dogs in E1M6)
            if dx != 0 && dy != 0 {
                match map_dynamic.lookup_tile(enter_x, thing.y.get_int()) {
                    MapTile::Walkable(_, _) if !things.blockmap.is_occupied(enter_x, thing.y.get_int()) => {
                        return Some(PathAction::Move {
                            dist: FP16_ONE,
                            dx,
                            dy: 0,
                        })
                    }
                    _ => (), // fall through
                }
                match map_dynamic.lookup_tile(thing.x.get_int(), enter_y) {
                    MapTile::Walkable(_, _) if !things.blockmap.is_occupied(thing.x.get_int(), enter_y) => {
                        return Some(PathAction::Move {
                            dist: FP16_ONE,
                            dx: 0,
                            dy,
                        })
                    }
                    _ => (), // fall through
                }
            }
            println!("path hits wall head on. stopping.");
            None
        }

        _ => None,
    }
}

fn think_chase(thing: &mut Enemy, map_dynamic: &mut MapDynamic, things: &Things, static_index: usize) {
    let mut dodge = false;
    if check_player_sight(thing, things, map_dynamic, static_index) {
        let d = things
            .player_x
            .abs_diff(thing.x.get_int())
            .max(things.player_y.abs_diff(thing.y.get_int()));

        let chance = if d == 0 || (d == 1 && thing.path_action.as_ref().map_or(true, boost_shoot_chance)) {
            256
        } else {
            16 / d
        };
        println!("chance: {chance}");
        if (rand::random::<u8>() as u32) < chance {
            thing.set_state("shoot");
        }
        dodge = true;
    }

    if thing.path_action.is_none() {
        let dx = things.player_x - thing.x.get_int();
        let dy = things.player_y - thing.y.get_int();

        if dodge {
            //
            // arange 5 direction choices in order of preference
            // the four cardinal directions plus the diagonal straight towards
            // the player
            //
            let mut dirtry = [None; 5];
            if dx > 0 {
                dirtry[1] = Some(Direction::East);
                dirtry[3] = Some(Direction::West);
            } else {
                dirtry[1] = Some(Direction::West);
                dirtry[3] = Some(Direction::East);
            }

            if dy > 0 {
                dirtry[2] = Some(Direction::South);
                dirtry[4] = Some(Direction::North);
            } else {
                dirtry[2] = Some(Direction::North);
                dirtry[4] = Some(Direction::South);
            }

            dirtry[0] =
                match (dirtry[1], dirtry[2]) {
                    (Some(Direction::North), Some(Direction::East))
                    | (Some(Direction::East), Some(Direction::North)) => Some(Direction::NorthEast),
                    (Some(Direction::North), Some(Direction::West))
                    | (Some(Direction::West), Some(Direction::North)) => Some(Direction::NorthWest),
                    (Some(Direction::South), Some(Direction::East))
                    | (Some(Direction::East), Some(Direction::South)) => Some(Direction::SouthEast),
                    (Some(Direction::South), Some(Direction::West))
                    | (Some(Direction::West), Some(Direction::South)) => Some(Direction::SouthWest),
                    _ => None,
                };

            // FIXME: make find pathaction code more generic so we don't have to modity thing.direction just to test
            let old_dir = thing.direction;
            for dir in dirtry.iter().filter_map(|x| *x) {
                thing.direction = dir;
                thing.path_action = try_find_pathaction(thing, map_dynamic, things);
                if thing.path_action.is_some() {
                    break;
                }
            }
            if thing.path_action.is_none() {
                thing.direction = old_dir;
            }
        } else {
            let mut dirtry = [None; 3];
            if dx > 0 {
                dirtry[1] = Some(Direction::East);
            } else {
                dirtry[1] = Some(Direction::West);
            }

            if dy > 0 {
                dirtry[2] = Some(Direction::South);
            } else {
                dirtry[2] = Some(Direction::North);
            }

            dirtry[0] =
                match (dirtry[1], dirtry[2]) {
                    (Some(Direction::North), Some(Direction::East))
                    | (Some(Direction::East), Some(Direction::North)) => Some(Direction::NorthEast),
                    (Some(Direction::North), Some(Direction::West))
                    | (Some(Direction::West), Some(Direction::North)) => Some(Direction::NorthWest),
                    (Some(Direction::South), Some(Direction::East))
                    | (Some(Direction::East), Some(Direction::South)) => Some(Direction::SouthEast),
                    (Some(Direction::South), Some(Direction::West))
                    | (Some(Direction::West), Some(Direction::South)) => Some(Direction::SouthWest),
                    _ => None,
                };

            // FIXME: make find pathaction code more generic so we don't have to modity thing.direction just to test
            let old_dir = thing.direction;
            for dir in dirtry.iter().filter_map(|x| *x) {
                thing.direction = dir;
                thing.path_action = try_find_pathaction(thing, map_dynamic, things);
                if thing.path_action.is_some() {
                    break;
                }
            }
            if thing.path_action.is_none() {
                thing.direction = old_dir;
            } else {
                println!("chase path action: {:?}", thing.path_action);
            }
        }
    }
    move_default(thing, map_dynamic, static_index);
}

fn boost_shoot_chance(path_action: &PathAction) -> bool {
    match path_action {
        PathAction::Move { dist, dx: _, dy: _ } => *dist < FP16_FRAC_64 * 2,
        PathAction::WaitForDoor { door_id: _ } => false,
        PathAction::MoveThroughDoor { dist, door_id: _ } => *dist < FP16_FRAC_64 * 2,
    }
}

fn think_shoot(thing: &mut Enemy, map_dynamic: &mut MapDynamic, things: &Things, static_index: usize) {}

fn think_path(thing: &mut Enemy, map_dynamic: &mut MapDynamic, things: &Things, static_index: usize) {
    if check_player_sight(thing, things, map_dynamic, static_index) {
        thing.set_state("chase");
        return;
    }

    if thing.path_action.is_none() {
        thing.direction = try_update_pathdir(thing, map_dynamic).unwrap_or(thing.direction);
        thing.path_action = try_find_pathaction(thing, map_dynamic, things);
    }
    move_default(thing, map_dynamic, static_index);
}

fn think_stand(thing: &mut Enemy, map_dynamic: &mut MapDynamic, things: &Things, static_index: usize) {
    if check_player_sight(thing, things, map_dynamic, static_index) {
        thing.set_state("chase");
    }
}

fn move_default(thing: &mut Enemy, map_dynamic: &mut MapDynamic, static_index: usize) {
    match &mut thing.path_action {
        Some(PathAction::Move { dist, dx, dy }) if *dist > FP16_ZERO => {
            if *dist == FP16_ONE {
                // check if we would bump into door
            }

            // still some way to go on old action
            *dist -= crate::fp16::FP16_FRAC_128;
            thing.x += crate::fp16::FP16_FRAC_128 * *dx;
            thing.y += crate::fp16::FP16_FRAC_128 * *dy;
            if *dist == FP16_ZERO {
                thing.path_action = None;
            }
        }
        Some(PathAction::Move { dist: _, dx: _, dy: _ }) => {
            panic!("PathAction::Move with zero dist.");
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
                thing.path_action = Some(PathAction::Move {
                    dist: FP16_ONE,
                    dx: thing.direction.x_offs(),
                    dy: thing.direction.y_offs(),
                });
            }
        }
        None => {
            // println!("no PathAction.")
        }
    }
}

#[derive(Debug)]
pub enum PathAction {
    Move { dist: Fp16, dx: i32, dy: i32 },
    WaitForDoor { door_id: usize },
    MoveThroughDoor { dist: Fp16, door_id: usize },
}

impl ms::Loadable for PathAction {
    fn read_from(r: &mut dyn Read) -> Result<Self> {
        Ok(match r.read_u8()? {
            0 => PathAction::Move {
                dist: Fp16::read_from(r)?,
                dx: r.read_i32::<LittleEndian>()?,
                dy: r.read_i32::<LittleEndian>()?,
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
            PathAction::Move { dist, dx, dy } => {
                w.write_u8(0)?;
                dist.write(w)?;
                w.write_i32::<LittleEndian>(*dx)?;
                w.write_i32::<LittleEndian>(*dy)?;
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
            Think::Stand => think_stand(self, map_dynamic, things, static_index),
            Think::Path => think_path(self, map_dynamic, things, static_index),
            Think::Chase => think_chase(self, map_dynamic, things, static_index),
            Think::Shoot => think_shoot(self, map_dynamic, things, static_index),
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
            crate::thing_def::EnemyState::Patrolling => (
                "path",
                None,
                // Some(PathAction::Move {
                //     dist: FP16_ONE,
                //     dx: direction.x_offs(),
                //     dy: direction.y_offs(),
                // }),
            ),
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
