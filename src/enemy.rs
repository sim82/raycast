use std::{
    collections::HashMap,
    io::{Read, Write},
    path::Path,
};

use crate::{
    ms::{Loadable, Writable},
    prelude::*,
    thing_def::EnemyType,
};
use anyhow::anyhow;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use lazy_static::lazy_static;

#[derive(Debug)]
pub enum Think {
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

#[derive(Debug)]
pub enum Action {
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
    fn read_from(r: &mut dyn std::io::Read) -> Result<Self> {
        Ok(Action::None)
    }
}

impl ms::Writable for Action {
    fn write(&self, w: &mut dyn std::io::Write) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
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

#[test]
fn write_state_bc() {
    let states = [
        // 0
        StateBc::new(0, 0, Think::Stand, Action::None, 0, true),
        // 1
        StateBc::new(8, 20, Think::Path, Action::None, 2, true),
        StateBc::new(8, 5, Think::None, Action::None, 3, true),
        StateBc::new(16, 15, Think::Path, Action::None, 4, true),
        StateBc::new(24, 20, Think::Path, Action::None, 5, true),
        StateBc::new(24, 5, Think::None, Action::None, 6, true),
        StateBc::new(32, 15, Think::Path, Action::None, 1, true),
        // 7
        StateBc::new(40, 10, Think::None, Action::None, 9, false),
        // 8
        StateBc::new(44, 10, Think::None, Action::None, 9, false),
        // 9
        StateBc::new(8, 10, Think::Chase, Action::None, 10, true),
        StateBc::new(8, 3, Think::None, Action::None, 11, true),
        StateBc::new(16, 8, Think::Chase, Action::None, 12, true),
        StateBc::new(24, 10, Think::Chase, Action::None, 13, true),
        StateBc::new(24, 3, Think::None, Action::None, 14, true),
        StateBc::new(32, 8, Think::Chase, Action::None, 9, true),
        // 15
        StateBc::new(41, 15, Think::None, Action::None, 16, false),
        StateBc::new(42, 15, Think::None, Action::None, 17, false),
        StateBc::new(43, 15, Think::None, Action::None, 18, false),
        StateBc::new(45, 0, Think::None, Action::None, 18, false),
    ];
    let mut f = std::fs::File::create("brown.bc").unwrap();
    for state in &states[..] {
        state.write(&mut f).unwrap();
    }
    let mut f = std::fs::File::create("brown.lb").unwrap();
    let labels = [
        ("stand", 0),
        ("path", STATE_BC_SIZE * 1),
        ("pain1", STATE_BC_SIZE * 7),
        ("pain2", STATE_BC_SIZE * 8),
        ("chase", STATE_BC_SIZE * 9),
        ("die", STATE_BC_SIZE * 15),
    ];
    f.write_i32::<LittleEndian>(labels.len() as i32).unwrap();
    for (name, ptr) in &labels {
        let b = name.as_bytes();
        f.write_u8(b.len() as u8).unwrap();
        let _ = f.write(b).unwrap();
        f.write_i32::<LittleEndian>(*ptr).unwrap();
    }
}

lazy_static! {
    static ref IMG_BROWN: ExecImage = ExecImage::load("brown_gen.bc").unwrap();
    static ref IMG_BLUE: ExecImage = ExecImage::load("blue_gen.bc").unwrap();
    static ref IMG_WHITE: ExecImage = ExecImage::load("white_gen.bc").unwrap();
    static ref IMG_ROTTEN: ExecImage = ExecImage::load("rotten_gen.bc").unwrap();
    static ref IMG_FURRY: ExecImage = ExecImage::load("furry_gen.bc").unwrap();
}

fn get_exec_image(enemy_type: EnemyType) -> &'static ExecImage {
    match enemy_type {
        EnemyType::Brown => &IMG_BROWN,
        EnemyType::White => &IMG_WHITE,
        EnemyType::Blue => &IMG_BLUE,
        EnemyType::Woof => &IMG_FURRY,
        EnemyType::Rotten => &IMG_ROTTEN,
    }
}

#[derive(Debug)]
struct ExecImage {
    pub code: Vec<u8>,
    pub labels: HashMap<String, i32>,
}

#[derive(Debug)]
struct ExecCtx {
    pub image: &'static ExecImage,
    pub state: StateBc,
}

impl ExecCtx {
    pub fn new(image: &'static ExecImage) -> Result<Self> {
        let state = StateBc::read_from(&mut std::io::Cursor::new(&image.code[..]))?;
        Ok(ExecCtx { image, state })
    }
    pub fn jump(&mut self, ptr: i32) -> Result<()> {
        self.state = StateBc::read_from(&mut std::io::Cursor::new(&self.image.code[(ptr as usize)..]))?;
        Ok(())
    }
    pub fn jump_label(&mut self, name: &str) -> Result<()> {
        let ptr = self.image.labels.get(name).ok_or(anyhow!("unknown label {name}"))?;
        self.jump(*ptr)
    }
}

impl ExecImage {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<ExecImage> {
        let mut lb_path = path.as_ref().to_path_buf();

        let code = std::fs::read(path)?;

        let mut c = std::io::Cursor::new(&code);
        let state = StateBc::read_from(&mut c)?;

        lb_path.set_extension("lb");
        println!("{lb_path:?}");
        let mut f = std::fs::File::open(lb_path)?;
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
        Ok(ExecImage { code, labels })
    }
}

fn think_stand(thing: &mut Thing) {}
fn think_chase(thing: &mut Thing) {}

// fn think_path(thing: &mut Thing) {}
fn think_path(thing: &mut Thing) {}

fn think_nil(thing: &mut Thing) {}
fn action_nil(thing: &mut Thing) {}

pub struct Enemy {
    exec_ctx: ExecCtx,
    enemy_type: EnemyType,
    direction: Direction,
    health: i32,
}

impl ms::Loadable for Enemy {
    fn read_from(r: &mut dyn Read) -> Result<Self> {
        let enemy_type = EnemyType::read_from(r)?;
        let state = StateBc::read_from(r)?;
        let direction = Direction::read_from(r)?;
        let health = r.read_i32::<LittleEndian>()?;
        Ok(Enemy {
            exec_ctx: ExecCtx {
                image: get_exec_image(enemy_type),
                state,
            },
            enemy_type,
            direction,
            health,
        })
    }
}

impl ms::Writable for Enemy {
    fn write(&self, w: &mut dyn Write) -> Result<()> {
        self.enemy_type.write(w)?;
        self.exec_ctx.state.write(w)?;
        self.direction.write(w)?;
        w.write_i32::<LittleEndian>(self.health)?;
        Ok(())
    }
}

impl Enemy {
    // pub fn set_state(&mut self, states: &'static [State]) {
    //     self.states = states;
    //     self.cur = 0;
    //     self.timeout = self.states[0].1;
    // }
    pub fn set_state(&mut self, name: &str) {
        self.exec_ctx.jump_label(name).unwrap();
        println!("state: {self:?}");
    }
    pub fn update(&mut self) {
        if self.exec_ctx.state.ticks <= 0 {
            self.exec_ctx.jump(self.exec_ctx.state.next).unwrap();
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
    pub fn get_sprite(&self, enemy_type: &EnemyType) -> SpriteIndex {
        if self.exec_ctx.state.directional {
            SpriteIndex::Directional(self.exec_ctx.state.id, self.direction)
        } else {
            SpriteIndex::Undirectional(self.exec_ctx.state.id)
        }
    }

    pub fn spawn(
        direction: Direction,
        difficulty: crate::thing_def::Difficulty,
        enemy_type: EnemyType,
        state: crate::thing_def::EnemyState,
    ) -> Enemy {
        let start_label = match state {
            crate::thing_def::EnemyState::Standing => "stand",
            crate::thing_def::EnemyState::Patrolling => "path",
        };

        let mut exec_ctx = ExecCtx::new(get_exec_image(enemy_type)).unwrap();
        exec_ctx.jump_label(start_label).unwrap();
        Enemy {
            direction,
            exec_ctx,
            enemy_type,
            health: 25,
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
