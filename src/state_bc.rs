use crate::{ms::Loadable, prelude::*};
use anyhow::anyhow;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    io::{Cursor, Read, Write},
};

#[derive(Debug, Default)]
pub enum Think {
    #[default]
    None,
    Stand,
    Path,
    Chase,
    Shoot,
    Bite,
    DogChase,
}

impl Think {
    pub fn from_identifier(name: &str) -> Self {
        match name {
            "None" => Think::None,
            "Stand" => Think::Stand,
            "Path" => Think::Path,
            "Chase" => Think::Chase,
            "Shoot" => Think::Shoot,
            "Bite" => Think::Bite,
            "DogChase" => Think::DogChase,
            _ => panic!("unhandled Think identifier {name}"),
        }
    }
}

impl ms::Loadable for Think {
    fn read_from(r: &mut dyn std::io::Read) -> Result<Self> {
        Ok(match r.read_u8()? {
            0 => Think::None,
            1 => Think::Stand,
            2 => Think::Path,
            3 => Think::Chase,
            4 => Think::Shoot,
            5 => Think::Bite,
            6 => Think::DogChase,
            x => return Err(anyhow!("unhandled Think discriminator {x}")),
        })
    }
}

impl ms::Writable for Think {
    fn write(&self, w: &mut dyn std::io::Write) -> Result<()> {
        let v = match self {
            Think::None => 0,
            Think::Stand => 1,
            Think::Path => 2,
            Think::Chase => 3,
            Think::Shoot => 4,
            Think::Bite => 5,
            Think::DogChase => 6,
        };
        w.write_u8(v)?;
        Ok(())
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub enum Action {
    #[default]
    None,
    Die,
    Shoot,
    Bite,
}

impl Action {
    pub fn from_identifier(name: &str) -> Self {
        match name {
            "None" => Action::None,
            "Die" => Action::Die,
            "Shoot" => Action::Shoot,
            "Bite" => Action::Bite,
            _ => panic!("unhandled Action identifier {name}"),
        }
    }
}

impl ms::Loadable for Action {
    fn read_from(r: &mut dyn std::io::Read) -> Result<Self> {
        Ok(match r.read_u8()? {
            0 => Action::None,
            1 => Action::Die,
            2 => Action::Shoot,
            3 => Action::Bite,
            x => return Err(anyhow!("unhandled Action dicriminator {x}")),
        })
    }
}

impl ms::Writable for Action {
    fn write(&self, w: &mut dyn std::io::Write) -> Result<()> {
        let v = match self {
            Action::None => 0,
            Action::Die => 1,
            Action::Shoot => 2,
            Action::Bite => 3,
        };
        w.write_u8(v)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct EnemySpawnInfo {
    pub id: i32,
    pub direction: Direction,
    pub state: String,
    pub bonus_item: Option<Item>,
}

impl ms::Loadable for EnemySpawnInfo {
    fn read_from(r: &mut dyn Read) -> Result<Self> {
        let id = r.read_i32::<LittleEndian>()?;
        let direction = Direction::read_from(r)?;
        let state = String::read_from(r)?;
        let bonus_item = Option::<Item>::read_from(r)?;
        Ok(Self {
            id,
            direction,
            state,
            bonus_item,
        })
    }
}

impl ms::Writable for EnemySpawnInfo {
    fn write(&self, w: &mut dyn Write) -> Result<()> {
        w.write_i32::<LittleEndian>(self.id)?;
        self.direction.write(w)?;
        self.state.write(w)?;
        self.bonus_item.write(w)?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct StateBc {
    pub id: i32,
    pub ticks: i32,
    pub directional: bool,
    pub think: Think,
    pub action: Action,
    pub next: i32,
}

pub const STATE_BC_SIZE: i32 = 15;

impl ms::Loadable for StateBc {
    fn read_from(r: &mut dyn std::io::Read) -> Result<Self> {
        let id = r.read_i32::<LittleEndian>()?;
        let ticks = r.read_i32::<LittleEndian>()?;
        let directional = r.read_u8()? != 0;
        let think = Think::read_from(r)?;
        let action = Action::read_from(r)?;
        let next = r.read_i32::<LittleEndian>()?;
        Ok(StateBc {
            id,
            ticks,
            directional,
            think,
            action,
            next,
        })
    }
}

impl ms::Writable for StateBc {
    fn write(&self, w: &mut dyn std::io::Write) -> Result<()> {
        w.write_i32::<LittleEndian>(self.id)?;
        w.write_i32::<LittleEndian>(self.ticks)?;
        w.write_u8(if self.directional { 1 } else { 0 })?;
        self.think.write(w)?;
        self.action.write(w)?;
        w.write_i32::<LittleEndian>(self.next)?;
        Ok(())
    }
}

impl StateBc {
    pub fn new(id: i32, ticks: i32, think: Think, action: Action, next: i32, directional: bool) -> Self {
        StateBc {
            id,
            ticks,
            directional,
            think,
            action,
            next: next * STATE_BC_SIZE,
        }
    }

    pub fn take_action(&mut self) -> Action {
        if self.action != Action::None {
            std::mem::take(&mut self.action)
        } else {
            Action::None
        }
    }
}

const WL6_IMAGE: &[u8] = include_bytes!("out.img");
const WL6_SPAWN_INFO: &[u8] = include_bytes!("out.spawn");

lazy_static! {
    static ref IMG_WL6: ExecImage = ExecImage::from_bytes(WL6_IMAGE).unwrap();
    pub static ref SPAWN_INFO_WL6: SpawnInfos = SpawnInfos::from_bytes(WL6_SPAWN_INFO).unwrap();
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
    pub fn new(initial_label: &str) -> Result<Self> {
        let image = &IMG_WL6; //get_exec_image(enemy_type);
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

impl ms::Loadable for ExecCtx {
    fn read_from(r: &mut dyn Read) -> Result<Self> {
        let state = StateBc::read_from(r)?;
        Ok(ExecCtx { image: &IMG_WL6, state })
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
        let num_labels = f.read_i32::<LittleEndian>()?;
        let mut labels = HashMap::new();
        // let mut tmp = [0u8; 16];
        for _ in 0..num_labels {
            let len = f.read_u8()? as usize;
            let mut name = vec![0u8; len];
            f.read_exact(&mut name)?;
            let ptr = f.read_i32::<LittleEndian>()?;

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
        let ptr = self.labels.get(label).ok_or(anyhow!("unknown label {label}"))?;
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
    pub fn find_spawn_info(&self, id: u16) -> Option<&EnemySpawnInfo> {
        self.spawn_infos.iter().find(|&info| info.id == id as i32)
    }
}

impl ms::Loadable for SpawnInfos {
    fn read_from(r: &mut dyn Read) -> Result<Self> {
        let num = r.read_i32::<LittleEndian>()?;
        let mut spawn_infos = Vec::new();

        for _ in 0..num {
            spawn_infos.push(EnemySpawnInfo::read_from(r)?)
        }

        Ok(Self { spawn_infos })
    }
}

impl ms::Writable for SpawnInfos {
    fn write(&self, w: &mut dyn Write) -> Result<()> {
        w.write_i32::<LittleEndian>(self.spawn_infos.len() as i32)?;
        for spawn_info in &self.spawn_infos {
            spawn_info.write(w)?;
        }
        Ok(())
    }
}
