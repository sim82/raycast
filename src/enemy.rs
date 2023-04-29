use crate::{fp16::FP16_FRAC_64, prelude::*, thing_def::get_capabilities_by_name};
use anyhow::anyhow;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use rand::random;
use std::io::{Cursor, Read, Write};

impl Enemy {
    fn check_player_sight(
        &mut self,
        things: &Things,
        map_dynamic: &mut MapDynamic,
        _static_index: usize,
    ) -> bool {
        let dx = things.player_x - self.x.get_int();
        let dy = things.player_y - self.y.get_int();

        let in_front = match self.direction {
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
            self.x.get_int(),
            self.y.get_int(),
            things.player_x,
            things.player_y,
            |x, y| match map_dynamic.lookup_tile(x, y) {
                MapTile::Walkable(_, _) => true,
                MapTile::Door(_, _, door_id) => map_dynamic.door_states[door_id].open_f > FP16_HALF,
                _ => false,
            },
        )
    }

    fn try_update_pathdir(&self, map_dynamic: &mut MapDynamic) -> Option<Direction> {
        // check how to continue
        let xaligned = self.x.fract() == FP16_HALF;
        let yaligned = self.y.fract() == FP16_HALF;
        if !(xaligned && yaligned) {
            // TODO: recover, e.g. teleport to next tile center. This should not happen in a fixedpoint world, but who knows...
            println!("PathAction::Move ended not on tile center. Aborting.");
            return None;
        }
        if let MapTile::Walkable(_, Some(path_direction)) =
            map_dynamic.lookup_tile(self.x.get_int(), self.y.get_int())
        {
            Some(path_direction)
        } else {
            None
        }
    }

    fn try_find_pathaction(
        &self,
        map_dynamic: &mut MapDynamic,
        things: &Things,
    ) -> Option<PathAction> {
        let (dx, dy) = self.direction.tile_offset();
        // check the block we are about to enter
        let enter_x = self.x.get_int() + dx;
        let enter_y = self.y.get_int() + dy;
        match map_dynamic.lookup_tile(enter_x, enter_y) {
            MapTile::Door(_, _, door_id) if !self.direction.is_diagonal() => {
                // println!("open door");
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
                    Some(PathAction::Move {
                        dist: FP16_ONE,
                        dx,
                        dy,
                    })
                }
            }
            MapTile::Blocked(_) | MapTile::Wall(_) | MapTile::PushWall(_, _) => {
                // fixup path direction pointing diagonally into wall. The expectation seems to be that the actor continues going in the
                // diagonal direction but 'slide' along walls (e.g. the dogs in E1M6)
                if dx != 0 && dy != 0 {
                    match map_dynamic.lookup_tile(enter_x, self.y.get_int()) {
                        MapTile::Walkable(_, _)
                            if !things.blockmap.is_occupied(enter_x, self.y.get_int()) =>
                        {
                            return Some(PathAction::Move {
                                dist: FP16_ONE,
                                dx,
                                dy: 0,
                            })
                        }
                        _ => (), // fall through
                    }
                    match map_dynamic.lookup_tile(self.x.get_int(), enter_y) {
                        MapTile::Walkable(_, _)
                            if !things.blockmap.is_occupied(self.x.get_int(), enter_y) =>
                        {
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

    fn try_chase_pathaction(
        &self,
        direction: Direction,
        map_dynamic: &mut MapDynamic,
        things: &Things,
    ) -> Option<PathAction> {
        let (dx, dy) = direction.tile_offset();
        // check the block we are about to enter
        let enter_x = self.x.get_int() + dx;
        let enter_y = self.y.get_int() + dy;
        match map_dynamic.lookup_tile(enter_x, enter_y) {
            MapTile::Door(_, _, door_id) if !direction.is_diagonal() => {
                // println!("open door");
                Some(PathAction::WaitForDoor { door_id })
            }
            MapTile::Walkable(_, _) => {
                // on diagonal moves check if both adjecent tiles are at least walkable (it is ok if they are occupied so an enemy can move diagonally
                // between two others, but it can nerver move through a wall corner)
                let old_x = self.x.get_int();
                let old_y = self.y.get_int();
                let diagonal_blocked = direction.is_diagonal()
                    && (!map_dynamic.can_walk(old_x, enter_y)
                        || !map_dynamic.can_walk(enter_x, old_y)
                        || things.blockmap.is_occupied(old_x, enter_y)
                        || things.blockmap.is_occupied(enter_x, old_y));

                if diagonal_blocked || things.blockmap.is_occupied(enter_x, enter_y) {
                    None
                } else {
                    Some(PathAction::Move {
                        dist: FP16_ONE,
                        dx,
                        dy,
                    })
                }
            }
            MapTile::Blocked(_) | MapTile::Wall(_) | MapTile::PushWall(_, _) => None,

            _ => None,
        }
    }
}

// think implementations
impl Enemy {
    fn think_chase(&mut self, map_dynamic: &mut MapDynamic, things: &Things, unique_id: usize) {
        let mut dodge = false;
        if self.check_player_sight(things, map_dynamic, unique_id) {
            let d = things
                .player_x
                .abs_diff(self.x.get_int())
                .max(things.player_y.abs_diff(self.y.get_int()));

            let chance = if d == 0
                || (d == 1 && self.path_action.as_ref().map_or(true, boost_shoot_chance))
            {
                256
            } else {
                16 / d
            };
            // println!("chance: {chance}");
            if (rand::random::<u8>() as u32) < chance {
                self.set_state("shoot");
            }
            dodge = true;
        }

        if self.path_action.is_none() {
            let cont = if dodge {
                self.select_dodge_action(things, map_dynamic)
            } else {
                self.select_chase_action(things, map_dynamic)
            };

            if let Some((path_action, dir)) = cont {
                self.path_action = Some(path_action);
                self.direction = dir;
            }
        }
        self.move_default(map_dynamic, unique_id, FP16_FRAC_64);
    }
    fn think_path(&mut self, map_dynamic: &mut MapDynamic, things: &Things, unique_id: usize) {
        if self.notify || self.check_player_sight(things, map_dynamic, unique_id) {
            self.set_state("chase");
            self.notify = true;
            return;
        }

        if self.path_action.is_none() {
            self.direction = self
                .try_update_pathdir(map_dynamic)
                .unwrap_or(self.direction);
            self.path_action = self.try_find_pathaction(map_dynamic, things);
        }
        self.move_default(map_dynamic, unique_id, FP16_FRAC_128);
    }

    fn think_stand(&mut self, map_dynamic: &mut MapDynamic, things: &Things, unique_id: usize) {
        if self.notify || self.check_player_sight(things, map_dynamic, unique_id) {
            self.set_state("chase");
            self.notify = true;
        }
    }
    fn think_dogchase(&mut self, map_dynamic: &mut MapDynamic, things: &Things, unique_id: usize) {
        if self.path_action.is_none() {
            // absurd fact: dogs never dogdge
            if let Some((path_action, dir)) = self.select_chase_action(things, map_dynamic) {
                self.path_action = Some(path_action);
                self.direction = dir;
            }
        }

        let dx = self.x.get_int().abs_diff(things.player_x);
        let dy = self.y.get_int().abs_diff(things.player_y);

        if dx <= 1 && dy <= 1 {
            self.set_state("jump");
        }

        self.move_default(map_dynamic, unique_id, FP16_FRAC_64);
    }
}

// action implementations
impl Enemy {
    fn action_die(&mut self) {
        self.dead = true;
    }
    fn action_bite(&mut self, _map_dynamic: &mut MapDynamic, _things: &Things, _unique_id: usize) {}
    fn action_shoot(
        &mut self,
        map_dynamic: &mut MapDynamic,
        _things: &Things,
        _unique_id: usize,
        player: &mut Player,
    ) {
        println!("shoot");
        if !bresenham_trace(
            (self.x * 4).get_int(),
            (self.y * 4).get_int(),
            (player.x * 4).get_int(),
            (player.y * 4).get_int(),
            |x, y| match map_dynamic.lookup_tile(x / 4, y / 4) {
                MapTile::Walkable(_, _) => true,
                MapTile::Door(_, _, door_id) => map_dynamic.door_states[door_id].open_f > FP16_HALF,
                _ => false,
            },
        ) {
            return;
        }

        let dx = self.x.get_int() - player.x.get_int();
        let dy = self.y.get_int() - player.y.get_int();
        let dist = dx.max(dy);

        if random::<u8>() as i32 > dist * 20 {
            let boost = 2 - dx.max(dy).min(2);
            let base_hitpoints = 7;
            let hitpoints = base_hitpoints + ((boost * 5) * (random::<u8>() as i32)) / 255;
            player.health -= hitpoints;
        }
    }
}
impl Enemy {
    fn select_chase_action(
        &self,
        things: &Things,
        map_dynamic: &mut MapDynamic,
    ) -> Option<(PathAction, Direction)> {
        let dx = things.player_x - self.x.get_int();
        let dy = things.player_y - self.y.get_int();
        let mut dirtry = [None; 3];

        if (dx > 0) ^ (random::<u8>() < 16) {
            dirtry[1] = Some(Direction::East);
        } else {
            dirtry[1] = Some(Direction::West);
        }

        if (dy > 0) ^ (random::<u8>() < 16) {
            dirtry[2] = Some(Direction::South);
        } else {
            dirtry[2] = Some(Direction::North);
        }

        if (dy.abs() > dx.abs()) ^ (random::<u8>() < 32) {
            dirtry.swap(1, 2);
        }
        if random::<u8>() < 192 {
            dirtry[0] = match (dirtry[1], dirtry[2]) {
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
        }

        for dir in dirtry.iter().filter_map(|x| *x) {
            // println!("chase try: {dir:?}");
            let path_action = self.try_chase_pathaction(dir, map_dynamic, things);
            if let Some(path_action) = path_action {
                return Some((path_action, dir));
            }
        }
        None
    }
    fn select_dodge_action(
        &self,
        things: &Things,
        map_dynamic: &mut MapDynamic,
    ) -> Option<(PathAction, Direction)> {
        let dx = things.player_x - self.x.get_int();
        let dy = things.player_y - self.y.get_int();
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

        dirtry[0] = match (dirtry[1], dirtry[2]) {
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

        for dir in dirtry.iter().filter_map(|x| *x) {
            let path_action = self.try_chase_pathaction(dir, map_dynamic, things);
            if let Some(path_action) = path_action {
                return Some((path_action, dir));
            }
        }
        None
    }
}
fn boost_shoot_chance(path_action: &PathAction) -> bool {
    match path_action {
        PathAction::Move { dist, dx: _, dy: _ } => *dist < FP16_FRAC_64 * 2,
        PathAction::WaitForDoor { door_id: _ } => false,
        PathAction::MoveThroughDoor { dist, door_id: _ } => *dist < FP16_FRAC_64 * 2,
    }
}

// move implementations
impl Enemy {
    fn move_default(&mut self, map_dynamic: &mut MapDynamic, static_index: usize, speed: Fp16) {
        match &mut self.path_action {
            Some(PathAction::Move { dist, dx, dy }) if *dist > FP16_ZERO => {
                if *dist == FP16_ONE {
                    // check if we would bump into door
                }

                // still some way to go on old action
                *dist -= speed;
                self.x += speed * *dx;
                self.y += speed * *dy;

                if *dist < FP16_ZERO {
                    // mid-move speed changes result in non precise end position. Not sure how to solve this. Hard reset to tile center to keep everything aligned.
                    println!("negative dist. fixing");
                    *dist = FP16_ZERO;
                    self.x = FP16_HALF + self.x.get_int().into();
                    self.y = FP16_HALF + self.y.get_int().into();
                }

                if *dist == FP16_ZERO {
                    self.path_action = None;
                }
            }
            Some(PathAction::Move {
                dist: _,
                dx: _,
                dy: _,
            }) => {

                // panic!("PathAction::Move with zero dist.");
            }
            Some(PathAction::WaitForDoor { door_id }) => {
                if map_dynamic.try_open_and_block_door(*door_id, static_index as i32) {
                    self.path_action = Some(PathAction::MoveThroughDoor {
                        dist: FP16_ONE,
                        door_id: *door_id,
                    })
                } else if get_capabilities_by_name(&self.enemy_type_name).can_open_doors {
                    self.path_action = None;
                }
            }
            Some(PathAction::MoveThroughDoor { dist, door_id }) => {
                if *dist > FP16_ZERO {
                    *dist -= speed;
                    let (dx, dy) = self.direction.tile_offset();
                    self.x += speed * dx;
                    self.y += speed * dy;
                } else {
                    // unblock door and keep moving in same direction
                    map_dynamic.unblock_door(*door_id, static_index as i32);
                    self.path_action = Some(PathAction::Move {
                        dist: FP16_ONE,
                        dx: self.direction.x_offs(),
                        dy: self.direction.y_offs(),
                    });
                }
            }
            None => {
                // println!("no PathAction.")
            }
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
    // enemy_type: EnemyType,
    enemy_type_name: String,
    direction: Direction,
    path_action: Option<PathAction>,
    pub health: i32,
    pub x: Fp16,
    pub y: Fp16,
    pub notify: bool,
    pub dead: bool,
}

impl ms::Loadable for Enemy {
    fn read_from(r: &mut dyn Read) -> Result<Self> {
        let exec_ctx = ExecCtx::read_from(r, &IMG_WL6)?;
        // let enemy_type = EnemyType::read_from(r)?;
        let enemy_type_name = String::read_from(r)?;
        let direction = Direction::read_from(r)?;
        let path_action = Option::<PathAction>::read_from(r)?;
        let health = r.read_i32::<LittleEndian>()?;
        let x = Fp16::read_from(r)?;
        let y = Fp16::read_from(r)?;
        let notify = r.read_u8()? != 0;
        let dead = r.read_u8()? != 0;
        Ok(Enemy {
            exec_ctx,
            enemy_type_name,
            direction,
            path_action,
            health,
            x,
            y,
            notify,
            dead,
        })
    }
}

impl ms::Writable for Enemy {
    fn write(&self, w: &mut dyn Write) -> Result<()> {
        self.exec_ctx.write(w)?;
        // self.enemy_type.write(w)?;
        self.enemy_type_name.write(w)?;
        self.direction.write(w)?;
        self.path_action.write(w)?;
        w.write_i32::<LittleEndian>(self.health)?;
        self.x.write(w)?;
        self.y.write(w)?;
        w.write_u8(if self.notify { 1 } else { 0 })?;
        w.write_u8(if self.dead { 1 } else { 0 })?;
        Ok(())
    }
}

impl Enemy {
    pub fn set_state(&mut self, name: &str) {
        // let label = self.enemy_type.map_label(name);
        let label = format!("{}::{}", self.enemy_type_name, name);
        self.exec_ctx
            .jump_label(&label)
            .unwrap_or_else(|err| panic!("failed to jump to state {label}: {err:?}"));
        println!("state: {self:?}");
        self.dead = name == "dead";
    }
    pub fn dispatch_call(
        &mut self,
        function: Function,
        map_dynamic: &mut MapDynamic,
        things: &Things,
        unique_id: usize,
        player: &mut Player,
    ) {
        match function {
            Function::None => (),
            Function::ThinkStand => self.think_stand(map_dynamic, things, unique_id),
            Function::ThinkPath => self.think_path(map_dynamic, things, unique_id),
            Function::ThinkChase => self.think_chase(map_dynamic, things, unique_id),
            Function::ThinkDogChase => self.think_dogchase(map_dynamic, things, unique_id),
            Function::ActionDie => self.action_die(),
            Function::ActionShoot => self.action_shoot(map_dynamic, things, unique_id, player),
            Function::ActionBite => self.action_bite(map_dynamic, things, unique_id),
        }
    }
    fn exec_code(
        &mut self,
        code_offs: i32,
        map_dynamic: &mut MapDynamic,
        things: &Things,
        unique_id: usize,
        player: &mut Player,
    ) {
        let mut env = opcode::Env::default();
        let mut cursor = Cursor::new(&self.exec_ctx.image.code[code_offs as usize..]);
        loop {
            let state = opcode::exec(&mut cursor, &mut env).expect("error on bytecode exec");
            match state {
                opcode::Event::Stop => break,
                opcode::Event::Call(function) => {
                    self.dispatch_call(function, map_dynamic, things, unique_id, player)
                }
            }
        }
    }
    pub fn update(
        &mut self,
        map_dynamic: &mut MapDynamic,
        things: &Things,
        unique_id: usize,
        player: &mut Player,
    ) {
        // NOTE: actions are meant to be executed exactly once per state enter (i.e. 'take_action_offs' resets state.action_offs to -1)
        // this is different from wolf3d where actions execute on state exit (don't understand why...)
        if let Some(action_offs) = self.exec_ctx.state.take_action_offs() {
            self.exec_code(action_offs, map_dynamic, things, unique_id, player);
        }

        if self.exec_ctx.state.ticks <= 0 {
            self.exec_ctx.jump(self.exec_ctx.state.next).unwrap();
        }

        self.exec_code(
            self.exec_ctx.state.think_offs,
            map_dynamic,
            things,
            unique_id,
            player,
        );

        // self.states[self.cur].2();

        self.exec_ctx.state.ticks -= 1;
    }

    pub fn hit(&mut self, hitpoints: i32) {
        self.health -= hitpoints;

        if self.health <= 0 {
            self.set_state("die");
        } else if self.health % 2 == 0 {
            self.set_state("pain1");
        } else {
            self.set_state("pain2");
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

    pub fn spawn(enemy_spawn_info: &EnemySpawnInfo, thing_def: &ThingDef) -> Enemy {
        let exec_ctx = ExecCtx::new(&enemy_spawn_info.state, &IMG_WL6).unwrap();

        // FIXME: hack. find better solution for 'enemy type name'
        let enemy_type_name = enemy_spawn_info
            .state
            .split("::")
            .next()
            .unwrap()
            .to_string();

        Enemy {
            direction: enemy_spawn_info.direction,
            path_action: None,
            exec_ctx,
            // enemy_type,
            enemy_type_name,
            health: 25,
            x: thing_def.x,
            y: thing_def.y,
            notify: false,
            dead: false,
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
