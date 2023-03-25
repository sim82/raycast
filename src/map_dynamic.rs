use crate::{fp16::FP16_FRAC_64, ms::Loadable, prelude::*};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::{
    collections::{HashMap, HashSet},
    io::Write,
};

pub struct MapDynamic {
    pub map: Map,
    pub door_states: Vec<DoorState>,
    pub pushwall_states: Vec<PushwallState>,
    pub pushwall_patch: HashMap<(i32, i32), MapTile>,
    pub tmp_door_triggers: HashSet<usize>, // not persistent. accumulated door triggers during thing update and applies them in same frame
}

impl MapDynamic {
    pub fn wrap(mut map: Map) -> MapDynamic {
        let mut door_states = Vec::new();
        let mut pushwall_states = Vec::new();

        for (y, line) in map.map.iter_mut().enumerate() {
            for (x, tile) in line.iter_mut().enumerate() {
                if let MapTile::Door(_, _, state_index) = tile {
                    *state_index = door_states.len();
                    door_states.push(Default::default());
                } else if let MapTile::PushWall(_, state_index) = tile {
                    *state_index = pushwall_states.len();
                    pushwall_states.push(PushwallState {
                        x: x as i32,
                        y: y as i32,
                        ..Default::default()
                    });
                }
            }
        }

        MapDynamic {
            map,
            door_states,
            pushwall_states,
            pushwall_patch: Default::default(),
            tmp_door_triggers: HashSet::new(),
        }
    }

    pub fn read_and_wrap(r: &mut dyn std::io::Read, mut map: Map) -> Result<MapDynamic> {
        let s = r.read_i32::<LittleEndian>()?;
        let mut door_states = Vec::new();
        for _ in 0..s {
            door_states.push(DoorState::read_from(r)?);
        }

        let s = r.read_i32::<LittleEndian>()?;
        let mut pushwall_states = Vec::new();
        for _ in 0..s {
            pushwall_states.push(PushwallState::read_from(r)?);
        }

        let mut door_count = 0;
        let mut pushwall_count = 0;
        for line in &mut map.map {
            for tile in line.iter_mut() {
                if let MapTile::Door(_, _, state_index) = tile {
                    *state_index = door_count;
                    door_count += 1;
                } else if let MapTile::PushWall(_, state_index) = tile {
                    *state_index = pushwall_count;
                    pushwall_count += 1;
                }
            }
        }

        assert_eq!(door_count, door_states.len());
        assert_eq!(pushwall_count, pushwall_states.len());

        Ok(MapDynamic {
            map,
            door_states,
            pushwall_states,
            pushwall_patch: Default::default(),
            tmp_door_triggers: HashSet::new(),
        })
    }

    pub fn release(mut self) -> Map {
        for line in &mut self.map.map {
            for tile in line.iter_mut() {
                if let MapTile::Door(_, _, state_index) = tile {
                    *state_index = 0;
                } else if let MapTile::PushWall(_, state_index) = tile {
                    *state_index = 0;
                }
            }
        }

        self.map
    }

    pub fn can_walk(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || (x as usize) >= MAP_SIZE || (y as usize) >= MAP_SIZE {
            return false; // solid outer
        }

        let tile = if let Some(tile) = self.pushwall_patch.get(&(x, y)) {
            tile
        } else {
            self.map.lookup_tile(x, y)
        };

        match tile {
            MapTile::Walkable(_, _) => true,
            MapTile::Door(_, _, state_index) => {
                matches!(self.door_states[*state_index].action, DoorAction::Open(_))
            }
            MapTile::PushWall(_, state_index) => {
                !matches!(self.pushwall_states[*state_index].action, PushwallAction::Closed)
            }
            _ => false,
        }
    }

    pub fn lookup_tile(&self, x: i32, y: i32) -> MapTile {
        if x < 0 || y < 0 || (x as usize) >= MAP_SIZE || (y as usize) >= MAP_SIZE {
            return MapTile::Wall(1); // solid outer
        }

        let tile = if let Some(tile) = self.pushwall_patch.get(&(x, y)) {
            tile
        } else {
            self.map.lookup_tile(x, y)
        };

        if let MapTile::PushWall(id, state_index) = tile {
            match self.pushwall_states[*state_index].action {
                PushwallAction::Closed => MapTile::Wall(*id),
                _ => MapTile::Walkable(0, None),
            }
        } else {
            *tile
        }
    }

    pub fn update(&mut self, player: &Player) {
        // use (and consume) door triggers accumulated since last update
        let mut trigger_doors = std::mem::take(&mut self.tmp_door_triggers);
        let mut trigger_pushwalls = HashMap::new();
        if player.trigger {
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                match self.map.lookup_tile(player.x.get_int() + dx, player.y.get_int() + dy) {
                    MapTile::Door(_, _, state_index) => {
                        trigger_doors.insert(*state_index);
                    }
                    MapTile::PushWall(_, state_index) => {
                        // FIXME: quick-n-dirty calc push direction
                        let push_direction = if dx > 0 {
                            Direction::East
                        } else if dx < 0 {
                            Direction::West
                        } else if dy > 0 {
                            Direction::South
                        } else {
                            Direction::North
                        };

                        trigger_pushwalls.insert(state_index, push_direction);
                    }
                    _ => (),
                }
            }
        }
        let mut blocked_doors = HashSet::new();
        let (tx, ty) = player.get_corners();

        for i in 0..4 {
            if let MapTile::Door(_, _, state_index) = self.map.lookup_tile(tx[i].get_int(), ty[i].get_int()) {
                blocked_doors.insert(state_index);
            }
        }

        for (i, door_state) in self.door_states.iter_mut().enumerate() {
            door_state.update(
                trigger_doors.contains(&i),
                blocked_doors.contains(&i) || !door_state.blockers.is_empty(),
            );
        }

        for (i, pushwall_state) in self.pushwall_states.iter_mut().enumerate() {
            let trigger = trigger_pushwalls.get(&i);
            pushwall_state.update(trigger.cloned())
        }

        // update active / finished pushwall 'patching' over static map data
        self.pushwall_patch.clear();
        for pushwall_state in &self.pushwall_states {
            match pushwall_state.action {
                PushwallAction::Sliding(direction, f) => {
                    let (dx, dy) = direction.tile_offset();
                    let fi = f.get_int();
                    if let MapTile::PushWall(id, _) = self.map.map[pushwall_state.y as usize][pushwall_state.x as usize]
                    {
                        self.pushwall_patch.insert(
                            ((fi * dx) + pushwall_state.x, (fi * dy) + pushwall_state.y),
                            MapTile::OffsetWall(id, direction, f.fract()),
                        );
                    }
                }
                PushwallAction::Open(xoffs, yoffs) => {
                    if let MapTile::PushWall(id, _) = self.map.map[pushwall_state.y as usize][pushwall_state.x as usize]
                    {
                        self.pushwall_patch
                            .insert((pushwall_state.x + xoffs, pushwall_state.y + yoffs), MapTile::Wall(id));
                    }
                }

                _ => (),
            }
        }
    }

    pub fn try_open_and_block_door(&mut self, door_id: usize, blocker: i32) -> bool {
        let door_state = &mut self.door_states[door_id];
        match door_state.action {
            DoorAction::Closed => {
                self.tmp_door_triggers.insert(door_id);
                false
            }
            // only use open door if it is not about to close (is this necessary?)
            DoorAction::Open(timeout) if timeout > 30 => {
                door_state.blockers.insert(blocker);
                true
            }
            _ => false,
        }

        // return false;
    }
    pub fn unblock_door(&mut self, door_id: usize, blocker: i32) {
        self.door_states[door_id].blockers.remove(&blocker);
    }
}

impl ms::Writable for MapDynamic {
    fn write(&self, w: &mut dyn std::io::Write) -> Result<()> {
        w.write_i32::<LittleEndian>(self.door_states.len() as i32)?;
        for state in &self.door_states {
            state.write(w)?;
        }

        w.write_i32::<LittleEndian>(self.pushwall_states.len() as i32)?;
        for state in &self.pushwall_states {
            state.write(w)?;
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Default)]
pub enum DoorAction {
    #[default]
    Closed,
    Opening,
    Open(i32),
    Closing,
}

impl ms::Writable for DoorAction {
    fn write(&self, w: &mut dyn std::io::Write) -> Result<()> {
        match self {
            DoorAction::Closed => w.write_u8(0)?,
            DoorAction::Opening => w.write_u8(1)?,
            DoorAction::Open(ticks) => {
                w.write_u8(2)?;
                w.write_i32::<LittleEndian>(*ticks)?;
            }
            DoorAction::Closing => w.write_u8(3)?,
        }
        Ok(())
    }
}

impl Loadable for DoorAction {
    fn read_from(r: &mut dyn std::io::Read) -> Result<Self> {
        let d = r.read_u8()?;
        Ok(match d {
            0 => DoorAction::Closed,
            1 => DoorAction::Opening,
            2 => {
                let ticks = r.read_i32::<LittleEndian>()?;
                DoorAction::Open(ticks)
            }
            3 => DoorAction::Closing,
            _ => return Err(anyhow::anyhow!("unhandled enum discriminator {}", d)),
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct DoorState {
    pub open_f: Fp16,
    pub action: DoorAction,
    pub blockers: HashSet<i32>,
}

impl ms::Writable for DoorState {
    fn write(&self, w: &mut dyn std::io::Write) -> Result<()> {
        w.write_i32::<LittleEndian>(self.open_f.v)?;
        self.action.write(w)?;
        w.write_u32::<LittleEndian>(self.blockers.len() as u32)?;
        for blocker in &self.blockers {
            w.write_i32::<LittleEndian>(*blocker)?;
        }
        Ok(())
    }
}

impl ms::Loadable for DoorState {
    fn read_from(r: &mut dyn std::io::Read) -> Result<Self> {
        let open = r.read_i32::<LittleEndian>()?;
        let action = DoorAction::read_from(r)?;
        let num_blockers = r.read_u32::<LittleEndian>()?;
        let mut blockers = HashSet::new();
        for _ in 0..num_blockers {
            blockers.insert(r.read_i32::<LittleEndian>()?);
        }

        Ok(Self {
            open_f: Fp16 { v: open },
            action,
            blockers,
        })
    }
}

impl DoorState {
    pub fn update(&mut self, trigger: bool, blocked: bool) {
        match &mut self.action {
            DoorAction::Closed if trigger => self.action = DoorAction::Opening,
            DoorAction::Closed => (),
            DoorAction::Opening if self.open_f < FP16_ONE => {
                self.open_f += FP16_FRAC_64;
                assert!(self.open_f <= FP16_ONE);
            }
            DoorAction::Opening => self.action = DoorAction::Open(60 * 4),
            DoorAction::Open(ticks_left) => {
                if (trigger || *ticks_left <= 0) && !blocked {
                    self.action = DoorAction::Closing;
                } else {
                    *ticks_left -= 1;
                }
            }
            DoorAction::Closing if self.open_f > FP16_ZERO => {
                self.open_f -= FP16_FRAC_64;
                assert!(self.open_f >= FP16_ZERO);
            }
            DoorAction::Closing => self.action = DoorAction::Closed,
        }
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub enum PushwallAction {
    #[default]
    Closed,
    Sliding(Direction, Fp16),
    Open(i32, i32),
}

#[derive(Debug, Clone, Default)]
pub struct PushwallState {
    pub x: i32, // FIXME: storing the pos is a q-n-d kludge
    pub y: i32,
    pub action: PushwallAction,
}
impl PushwallState {
    pub fn update(&mut self, trigger_direction: Option<Direction>) {
        match (&mut self.action, trigger_direction) {
            (PushwallAction::Closed, Some(direction)) => self.action = PushwallAction::Sliding(direction, FP16_ZERO),
            (PushwallAction::Sliding(_, f), _) if *f < (FP16_ONE * 2) => {
                *f += FP16_FRAC_64;
            }
            (PushwallAction::Sliding(direction, f), _) => {
                let (dx, dy) = direction.tile_offset();
                self.action = PushwallAction::Open(dx * f.get_int(), dy * f.get_int());
            }
            _ => (),
        }
    }
}

impl ms::Writable for PushwallState {
    fn write(&self, w: &mut dyn Write) -> Result<()> {
        w.write_i32::<LittleEndian>(self.x)?;
        w.write_i32::<LittleEndian>(self.y)?;
        match self.action {
            PushwallAction::Closed => w.write_u8(0)?,
            PushwallAction::Sliding(direction, f) => {
                w.write_u8(1)?;
                direction.write(w)?;
                f.write(w)?;
            }
            PushwallAction::Open(x, y) => {
                w.write_u8(2)?;
                w.write_i32::<LittleEndian>(x)?;
                w.write_i32::<LittleEndian>(y)?;
            }
        }
        Ok(())
    }
}

impl ms::Loadable for PushwallState {
    fn read_from(r: &mut dyn std::io::Read) -> Result<Self> {
        let x = r.read_i32::<LittleEndian>()?;
        let y = r.read_i32::<LittleEndian>()?;
        let action = match r.read_u8()? {
            0 => PushwallAction::Closed,
            1 => {
                let direction = Direction::read_from(r)?;
                let f = Fp16::read_from(r)?;
                PushwallAction::Sliding(direction, f)
            }
            2 => {
                let x = r.read_i32::<LittleEndian>()?;
                let y = r.read_i32::<LittleEndian>()?;
                PushwallAction::Open(x, y)
            }
            _ => panic!(),
        };
        Ok(Self { x, y, action })
    }
}

pub fn bresenham(mut x0: i32, mut y0: i32, x1: i32, y1: i32) -> Vec<(i32, i32)> {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut error = dx + dy;
    let mut out = Vec::new();
    loop {
        //   plot(x0, y0)
        out.push((x0, y0));
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * error;
        if e2 >= dy {
            if x0 == x1 {
                break;
            }
            error += dy;
            x0 += sx;
        }
        if e2 <= dx {
            if y0 == y1 {
                break;
            }
            error += dx;
            y0 += sy;
        }
    }
    out
}
