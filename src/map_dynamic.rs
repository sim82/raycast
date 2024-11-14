use crate::{fp16::FP16_FRAC_64, map::ROOM_ID_NONE, ms::Loadable, prelude::*};
use std::{
    collections::{HashMap, HashSet},
    io::Write,
};

pub struct RoomGraph {
    adj: Vec<(i32, i32, i32)>,
}

impl RoomGraph {
    pub fn new(mut adj: Vec<(i32, i32, i32)>) -> RoomGraph {
        adj.sort_by_key(|(a, _, _)| *a);
        RoomGraph { adj }
    }

    pub fn propagate_notifications(
        &self,
        door_states: &[Door],
        notifications: &HashSet<i32>,
    ) -> HashSet<i32> {
        let mut stack = notifications.iter().collect::<Vec<_>>();
        let mut out = notifications.iter().cloned().collect::<HashSet<_>>();
        while let Some(room_id) = stack.pop() {
            let adj_start = self.adj.partition_point(|(src, _, _)| *src < *room_id);
            // println!("adj: {:x}: {:x?}", room_id, &self.adj[adj_start..]);
            for (src_id, dest_id, door_id) in &self.adj[adj_start..] {
                if *src_id != *room_id {
                    break;
                }
                if door_states[*door_id as usize].open_f > FP16_HALF && out.insert(*dest_id) {
                    stack.push(dest_id);
                }
            }
        }
        out
    }
}

pub struct Map {
    pub map: MapDef,
    pub room_graph: RoomGraph,
    pub door_states: Vec<Door>,
    pub pushwall_states: Vec<PushwallState>,
    pub pushwall_patch: HashMap<(i32, i32), MapTile>,

    // not persistent. accumulated door triggers during thing update and applies them in same frame
    pub tmp_door_triggers: HashSet<usize>,

    // notifications are persistent across frame bounds: they are expected to be valid at the start of a frame (i.e. need to be loaded/saved).
    // they will be overwritten during the frame to be used as input on the next frame
    pub notifications: HashSet<i32>,
}

impl Map {
    pub fn wrap(mut map: MapDef) -> Map {
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
        Map {
            room_graph: RoomGraph::new(map.get_room_connectivity()),
            map,
            door_states,
            pushwall_states,
            pushwall_patch: Default::default(),
            tmp_door_triggers: HashSet::new(),
            notifications: HashSet::new(),
        }
    }

    pub fn read_and_wrap(r: &mut dyn std::io::Read, mut map: MapDef) -> Result<Map> {
        let door_states = Vec::read_from(r)?;
        let pushwall_states = Vec::read_from(r)?;

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

        let mut notifications = HashSet::new();
        let notification_count = r.readi32()?;
        for _ in 0..notification_count {
            notifications.insert(r.readi32()?);
        }

        assert_eq!(door_count, door_states.len());
        assert_eq!(pushwall_count, pushwall_states.len());

        Ok(Map {
            room_graph: RoomGraph::new(map.get_room_connectivity()),
            map,
            door_states,
            pushwall_states,
            pushwall_patch: Default::default(),
            tmp_door_triggers: HashSet::new(),
            notifications,
        })
    }

    pub fn release(mut self) -> MapDef {
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
                // matches!(self.door_states[*state_index].action, DoorAction::Open(_))
                self.door_states[*state_index].open_f == FP16_ONE
            }
            MapTile::PushWall(_, state_index) => !matches!(
                self.pushwall_states[*state_index].action,
                PushwallAction::Closed
            ),
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

    pub fn get_room_id(&self, x: i32, y: i32) -> Option<i32> {
        match self.lookup_tile(x, y) {
            MapTile::Walkable(room_id, _) if room_id != ROOM_ID_NONE => Some(room_id),
            _ => None,
        }
    }

    pub fn update(&mut self, player: &Player, audio_service: &mut dyn AudioService) {
        // use (and consume) door triggers accumulated since last update
        let mut trigger_doors = std::mem::take(&mut self.tmp_door_triggers);
        let mut trigger_pushwalls = HashMap::new();
        if player.trigger {
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                match self
                    .map
                    .lookup_tile(player.x.get_int() + dx, player.y.get_int() + dy)
                {
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
            if let MapTile::Door(_, _, state_index) =
                self.map.lookup_tile(tx[i].get_int(), ty[i].get_int())
            {
                blocked_doors.insert(state_index);
            }
        }

        for (i, door_state) in self.door_states.iter_mut().enumerate() {
            door_state.update(
                trigger_doors.contains(&i),
                blocked_doors.contains(&i) || !door_state.blockers.is_empty(),
                audio_service,
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
                    if let MapTile::PushWall(id, _) =
                        self.map.map[pushwall_state.y as usize][pushwall_state.x as usize]
                    {
                        self.pushwall_patch.insert(
                            ((fi * dx) + pushwall_state.x, (fi * dy) + pushwall_state.y),
                            MapTile::OffsetWall(id, direction, f.fract()),
                        );
                    }
                }
                PushwallAction::Open(xoffs, yoffs) => {
                    if let MapTile::PushWall(id, _) =
                        self.map.map[pushwall_state.y as usize][pushwall_state.x as usize]
                    {
                        self.pushwall_patch.insert(
                            (pushwall_state.x + xoffs, pushwall_state.y + yoffs),
                            MapTile::Wall(id),
                        );
                    }
                }

                _ => (),
            }
        }
    }

    pub fn try_open_and_block_door(&mut self, door_id: usize, blocker: i32) -> bool {
        let door_state = &mut self.door_states[door_id];
        // match door_state.action {
        //     DoorAction::Closed => {
        //         self.tmp_door_triggers.insert(door_id);
        //         false
        //     }
        //     // only use open door if it is not about to close
        //     DoorAction::Open(timeout) if timeout > 15 || !door_state.blockers.is_empty() => {
        //         door_state.blockers.insert(blocker);
        //         true
        //     }
        //     _ => false,
        // }

        if door_state.open_f == FP16_ZERO {
            self.tmp_door_triggers.insert(door_id);
            false
        } else if door_state.open_f == FP16_ONE {
            door_state.blockers.insert(blocker);
            true
        } else {
            false
        }

        // return false;
    }
    pub fn unblock_door(&mut self, door_id: usize, blocker: i32) {
        self.door_states[door_id].blockers.remove(&blocker);
    }

    pub fn propagate_notifications(&mut self) {
        self.notifications = self
            .room_graph
            .propagate_notifications(&self.door_states, &self.notifications);
    }
}

impl ms::Writable for Map {
    fn write(&self, w: &mut dyn std::io::Write) -> Result<()> {
        self.door_states.write(w)?;
        self.pushwall_states.write(w)?;

        w.writei32(self.notifications.len() as i32)?;
        for room_id in &self.notifications {
            w.writei32(*room_id)?;
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
            DoorAction::Closed => w.writeu8(0)?,
            DoorAction::Opening => w.writeu8(1)?,
            DoorAction::Open(ticks) => {
                w.writeu8(2)?;
                w.writei32(*ticks)?;
            }
            DoorAction::Closing => w.writeu8(3)?,
        }
        Ok(())
    }
}

impl Loadable for DoorAction {
    fn read_from(r: &mut dyn std::io::Read) -> Result<Self> {
        let d = r.readu8()?;
        Ok(match d {
            0 => DoorAction::Closed,
            1 => DoorAction::Opening,
            2 => {
                let ticks = r.readi32()?;
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
        w.writei32(self.open_f.v)?;
        self.action.write(w)?;
        w.writeu32(self.blockers.len() as u32)?;
        for blocker in &self.blockers {
            w.writei32(*blocker)?;
        }
        Ok(())
    }
}

impl ms::Loadable for DoorState {
    fn read_from(r: &mut dyn std::io::Read) -> Result<Self> {
        let open = r.readi32()?;
        let action = DoorAction::read_from(r)?;
        let num_blockers = r.readu32()?;
        let mut blockers = HashSet::new();
        for _ in 0..num_blockers {
            blockers.insert(r.readi32()?);
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
            (PushwallAction::Closed, Some(direction)) => {
                self.action = PushwallAction::Sliding(direction, FP16_ZERO)
            }
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
        w.writei32(self.x)?;
        w.writei32(self.y)?;
        match self.action {
            PushwallAction::Closed => w.writeu8(0)?,
            PushwallAction::Sliding(direction, f) => {
                w.writeu8(1)?;
                direction.write(w)?;
                f.write(w)?;
            }
            PushwallAction::Open(x, y) => {
                w.writeu8(2)?;
                w.writei32(x)?;
                w.writei32(y)?;
            }
        }
        Ok(())
    }
}

impl ms::Loadable for PushwallState {
    fn read_from(r: &mut dyn std::io::Read) -> Result<Self> {
        let x = r.readi32()?;
        let y = r.readi32()?;
        let action = match r.readu8()? {
            0 => PushwallAction::Closed,
            1 => {
                let direction = Direction::read_from(r)?;
                let f = Fp16::read_from(r)?;
                PushwallAction::Sliding(direction, f)
            }
            2 => {
                let x = r.readi32()?;
                let y = r.readi32()?;
                PushwallAction::Open(x, y)
            }
            _ => panic!(),
        };
        Ok(Self { x, y, action })
    }
}
