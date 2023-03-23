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
}

impl Think {
    pub fn from_identifier(name: &str) -> Self {
        match name {
            "None" => Think::None,
            "Stand" => Think::Stand,
            "Path" => Think::Path,
            "Chase" => Think::Chase,
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
            x => return Err(anyhow!("unhandled Think dicriminator {x}")),
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
        };
        w.write_u8(v)?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub enum Action {
    #[default]
    None,
}

impl Action {
    pub fn from_identifier(name: &str) -> Self {
        match name {
            "None" => Action::None,
            _ => panic!("unhandled Action identifier {name}"),
        }
    }
}

impl ms::Loadable for Action {
    fn read_from(_r: &mut dyn std::io::Read) -> Result<Self> {
        Ok(Action::None)
    }
}

impl ms::Writable for Action {
    fn write(&self, _w: &mut dyn std::io::Write) -> Result<()> {
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

pub const STATE_BC_SIZE: i32 = 14;

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
}

const WL6_IMAGE: &[u8] = include_bytes!("out.img");

lazy_static! {
    // static ref IMG_BROWN: ExecImage = ExecImage::load("brown_gen.bc").unwrap();
    // static ref IMG_BLUE: ExecImage = ExecImage::load("blue_gen.bc").unwrap();
    // static ref IMG_WHITE: ExecImage = ExecImage::load("white_gen.bc").unwrap();
    // static ref IMG_ROTTEN: ExecImage = ExecImage::load("rotten_gen.bc").unwrap();
    // static ref IMG_FURRY: ExecImage = ExecImage::load("furry_gen.bc").unwrap();
    static ref IMG_WL6: ExecImage = ExecImage::from_bytes(WL6_IMAGE).unwrap();
}

// fn get_exec_image(enemy_type: EnemyType) -> &'static ExecImage {
//     match enemy_type {
//         EnemyType::Brown => &IMG_BROWN,
//         EnemyType::White => &IMG_WHITE,
//         EnemyType::Blue => &IMG_BLUE,
//         EnemyType::Woof => &IMG_FURRY,
//         EnemyType::Rotten => &IMG_ROTTEN,
//     }
// }

#[derive(Debug)]
pub struct ExecImage {
    pub code: &'static [u8],
    pub labels: HashMap<String, i32>,
}

#[derive(Debug)]
pub struct ExecCtx {
    pub image: &'static ExecImage,
    pub state: StateBc,
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
        // println!("labels: {labels:?}");
        Ok(ExecImage { code, labels })
    }
    pub fn read_state(&self, ptr: i32) -> Result<StateBc> {
        StateBc::read_from(&mut std::io::Cursor::new(&self.code[(ptr as usize)..]))
    }

    pub fn read_state_by_label(&self, label: &str) -> Result<StateBc> {
        let ptr = self.labels.get(label).ok_or(anyhow!("unknown label {label}"))?;
        self.read_state(*ptr)
    }
}
