use std::{
    collections::HashMap,
    io::{Read, Seek, SeekFrom},
};

use crate::{Function, Result};
use anyhow::anyhow;
use byteorder::{LittleEndian, ReadBytesExt};
const STOP: u8 = 0xff;
const PUSH_U8: u8 = 1;
const CALL: u8 = 2;
// const LOAD_I32: u8 = 3;
const LOADI_I32: u8 = 4;
// const STORE_I32: u8 = 5;
const ADD: u8 = 6;
const JRC: u8 = 7;
const CEQ: u8 = 8;
const NOT: u8 = 9;
const DUP: u8 = 10;
const TRAP: u8 = 11;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Value {
    None,
    U8(u8),
    I32(i32),
    Bool(bool),
}

#[derive(Default)]
pub struct Env {
    pub stack: Vec<Value>,
}

#[derive(Debug)]
pub enum Event {
    Stop,
    Call(Function),
    // Load(u8),
    // Store(u8),
    Trap,
}

pub fn exec<R: Read + Seek>(bc: &mut R, env: &mut Env) -> Result<Event> {
    loop {
        match bc.read_u8()? {
            STOP => return Ok(Event::Stop),
            PUSH_U8 => env.stack.push(Value::U8(bc.read_u8()?)),
            CALL => {
                match env.stack.pop() {
                    Some(Value::U8(function)) => return Ok(Event::Call(function.try_into()?)),
                    None => return Err(anyhow!("stack underflow")),
                    _ => return Err(anyhow!("expect U8")),
                };
            }
            // LOAD_I32 => {
            //     let addr = bc.read_u8()?;
            //     return Ok(Event::Load(addr));
            // }
            LOADI_I32 => {
                let v = bc.read_i32::<LittleEndian>()?;
                env.stack.push(Value::I32(v));
            }
            // STORE_I32 => {
            //     let addr = bc.read_u8()?;
            //     return Ok(Event::Store(addr));
            // }
            TRAP => {
                // let addr = bc.read_u8()?;
                return Ok(Event::Trap);
            }
            ADD => {
                let a = env.stack.pop();
                let b = env.stack.pop();
                match (a, b) {
                    (Some(Value::I32(a)), Some(Value::I32(b))) => {
                        env.stack.push(Value::I32(a + b));
                    }
                    (a, b) => return Err(anyhow!("unhandled ADD operands: {a:?} {b:?}")),
                }
            }
            CEQ => {
                let a = env.stack.pop();
                let b = env.stack.pop();
                match (a, b) {
                    (Some(Value::I32(a)), Some(Value::I32(b))) => {
                        env.stack.push(Value::Bool(a == b))
                    }
                    (Some(Value::U8(a)), Some(Value::U8(b))) => env.stack.push(Value::Bool(a == b)),

                    (a, b) => return Err(anyhow!("unhandled CEQ operands: {a:?} {b:?}")),
                }
            }
            NOT => match env.stack.pop() {
                Some(Value::Bool(b)) => env.stack.push(Value::Bool(!b)),
                x => return Err(anyhow!("unhandled not operand {x:?}")),
            },
            DUP => match env.stack.last() {
                Some(v) => env.stack.push(v.clone()),
                _ => return Err(anyhow!("dup on empty stack")),
            },
            JRC => {
                let offs = bc.read_i32::<LittleEndian>()?;
                match env.stack.pop() {
                    Some(Value::Bool(b)) => {
                        if b {
                            bc.seek(SeekFrom::Current(offs as i64))?;
                        }
                    }
                    x => return Err(anyhow!("unhandled jrc operand {x:?}")),
                }
            }
            x => return Err(anyhow!("unhandled opcode {x:?}")),
        }
    }
}

#[derive(Default, Clone)]
pub struct Codegen {
    code: Vec<u8>,
    labels: HashMap<String, usize>,
    label_refs: Vec<(String, usize)>,
}

impl Codegen {
    pub fn function_call(mut self, function: Function) -> Self {
        self.code.push(PUSH_U8);
        self.code.push(function.into());
        self.code.push(CALL);
        self
    }
    // pub fn load_i32(mut self, addr: u8) -> Self {
    //     self.code.push(LOAD_I32);
    //     self.code.push(addr);
    //     self
    // }
    pub fn loadi_i32(mut self, v: i32) -> Self {
        self.code.push(LOADI_I32);
        self.code.extend_from_slice(&v.to_le_bytes());
        self
    }
    pub fn loadi_u8(mut self, v: u8) -> Self {
        self.code.push(PUSH_U8);
        self.code.push(v);
        self
    }
    // pub fn store_i32(mut self, addr: u8) -> Self {
    //     self.code.push(STORE_I32);
    //     self.code.push(addr);
    //     self
    // }
    pub fn trap(mut self) -> Self {
        self.code.push(TRAP);
        self
    }
    pub fn add(mut self) -> Self {
        self.code.push(ADD);
        self
    }
    pub fn ceq(mut self) -> Self {
        self.code.push(CEQ);
        self
    }
    pub fn bin_not(mut self) -> Self {
        self.code.push(NOT);
        self
    }
    pub fn call(mut self) -> Self {
        self.code.push(CALL);
        self
    }
    pub fn dup(mut self) -> Self {
        self.code.push(DUP);
        self
    }
    pub fn stop(mut self) -> Self {
        self.code.push(STOP);
        self
    }
    pub fn jrc(mut self, offs: i32) -> Self {
        self.code.push(JRC);
        self.code.extend_from_slice(&offs.to_le_bytes());
        self
    }
    pub fn jrc_label(mut self, name: &str) -> Self {
        self.code.push(JRC);
        self.label_refs.push((name.into(), self.code.len()));
        self.code.extend_from_slice(&0i32.to_le_bytes());
        self
    }
    pub fn label(mut self, label: &str) -> Self {
        self.labels.insert(label.into(), self.code.len());
        self
    }
    pub fn into_code(mut self) -> Vec<u8> {
        for (label, pos) in self.label_refs {
            let label_pos = *self
                .labels
                .get(&label)
                .unwrap_or_else(|| panic!("could not find label {label}"));
            let offs = label_pos as i32 - pos as i32 - 4;
            self.code[pos..(pos + 4)].copy_from_slice(&offs.to_le_bytes());
        }
        self.code
    }
    pub fn get_code(&self) -> &[u8] {
        &self.code
    }
}

#[test]
fn test_codegen() {
    let code = Codegen::default()
        .function_call(Function::ActionDie)
        .stop()
        .into_code();
    assert_eq!(code[..], [0x1, 0x5, 0x2, 0xff]);
}
#[test]
fn test_exec() {
    let mut env = Env::default();
    let bc = [0x1, 0x5, 0x2, 0xff];

    let mut c = std::io::Cursor::new(bc);
    let e = exec(&mut c, &mut env);
    assert!(matches!(e, Ok(Event::Call(Function::ActionDie))));

    let e = exec(&mut c, &mut env);
    assert!(matches!(e, Ok(Event::Stop)));
}
#[test]
fn test_ceq() {
    let mut env = Env::default();
    let bc = Codegen::default()
        .loadi_u8(123)
        .loadi_u8(123)
        .ceq()
        .loadi_u8(123)
        .loadi_u8(124)
        .ceq()
        .stop()
        .into_code();

    let mut c = std::io::Cursor::new(bc);
    let e = exec(&mut c, &mut env);
    assert!(matches!(e, Ok(Event::Stop)));
    assert_eq!(env.stack, [Value::Bool(true), Value::Bool(false)]);
}
#[test]
fn test_cond_jmp() {
    let mut env = Env::default();
    let bc = Codegen::default()
        .loadi_u8(123)
        .loadi_u8(123)
        .ceq()
        .jrc_label("after") // expect to jump over this
        .loadi_i32(4711)
        .label("after")
        .loadi_u8(123)
        .loadi_u8(124)
        .ceq()
        .jrc_label("after2") // expect no jump over
        .loadi_i32(4712)
        .label("after2")
        .stop()
        .into_code();
    println!("{bc:?}");
    let mut c = std::io::Cursor::new(bc);
    let e = exec(&mut c, &mut env);
    assert!(matches!(e, Ok(Event::Stop)));
    assert_eq!(env.stack, [Value::I32(4712)]);
}
#[test]
fn test_loop() {
    let mut env = Env::default();
    let bc = Codegen::default()
        .loadi_i32(5) // loop counter
        .label("loop")
        .dup() // dup counter and add 4711 (for test load event)
        .loadi_i32(4711)
        .add()
        .trap() // emit test event
        .loadi_i32(-1) // sub 1 from counter
        .add()
        .dup() // dup counter and compare zero
        .loadi_i32(0)
        .ceq() // compare and jump to loop: if not zero
        .bin_not()
        .jrc_label("loop")
        .stop()
        .into_code();
    println!("{bc:?}");
    let mut c = std::io::Cursor::new(bc);
    for i in 0..5 {
        let e = exec(&mut c, &mut env);
        let v = env.stack.pop();
        println!("{e:?} {v:?}");
        assert!(matches!(e, Ok(Event::Trap)));
        assert_eq!(v, Some(Value::I32(4716 - i)));
    }
    let e = exec(&mut c, &mut env);
    assert!(matches!(e, Ok(Event::Stop)));
}
