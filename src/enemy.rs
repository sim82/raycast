use std::{
    collections::HashMap,
    io::{Read, Write},
    path::Path,
};

use anyhow::anyhow;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::{
    ms::{Loadable, Writable},
    prelude::*,
    thing_def::EnemyType,
};

#[derive(Debug)]
enum Think {
    None,
    Stand,
    Path,
    Chase,
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
enum Action {
    None,
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
struct StateBc {
    id: i32,
    ticks: i32,
    directional: bool,
    think: Think,
    action: Action,
    next: i32,
}

const STATE_BC_SIZE: i32 = 14;

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

#[test]
fn test_exec_ctx() {
    let mut exec_ctx = ExecCtx::load("brown.bc").unwrap();
    println!("{exec_ctx:?}");
    exec_ctx.jump_label("path").unwrap();

    for i in 0..10 {
        println!("state {i}: {:?}", exec_ctx.state);
        exec_ctx.jump(exec_ctx.state.next).unwrap();
    }
}

#[derive(Debug)]
struct ExecCtx {
    pub code: Vec<u8>,
    pub state: StateBc,
    pub labels: HashMap<String, i32>,
}

impl ExecCtx {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<ExecCtx> {
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
        Ok(ExecCtx { code, state, labels })
    }
    pub fn jump(&mut self, ptr: i32) -> Result<()> {
        self.state = StateBc::read_from(&mut std::io::Cursor::new(&self.code[(ptr as usize)..]))?;
        Ok(())
    }
    pub fn jump_label(&mut self, name: &str) -> Result<()> {
        let ptr = self.labels.get(name).ok_or(anyhow!("unknown label {name}"))?;
        self.jump(*ptr)
    }
}

type State = (i32, i32, fn(&mut Thing), fn(&mut Thing) -> (), usize, bool);
const HUMANOID_FRAMES: [State; 19] = [
    // 0
    (0, 0, think_stand, action_nil, 0, true),
    // 1
    (8, 20, think_path, action_nil, 2, true),
    (8, 5, think_nil, action_nil, 3, true),
    (16, 15, think_path, action_nil, 4, true),
    (24, 20, think_path, action_nil, 5, true),
    (24, 5, think_nil, action_nil, 6, true),
    (32, 15, think_path, action_nil, 1, true),
    // 7
    (40, 10, think_nil, action_nil, 9, false),
    // 8
    (44, 10, think_nil, action_nil, 9, false),
    // 9
    (8, 10, think_chase, action_nil, 10, true),
    (8, 3, think_nil, action_nil, 11, true),
    (16, 8, think_chase, action_nil, 12, true),
    (24, 10, think_chase, action_nil, 13, true),
    (24, 3, think_nil, action_nil, 14, true),
    (32, 8, think_chase, action_nil, 9, true),
    // 15
    (41, 15, think_nil, action_nil, 16, false),
    (42, 15, think_nil, action_nil, 17, false),
    (43, 15, think_nil, action_nil, 18, false),
    (45, 0, think_nil, action_nil, 18, false),
];

fn think_stand(thing: &mut Thing) {}
fn think_chase(thing: &mut Thing) {}

// fn think_path(thing: &mut Thing) {}
fn think_path(thing: &mut Thing) {}

fn think_nil(thing: &mut Thing) {}
fn action_nil(thing: &mut Thing) {}

pub struct Enemy {
    states: &'static [State],
    cur: usize,
    timeout: i32,
    direction: Direction,
    health: i32,
}

impl Enemy {
    pub fn set_state(&mut self, states: &'static [State]) {
        self.states = states;
        self.cur = 0;
        self.timeout = self.states[0].1;
    }
    pub fn update(&mut self) {
        if self.timeout <= 0 {
            self.cur = self.states[self.cur].4;
            self.timeout = self.states[self.cur].1;
        }

        // self.states[self.cur].2();

        self.timeout -= 1;
    }
    pub fn hit(&mut self) {
        self.health -= 7;

        if self.health > 10 {
            self.cur = 7;
        } else if self.health > 0 {
            self.cur = 8;
        } else {
            self.cur = 15;
        }
        self.timeout = self.states[self.cur].1;
    }
    pub fn get_sprite(&self, enemy_type: &EnemyType) -> SpriteIndex {
        let (id, _, _, _, _, dir) = self.states[self.cur];
        if dir {
            SpriteIndex::Directional(id + enemy_type.sprite_offset(), self.direction)
        } else {
            SpriteIndex::Undirectional(id + enemy_type.sprite_offset())
        }
    }

    pub fn spawn(
        direction: Direction,
        difficulty: crate::thing_def::Difficulty,
        enemy_type: EnemyType,
        state: crate::thing_def::EnemyState,
    ) -> Enemy {
        let cur = match state {
            crate::thing_def::EnemyState::Standing => 0,
            crate::thing_def::EnemyState::Patrolling => 1,
        };
        Enemy {
            direction,
            states: &HUMANOID_FRAMES,
            cur,
            timeout: HUMANOID_FRAMES[cur].1,
            health: 25,
        }
    }
}

impl std::fmt::Debug for Enemy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Enemy")
            .field("cur", &self.cur) /*.field("states", &self.states)*/
            .finish()
    }
}
