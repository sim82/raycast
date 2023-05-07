use anyhow::anyhow;
use ms::endian::{ReadExt, WriteExt};
use prelude::ms::Loadable;
use std::{
    collections::HashMap,
    io::{Cursor, Read, Write},
};
pub mod ms;
pub use anyhow::Result;

pub mod prelude {
    pub use crate::ms;
    pub use crate::Result;
}

#[cfg(feature = "compiler")]
pub mod compiler;

pub mod opcode;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Function {
    #[default]
    None,
    ThinkStand,
    ThinkPath,
    ThinkChase,
    ThinkDogChase,
    ActionDie,
    ActionShoot,
    ActionBite,
}
impl Function {
    pub fn try_from_identifier(name: &str) -> Option<Self> {
        Some(match name {
            "None" => Self::None,
            "ThinkStand" => Self::ThinkStand,
            "ThinkPath" => Self::ThinkPath,
            "ThinkChase" => Self::ThinkChase,
            "ThinkDogChase" => Self::ThinkDogChase,
            "ActionDie" => Self::ActionDie,
            "ActionShoot" => Self::ActionShoot,
            "ActionBite" => Self::ActionBite,
            _ => return None,
        })
    }
}

impl TryFrom<u8> for Function {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self> {
        Ok(match value {
            // KEEP IN SYNC WITH .st files!
            0 => Self::None,
            1 => Self::ThinkStand,
            2 => Self::ThinkPath,
            3 => Self::ThinkChase,
            4 => Self::ThinkDogChase,
            5 => Self::ActionDie,
            6 => Self::ActionShoot,
            7 => Self::ActionBite,
            x => return Err(anyhow!("unhandled Think discriminator {x}")),
        })
    }
}

impl From<Function> for u8 {
    fn from(val: Function) -> Self {
        match val {
            // KEEP IN SYNC WITH .st files!
            Function::None => 0,
            Function::ThinkStand => 1,
            Function::ThinkPath => 2,
            Function::ThinkChase => 3,
            Function::ThinkDogChase => 4,
            Function::ActionDie => 5,
            Function::ActionShoot => 6,
            Function::ActionBite => 7,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Direction {
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
    North,
    NorthEast,
}

impl ms::Loadable for Direction {
    fn read_from(r: &mut dyn std::io::Read) -> Result<Self> {
        Ok(match r.readu8()? {
            0 => Direction::North, // TEMP: keep backward compatible with serialized Direction enum
            1 => Direction::East,
            2 => Direction::South,
            3 => Direction::West,
            4 => Direction::NorthEast,
            5 => Direction::NorthWest,
            6 => Direction::SouthWest,
            7 => Direction::SouthEast,
            x => return Err(anyhow!("unrecognized Direction discriminator {x}")),
        })
    }
}

impl ms::Writable for Direction {
    fn write(&self, w: &mut dyn Write) -> Result<()> {
        match self {
            Direction::North => w.writeu8(0)?,
            Direction::East => w.writeu8(1)?,
            Direction::South => w.writeu8(2)?,
            Direction::West => w.writeu8(3)?,
            Direction::NorthEast => w.writeu8(4)?,
            Direction::NorthWest => w.writeu8(5)?,
            Direction::SouthWest => w.writeu8(6)?,
            Direction::SouthEast => w.writeu8(7)?,
        }

        Ok(())
    }
}

impl Direction {
    pub fn try_from_prop_id(p: i32) -> Option<Direction> {
        Some(match p {
            0x5a => Direction::East,
            0x5b => Direction::NorthEast,
            0x5c => Direction::North,
            0x5d => Direction::NorthWest,
            0x5e => Direction::West,
            0x5f => Direction::SouthWest,
            0x60 => Direction::South,
            0x61 => Direction::SouthEast,
            _ => return None,
        })
    }

    pub fn is_diagonal(&self) -> bool {
        matches!(
            self,
            Direction::SouthEast
                | Direction::SouthWest
                | Direction::NorthWest
                | Direction::NorthEast
        )
    }
}

impl Direction {
    pub fn sprite_offset(&self) -> i32 {
        match &self {
            Direction::North => 0,
            Direction::NorthEast => 1,
            Direction::East => 2,
            Direction::SouthEast => 3,
            Direction::South => 4,
            Direction::SouthWest => 5,
            Direction::West => 6,
            Direction::NorthWest => 7,
        }
    }

    pub fn tile_offset(&self) -> (i32, i32) {
        match self {
            Direction::NorthWest => (-1, -1),
            Direction::North => (0, -1),
            Direction::NorthEast => (1, -1),
            Direction::East => (1, 0),
            Direction::SouthEast => (1, 1),
            Direction::South => (0, 1),
            Direction::SouthWest => (-1, 1),
            Direction::West => (-1, 0),
        }
    }
    pub fn x_offs(&self) -> i32 {
        match self {
            Direction::NorthWest => -1,
            Direction::North => 0,
            Direction::NorthEast => 1,
            Direction::East => 1,
            Direction::SouthEast => 1,
            Direction::South => 0,
            Direction::SouthWest => -1,
            Direction::West => -1,
        }
    }
    pub fn y_offs(&self) -> i32 {
        match self {
            Direction::NorthWest => -1,
            Direction::North => -1,
            Direction::NorthEast => -1,
            Direction::East => 0,
            Direction::SouthEast => 1,
            Direction::South => 1,
            Direction::SouthWest => 1,
            Direction::West => 0,
        }
    }
    pub fn opposite(&self) -> Direction {
        match self {
            Direction::East => Direction::West,
            Direction::SouthEast => Direction::NorthWest,
            Direction::South => Direction::North,
            Direction::SouthWest => Direction::NorthEast,
            Direction::West => Direction::East,
            Direction::NorthWest => Direction::SouthEast,
            Direction::North => Direction::South,
            Direction::NorthEast => Direction::SouthWest,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnemySpawnInfo {
    pub id: i32,
    pub direction: Direction,
    pub state: String,
    pub spawn_on_death: Option<i32>,
}

impl ms::Loadable for EnemySpawnInfo {
    fn read_from(r: &mut dyn Read) -> Result<Self> {
        let id = r.readi32()?;
        let direction = Direction::read_from(r)?;
        let state = String::read_from(r)?;

        let spawn_on_death = match r.readi32()? {
            x if x >= 0 => Some(x),
            _ => None,
        };

        Ok(Self {
            id,
            direction,
            state,

            spawn_on_death,
        })
    }
}

impl ms::Writable for EnemySpawnInfo {
    fn write(&self, w: &mut dyn Write) -> Result<()> {
        w.writei32(self.id)?;
        self.direction.write(w)?;
        self.state.write(w)?;

        w.writei32(self.spawn_on_death.unwrap_or(-1))?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct StateBc {
    pub id: i32,
    pub ticks: i32,
    pub directional: bool,
    pub think_offs: i32,
    pub action_offs: i32,
    pub next: i32,
}

pub const STATE_BC_SIZE: i32 = 21;

impl ms::Loadable for StateBc {
    fn read_from(r: &mut dyn std::io::Read) -> Result<Self> {
        let id = r.readi32()?;
        let ticks = r.readi32()?;
        let directional = r.readu8()? != 0;
        let think_offs = r.readi32()?;
        let action_offs = r.readi32()?;
        let next = r.readi32()?;
        Ok(StateBc {
            id,
            ticks,
            directional,
            think_offs,
            action_offs,
            next,
        })
    }
}

impl ms::Writable for StateBc {
    fn write(&self, w: &mut dyn std::io::Write) -> Result<()> {
        w.writei32(self.id)?;
        w.writei32(self.ticks)?;
        w.writeu8(if self.directional { 1 } else { 0 })?;
        w.writei32(self.think_offs)?;
        w.writei32(self.action_offs)?;
        w.writei32(self.next)?;
        Ok(())
    }
}

impl StateBc {
    pub fn new(id: i32, ticks: i32, next: i32, directional: bool) -> Self {
        StateBc {
            id,
            ticks,
            directional,
            think_offs: 0,
            action_offs: 0,
            next: next * STATE_BC_SIZE,
        }
    }
}

#[derive(Debug)]
pub struct ExecImage {
    pub code: &'static [u8],
    pub labels: HashMap<String, i32>,
    pub spawn_infos: SpawnInfos,
}

#[derive(Debug)]
pub struct ExecCtx {
    pub image: &'static ExecImage,
    pub state: StateBc,
}

#[derive(Debug)]
pub struct SpawnInfos {
    pub spawn_infos: Vec<EnemySpawnInfo>,
}

impl ExecCtx {
    pub fn new(initial_label: &str, image: &'static ExecImage) -> Result<Self> {
        let state = image.read_state_by_label(initial_label)?;
        Ok(ExecCtx { image, state })
    }
    pub fn jump(&mut self, ptr: i32) -> Result<()> {
        self.state = self.image.read_state(ptr)?;
        Ok(())
    }
    pub fn jump_label(&mut self, name: &str) -> Result<()> {
        self.state = self.image.read_state_by_label(name)?;
        Ok(())
    }
}

impl ExecCtx {
    pub fn read_from(r: &mut dyn Read, image: &'static ExecImage) -> Result<Self> {
        let state = StateBc::read_from(r)?;
        Ok(ExecCtx { image, state })
    }
}

impl ms::Writable for ExecCtx {
    fn write(&self, w: &mut dyn Write) -> Result<()> {
        self.state.write(w)?;
        Ok(())
    }
}

impl ExecImage {
    pub fn from_bytes(code: &'static [u8]) -> Result<ExecImage> {
        let mut f = Cursor::new(code);
        let num_labels = f.readi32()?;
        let mut labels = HashMap::new();
        // let mut tmp = [0u8; 16];
        for _ in 0..num_labels {
            let len = f.readu8()? as usize;
            let mut name = vec![0u8; len];
            f.read_exact(&mut name)?;
            let ptr = f.readi32()?;

            labels.insert(String::from_utf8(name)?, ptr);
        }
        let spawn_infos = SpawnInfos::read_from(&mut f)?;
        // println!("labels: {labels:?}");
        let code_offs = f.position() as usize;
        Ok(ExecImage {
            code: &code[code_offs..],
            labels,
            spawn_infos,
        })
    }
    pub fn read_state(&self, ptr: i32) -> Result<StateBc> {
        StateBc::read_from(&mut std::io::Cursor::new(&self.code[(ptr as usize)..]))
    }

    pub fn read_state_by_label(&self, label: &str) -> Result<StateBc> {
        let ptr = self
            .labels
            .get(label)
            .ok_or(anyhow!("unknown label {label}"))?;
        let res = self.read_state(*ptr);
        if res.is_err() {
            println!("while read from: 0x{ptr:x}");
        }
        res
    }
}

impl SpawnInfos {
    pub fn from_bytes(buf: &[u8]) -> Result<Self> {
        let mut f = Cursor::new(buf);
        SpawnInfos::read_from(&mut f)
    }
    pub fn find_spawn_info(&self, id: i32) -> Option<&EnemySpawnInfo> {
        self.spawn_infos.iter().find(|&info| info.id == id)
    }
}

impl ms::Loadable for SpawnInfos {
    fn read_from(r: &mut dyn Read) -> Result<Self> {
        let num = r.readi32()?;
        let mut spawn_infos = Vec::new();

        for _ in 0..num {
            spawn_infos.push(EnemySpawnInfo::read_from(r)?)
        }

        Ok(Self { spawn_infos })
    }
}

impl ms::Writable for SpawnInfos {
    fn write(&self, w: &mut dyn Write) -> Result<()> {
        w.writei32(self.spawn_infos.len() as i32)?;
        for spawn_info in &self.spawn_infos {
            spawn_info.write(w)?;
        }
        Ok(())
    }
}

#[test]
fn test_state_bc_size() {
    let mut c = std::io::Cursor::new(Vec::<u8>::new());
    let state_bc = StateBc::default();
    ms::Writable::write(&state_bc, &mut c).unwrap();
    assert_eq!(c.into_inner().len(), STATE_BC_SIZE as usize);
}
