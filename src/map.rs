use crate::prelude::*;
use std::io::Write;

pub const MAP_SIZE: usize = 64;

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
    Walkable(i32, Option<Direction>),
    Blocked(i32),
    Door(PlaneOrientation, DoorType, usize),
    PushWall(i32, usize),
    OffsetWall(i32, Direction, Fp16),
}

pub struct Map {
    pub map: [[MapTile; MAP_SIZE]; MAP_SIZE],
}

pub const ROOM_ID_NONE: i32 = 0x6a;

impl Default for Map {
    fn default() -> Self {
        let mut map = [[MapTile::Walkable(0, None); MAP_SIZE]; MAP_SIZE];

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
        let mut map = [[MapTile::Walkable(0, None); MAP_SIZE]; MAP_SIZE];

        let mut dump_f = std::fs::File::create("map.txt").unwrap();
        for line in &mut map {
            for out in line.iter_mut() {
                let c = *plane_iter.next().unwrap();
                let p = *prop_plane_iter.next().unwrap();
                write!(dump_f, "{c:2x}:{p:2x} ").unwrap();
                // write!(dump_f, "{p:2x} ").unwrap();

                let pushwall = p == 98;

                match c {
                    1..=63 if !pushwall => *out = MapTile::Wall(((c - 1) * 2) as i32),
                    1..=63 if pushwall => *out = MapTile::PushWall(((c - 1) * 2) as i32, 0),
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
                    _ => *out = MapTile::Walkable(c as i32, Direction::try_from_prop_id(p as i32)),
                }
            }
            writeln!(dump_f).unwrap();
        }

        Self { map }
    }

    pub fn lookup_tile(&self, x: i32, y: i32) -> &MapTile {
        &self.map[y as usize][x as usize]
    }

    pub fn draw_automap<D: Draw + ?Sized>(&self, screen: &mut D) {
        let wall_color = |x: usize, y: usize| match self.map[y][x] {
            MapTile::Wall(wall) => Some(wall % 16),
            MapTile::Walkable(_, _) => None,
            MapTile::Blocked(_) => None,
            MapTile::Door(_, _, _) => None,
            MapTile::PushWall(_, _) => None,
            MapTile::OffsetWall(_, _, _) => None,
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

    pub fn get_room_connectivity(&self) -> Vec<(i32, i32, i32)> {
        let mut connections = Vec::new();
        for y in 1..(MAP_SIZE as i32 - 1) {
            for x in 1..(MAP_SIZE as i32 - 1) {
                for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let nx = x + dx;
                    let ny = y + dy;

                    match (self.lookup_tile(x, y), self.lookup_tile(nx, ny)) {
                        (MapTile::Walkable(id1, _), MapTile::Walkable(id2, _))
                            if id1 != id2 && *id1 != ROOM_ID_NONE && *id2 != ROOM_ID_NONE =>
                        {
                            println!("rooms touching at {x} {y}: {id1:x} {id2:x}");
                        }
                        _ => (),
                    }
                }

                for (dx1, dx2, dy1, dy2) in [(1, -1, 0, 0), (0, 0, 1, -1)] {
                    let nx1 = x + dx1;
                    let ny1 = y + dy1;

                    let nx2 = x + dx2;
                    let ny2 = y + dy2;

                    match (
                        self.lookup_tile(nx1, ny1),
                        self.lookup_tile(x, y),
                        self.lookup_tile(nx2, ny2),
                    ) {
                        (
                            MapTile::Walkable(id1, _),
                            MapTile::Door(_, _, door_id),
                            MapTile::Walkable(id2, _),
                        ) if id1 != id2 && *id1 != ROOM_ID_NONE && *id2 != ROOM_ID_NONE => {
                            // println!("door {door_id} at {x} {y}, connecting {id1:x} {id2:x}");
                            connections.push((*id1, *id2, *door_id as i32));
                            connections.push((*id2, *id1, *door_id as i32));
                        }
                        _ => (),
                    }
                }
            }
        }
        connections
    }
}

pub fn bresenham_trace<F: Fn(i32, i32) -> bool>(
    mut x0: i32,
    mut y0: i32,
    x1: i32,
    y1: i32,
    f: F,
) -> bool {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut error = dx + dy;
    loop {
        if !f(x0, y0) {
            return false;
        }
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
    true
}
