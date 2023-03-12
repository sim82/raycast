use std::{collections::HashSet, ops::Range};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::{fp16::FP16_FRAC_64, ms::Loadable, prelude::*};

const MAP_SIZE: usize = 64;

const MAP: [&[u8]; MAP_SIZE] = include!("maps/map2.txt");
// const MAP: [&[u8]; MAP_SIZE] = include!("maps/map_empty.txt");

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PlaneOrientation {
    X,
    Y,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DoorType {
    Regular,
    Elevator,
    GoldLocked,
    SilverLocked,
}

impl DoorType {
    pub fn get_texture_id(&self) -> i32 {
        match self {
            DoorType::Regular => 98,
            DoorType::Elevator => 102,
            DoorType::GoldLocked => 104,
            DoorType::SilverLocked => 104,
        }
    }
}

// TODO: model this better
#[derive(Debug, Clone, Copy)]
pub enum MapTile {
    Wall(i32),
    Walkable(i32),
    Blocked(i32),
    Door(PlaneOrientation, DoorType, usize),
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
    fn write(&self, w: &mut dyn std::io::Write) {
        match self {
            DoorAction::Closed => w.write_u8(0).unwrap(),
            DoorAction::Opening => w.write_u8(1).unwrap(),
            DoorAction::Open(ticks) => {
                w.write_u8(2).unwrap();
                w.write_i32::<LittleEndian>(*ticks).unwrap();
            }
            DoorAction::Closing => w.write_u8(3).unwrap(),
        }
    }
}

impl Loadable for DoorAction {
    fn read_from(r: &mut dyn std::io::Read) -> Self {
        let d = r.read_u8().unwrap();
        match d {
            0 => DoorAction::Closed,
            1 => DoorAction::Opening,
            2 => {
                let ticks = r.read_i32::<LittleEndian>().unwrap();
                DoorAction::Open(ticks)
            }
            3 => DoorAction::Closing,
            _ => panic!(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DoorState {
    pub open_f: Fp16,
    pub action: DoorAction,
}

impl ms::Writable for DoorState {
    fn write(&self, w: &mut dyn std::io::Write) {
        w.write_i32::<LittleEndian>(self.open_f.v).unwrap();
        self.action.write(w);
    }
}

impl ms::Loadable for DoorState {
    fn read_from(r: &mut dyn std::io::Read) -> Self {
        let open = r.read_i32::<LittleEndian>().unwrap();
        let action = DoorAction::read_from(r);
        Self {
            open_f: Fp16 { v: open },
            action,
        }
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

pub struct Map {
    pub map: [[MapTile; MAP_SIZE]; MAP_SIZE],
}

pub struct MapDynamic {
    pub door_states: Vec<DoorState>,
}

impl Default for Map {
    fn default() -> Self {
        let mut map = [[MapTile::Walkable(0); MAP_SIZE]; MAP_SIZE];

        for (in_line, out_line) in MAP.iter().zip(map.iter_mut()) {
            for (c, out) in in_line.iter().zip(out_line.iter_mut()) {
                // FIXME: crappy harcoded
                if *c >= b'1' && *c <= b'9' {
                    let tile: i32 = (*c - b'0').into();
                    *out = MapTile::Wall(tile - 1 + 10);
                } else if *c == b'8' || *c == b':' {
                    // *out = (0, false);
                }
            }
        }

        Self { map }
    }
}

const BLOCKING_PROPS: [u16; 21] = [
    24, 25, 26, 28, 30, 31, 33, 34, 35, 36, 39, 40, 41, 45, 58, 59, 60, 62, 63, 68, 69,
];
impl Map {
    pub fn from_map_planes(plane: &[u16], prop_plane: &[u16]) -> Self {
        assert_eq!(plane.len(), 64 * 64);
        assert_eq!(prop_plane.len(), 64 * 64);
        let mut plane_iter = plane.iter();
        let mut prop_plane_iter = prop_plane.iter();
        let mut map = [[MapTile::Walkable(0); MAP_SIZE]; MAP_SIZE];
        for line in &mut map {
            for out in line.iter_mut() {
                let c = *plane_iter.next().unwrap();
                let p = *prop_plane_iter.next().unwrap();

                match c {
                    1..=63 => *out = MapTile::Wall(((c - 1) * 2) as i32),
                    90 => *out = MapTile::Door(PlaneOrientation::X, DoorType::Regular, 0),
                    91 => *out = MapTile::Door(PlaneOrientation::Y, DoorType::Regular, 0),
                    92 => *out = MapTile::Door(PlaneOrientation::X, DoorType::GoldLocked, 0),
                    93 => *out = MapTile::Door(PlaneOrientation::Y, DoorType::GoldLocked, 0),
                    94 => *out = MapTile::Door(PlaneOrientation::X, DoorType::SilverLocked, 0),
                    95 => *out = MapTile::Door(PlaneOrientation::Y, DoorType::SilverLocked, 0),
                    100 => *out = MapTile::Door(PlaneOrientation::X, DoorType::Elevator, 0),
                    101 => *out = MapTile::Door(PlaneOrientation::Y, DoorType::Elevator, 0),
                    _ if BLOCKING_PROPS.binary_search(&p).is_ok() => {
                        *out = MapTile::Blocked(p as i32)
                    }
                    _ => (),
                }
            }
        }

        Self { map }
    }
    pub fn split_dynamic(mut self) -> (Self, MapDynamic) {
        let mut door_states = Vec::new();

        for line in &mut self.map {
            for tile in line.iter_mut() {
                if let MapTile::Door(_, _, state_index) = tile {
                    *state_index = door_states.len();
                    door_states.push(Default::default());
                }
            }
        }

        (self, MapDynamic { door_states })
    }

    pub fn lookup_tile(&self, x: i32, y: i32) -> MapTile {
        if x < 0 || y < 0 || (x as usize) >= MAP_SIZE || (y as usize) >= MAP_SIZE {
            return MapTile::Wall(1); // solid outer
        }
        self.map[y as usize][x as usize]
    }

    pub fn lookup_wall(&self, x: i32, y: i32) -> Option<i32> {
        if x < 0 || y < 0 || (x as usize) >= MAP_SIZE || (y as usize) >= MAP_SIZE {
            return Some(1); // solid outer
        }
        match self.map[y as usize][x as usize] {
            MapTile::Wall(id) => Some(id),
            MapTile::Walkable(_) | MapTile::Blocked(_) => None,
            MapTile::Door(_, _, _) => None,
        }
    }
    pub fn can_walk(&self, map_dynamic: &MapDynamic, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || (x as usize) >= MAP_SIZE || (y as usize) >= MAP_SIZE {
            return false; // solid outer
        }
        match self.map[y as usize][x as usize] {
            MapTile::Walkable(_) => true,
            MapTile::Door(_, _, state_index) => {
                matches!(
                    map_dynamic.door_states[state_index].action,
                    DoorAction::Open(_)
                )
            }
            _ => false,
        }
    }

    pub fn draw_automap(&self, screen: &mut Vec<u32>) {
        let wall_color = |x: usize, y: usize| match self.map[y][x] {
            MapTile::Wall(wall) => Some(wall % 16),
            MapTile::Walkable(_) => None,
            MapTile::Blocked(_) => None,
            MapTile::Door(_, _, _) => None,
        };

        for y in 0..64 {
            for x in 0..64 {
                if (1..63).contains(&x)
                    && (1..63).contains(&y)
                    && wall_color(x - 1, y).is_some()
                    && wall_color(x + 1, y).is_some()
                    && wall_color(x, y - 1).is_some()
                    && wall_color(x, y + 1).is_some()
                {
                    continue;
                }

                if let Some(color) = wall_color(x, y) {
                    screen.point_world(
                        FP16_HALF + (x as i32).into(),
                        FP16_HALF + (y as i32).into(),
                        color,
                    )
                }
            }
        }
    }
}

impl MapDynamic {
    pub fn update(&mut self, map: &Map, player: &Player) {
        let mut trigger_doors = HashSet::new();

        if player.trigger {
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                if let MapTile::Door(_, _, state_index) =
                    map.lookup_tile(player.x.get_int() + dx, player.y.get_int() + dy)
                {
                    trigger_doors.insert(state_index);
                }
            }
        }
        let mut blocked_doors = HashSet::new();
        let (tx, ty) = player.get_corners();

        for i in 0..4 {
            if let MapTile::Door(_, _, state_index) =
                map.lookup_tile(tx[i].get_int(), ty[i].get_int())
            {
                blocked_doors.insert(state_index);
            }
        }

        for (i, door_state) in self.door_states.iter_mut().enumerate() {
            door_state.update(trigger_doors.contains(&i), blocked_doors.contains(&i));
        }
    }
}

impl ms::Writable for MapDynamic {
    fn write(&self, w: &mut dyn std::io::Write) {
        w.write_i32::<LittleEndian>(self.door_states.len() as i32)
            .unwrap();
        for state in &self.door_states {
            state.write(w);
        }
    }
}

impl ms::Loadable for MapDynamic {
    fn read_from(r: &mut dyn std::io::Read) -> Self {
        let s = r.read_i32::<LittleEndian>().unwrap();
        let mut door_states = Vec::new();
        for _ in 0..s {
            door_states.push(DoorState::read_from(r));
        }

        MapDynamic { door_states }
    }
}
