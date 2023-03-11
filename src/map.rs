use std::{collections::HashSet, ops::Range};

use crate::{fp16::FP16_FRAC_64, prelude::*};

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

#[derive(Debug, Clone, Default)]
pub struct DoorState {
    pub open_f: Fp16,
    pub action: DoorAction,
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

        Self {
            map,
            door_states: Vec::new(),
        }
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
        let mut door_states = Vec::new();
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
                // a bit crappy but simple: touch doors again to link state_index
                if let MapTile::Door(_, _, state_index) = out {
                    *state_index = door_states.len();
                    door_states.push(Default::default());
                }
            }
        }

        Self { map, door_states }
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
    pub fn can_walk(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || (x as usize) >= MAP_SIZE || (y as usize) >= MAP_SIZE {
            return false; // solid outer
        }
        match self.map[y as usize][x as usize] {
            MapTile::Walkable(_) => true,
            MapTile::Door(_, _, state_index) => {
                matches!(self.door_states[state_index].action, DoorAction::Open(_))
            }
            _ => false,
        }
    }

    pub fn sweep_raycast(
        &self,
        screen: &mut Vec<u32>,
        zbuffer: &mut [Fp16; WIDTH],
        player: &Player,
        columns: Range<usize>,
        resources: &Resources,
        frame: i32,
    ) {
        let (x, y) = player.int_pos();

        if !self.can_walk(x, y) {
            return;
        }
        let (dx, dy) = player.frac_pos();
        let (ex, ey) = (FP16_ONE - dx, FP16_ONE - dy);
        for column in columns {
            let alpha_full = player.rot + COL_ANGLE[column];
            let alpha_full = if alpha_full < 0 {
                alpha_full + FA_TAU
            } else if alpha_full >= FA_TAU {
                alpha_full - FA_TAU
            } else {
                alpha_full
            };

            let hstep_x;
            let hstep_y;
            let tx;
            let ty;

            let mut nx;
            let mut ny;

            if QUADRANT_1.contains(&alpha_full) {
                let alpha = alpha_full - QUADRANT_1.start;
                hstep_x = 1;
                hstep_y = 1;
                tx = fa_cot(alpha);
                ty = fa_tan(alpha);
                nx = player.x + (tx * ey);
                ny = player.y + (ty * ex);
            } else if QUADRANT_2.contains(&alpha_full) {
                let alpha = alpha_full - QUADRANT_2.start;
                hstep_x = -1;
                hstep_y = 1;
                tx = -fa_tan(alpha);
                ty = fa_cot(alpha);
                nx = player.x + (tx * ey);
                ny = player.y + (ty * dx); // - 1.into();
            } else if QUADRANT_3.contains(&alpha_full) {
                let alpha = alpha_full - QUADRANT_3.start;
                hstep_x = -1;
                hstep_y = -1;
                tx = -fa_cot(alpha);
                ty = -fa_tan(alpha);
                nx = player.x + (tx * dy);
                ny = player.y + (ty * dx);
            } else if QUADRANT_4.contains(&alpha_full) {
                let alpha = alpha_full - QUADRANT_4.start;
                hstep_x = 1;
                hstep_y = -1;
                tx = fa_tan(alpha);
                ty = -fa_cot(alpha);
                nx = player.x + (tx * dy);
                ny = player.y + (ty * ex);
            } else {
                continue;
            };

            // TODO: can't we just shift hx/nx (or hy/ny) both by -1 to get rid of lookup correction?
            // hx/hy is the x/y boundary of the next block we enter. Depending on the hstep direction this is either the left or right (upper or lower)
            // boundary.
            let mut hx = x + hstep_x.max(0);
            let mut hy = y + hstep_y.max(0);

            let hit_tile;
            let dx;
            let dy;
            let tex_u;
            let tyh = ty * FP16_HALF;
            let txh = tx * FP16_HALF;
            let hstep_x_half = FP16_HALF * hstep_x;
            let hstep_y_half = FP16_HALF * hstep_y;

            // from_door tracks if the *last* tile we traced through was a door, to easily fix up textures on door sidewalls
            let mut from_door = matches!(self.lookup_tile(x, y), MapTile::Door(_, _, _));

            'outer: loop {
                if (hstep_y > 0 && ny <= hy.into()) || (hstep_y < 0 && ny >= hy.into()) {
                    // when hstep_x is negative, hx needs to be corrected by -1 (enter block from right / below). Inverse of the correction during hx initialization.
                    let lookup_tile = self.lookup_tile(hx + hstep_x.min(0), ny.get_int());

                    if let MapTile::Wall(tile) = lookup_tile {
                        screen.point_world(hx.into(), ny, (tile) % 16);

                        if from_door {
                            hit_tile = 101;
                        } else {
                            hit_tile = tile + 1
                        }

                        dx = Fp16::from(hx) - player.x;
                        dy = ny - player.y;
                        tex_u = if hstep_x > 0 {
                            (ny.get_fract() >> 10) as i32
                        } else {
                            63 - (ny.get_fract() >> 10) as i32
                        };
                        break 'outer;
                    } else if let MapTile::Door(PlaneOrientation::X, door_type, state_index) =
                        lookup_tile
                    {
                        let door_state = &self.door_states[state_index];
                        let door_hit = ny + tyh;
                        let door_hit_f = door_hit.fract() - door_state.open_f;
                        if door_hit_f > FP16_ZERO
                            && ((hstep_y > 0 && door_hit <= hy.into())
                                || (hstep_y < 0 && door_hit >= hy.into()))
                        {
                            hit_tile = door_type.get_texture_id() + 1;
                            dx = (Fp16::from(hx) + hstep_x_half) - player.x;
                            dy = door_hit - player.y;
                            tex_u = ((door_hit_f).get_fract() >> 10) as i32;
                            break 'outer;
                        }
                        from_door = true;
                    } else {
                        from_door = false;
                    }

                    hx += hstep_x;
                    ny += ty;
                } else {
                    // when hstep_y is negative, hy needs to be corrected by -1 (enter block from right / below). Inverse of the correction during hx initialization.
                    let lookup_tile = self.lookup_tile(nx.get_int(), hy + hstep_y.min(0));
                    if let MapTile::Wall(tile) = lookup_tile {
                        if from_door {
                            hit_tile = 100;
                        } else {
                            hit_tile = tile;
                        }

                        screen.point_world(nx, hy.into(), (tile) % 16);
                        dx = nx - player.x;
                        dy = Fp16::from(hy) - player.y;
                        tex_u = if hstep_y < 0 {
                            (nx.get_fract() >> 10) as i32
                        } else {
                            63 - (nx.get_fract() >> 10) as i32
                        };

                        break 'outer;
                    } else if let MapTile::Door(PlaneOrientation::Y, door_type, state_index) =
                        lookup_tile
                    {
                        let door_state = &self.door_states[state_index];
                        let door_hit = nx + txh;
                        let door_hit_f = door_hit.fract() - door_state.open_f;
                        if door_hit_f > FP16_ZERO
                            && ((hstep_x > 0 && door_hit <= hx.into())
                                || (hstep_x < 0 && door_hit >= hx.into()))
                        {
                            hit_tile = door_type.get_texture_id();
                            dx = door_hit - player.x;
                            dy = (Fp16::from(hy) + hstep_y_half) - player.y;
                            tex_u = (door_hit_f.get_fract() >> 10) as i32;
                            break 'outer;
                        }
                        from_door = true;
                    } else {
                        from_door = false;
                    }
                    hy += hstep_y;
                    nx += tx;
                }
            }

            const C: i32 = MID;
            let beta = player.rot;
            let p = fa_cos(beta) * dx + fa_sin(beta) * dy;
            let offs = if p > FP16_ZERO {
                (C << FP16_SCALE) / p.v
            } else {
                C
            };
            zbuffer[column] = p;
            let line_range = (MID - offs)..(MID + offs);

            // TODO: pre-compute LUTs for the non clipped cases?
            const TEXEL_SCALE: i32 = 1; // NOTE: this currently influences the performance of the bresenham loop
            let (tex_clip, line_range_clamped) = if line_range.start < 0 {
                const HALF_TEX: i32 = 32 << TEXEL_SCALE;
                let h = HALF_HEIGHT - line_range.start; // overall (half) height of drawn column (partially offscreen)
                let x = (HALF_TEX * HALF_HEIGHT) / h; // (half) number of texture pixels inside visible range (derived via HALF_HEIGHT / h == x / HALF_TEX)
                let tex_clip = HALF_TEX - x;
                (tex_clip, 0..VIEW_HEIGHT)
            } else {
                (0, line_range)
            };
            // do not include 'endpoints' in bresenham. -1 on d_screen and d_tex to account for this.
            let d_screen = line_range_clamped.end - line_range_clamped.start - 1;
            let d_tex = (64 << TEXEL_SCALE) - 2 * tex_clip - 1;
            let mut d = 2 * d_tex - d_screen;
            let mut row_tex = 0;
            let tex_col = &resources.get_texture(hit_tile)[(tex_u) as usize];

            for row_screen in line_range_clamped {
                screen.point_rgb(
                    column as i32,
                    row_screen,
                    tex_col[(row_tex + tex_clip) as usize >> TEXEL_SCALE],
                );
                while d > 0 {
                    row_tex += 1;
                    d -= 2 * d_screen;
                }
                d += 2 * d_tex
            }
            // screen.point(column as i32, MID + offs, 0);
            // screen.point(column as i32, MID - offs, 0);
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
    pub fn update(&mut self, player: &Player) {
        let mut trigger_doors = HashSet::new();

        if player.trigger {
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                if let MapTile::Door(_, _, state_index) =
                    self.lookup_tile(player.x.get_int() + dx, player.y.get_int() + dy)
                {
                    trigger_doors.insert(state_index);
                }
            }
        }
        let mut blocked_doors = HashSet::new();
        let (tx, ty) = player.get_corners();

        for i in 0..4 {
            if let MapTile::Door(_, _, state_index) =
                self.lookup_tile(tx[i].get_int(), ty[i].get_int())
            {
                blocked_doors.insert(state_index);
            }
        }

        for (i, door_state) in self.door_states.iter_mut().enumerate() {
            door_state.update(trigger_doors.contains(&i), blocked_doors.contains(&i));
        }
    }
}

#[test]
fn raycast_test() {
    let map = Map::default();
    let resources = Resources::default();
    let mut screen = vec![0; WIDTH * HEIGHT];
    let mut zbuffer = [Fp16::default(); WIDTH];
    let player = Player {
        x: Fp16 { v: 72090 },
        y: Fp16 { v: 72090 },
        rot: 0,
        trigger: false,
    };
    let col = 10;
    map.sweep_raycast(
        &mut screen,
        &mut zbuffer,
        &player,
        col..(col + 1),
        &resources,
        0,
    );
}
