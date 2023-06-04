use std::{
    collections::HashMap,
    io::{Read, Seek, SeekFrom},
};

use crate::{ms::endian::ReadExt, Function, Result};
use anyhow::anyhow;

const STOP: u8 = 0xff;
const PUSH_U8: u8 = 1;
const CALL: u8 = 2;
const LOADI_I32: u8 = 3;
const ADD: u8 = 4;
const JRC: u8 = 5;
const CEQ: u8 = 6;
const NOT: u8 = 7;
const DUP: u8 = 8;
const TRAP: u8 = 9;
const GOSTATE: u8 = 10;

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
    GoState,
}

pub fn exec<R: Read + Seek>(bc: &mut R, env: &mut Env) -> Result<Event> {
    loop {
        match bc.readu8()? {
            STOP => return Ok(Event::Stop),
            PUSH_U8 => env.stack.push(Value::U8(bc.readu8()?)),
            CALL => {
                match env.stack.pop() {
                    Some(Value::U8(function)) => return Ok(Event::Call(function.try_into()?)),
                    None => return Err(anyhow!("stack underflow")),
                    _ => return Err(anyhow!("expect U8")),
                };
            }
            LOADI_I32 => {
                let v = bc.readi32()?;
                env.stack.push(Value::I32(v));
            }
            TRAP => {
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
                let offs = bc.readi32()?;
                match env.stack.pop() {
                    Some(Value::Bool(b)) => {
                        if b {
                            bc.seek(SeekFrom::Current(offs as i64))?;
                        }
                    }
                    x => return Err(anyhow!("unhandled jrc operand {x:?}")),
                }
            }
            GOSTATE => {
                return Ok(Event::GoState);
            }
            x => return Err(anyhow!("unhandled opcode {x:?}")),
        }
    }
}

#[derive(Default, Clone)]
pub struct Codegen {
    code: Vec<u8>,
    labels: HashMap<String, usize>,
    state_labels: HashMap<String, i32>,
    label_refs: Vec<(String, usize)>,
    state_label_refs: Vec<(String, usize)>,
    annotations: HashMap<String, String>,
    autolabel_index: i32,
}

impl Codegen {
    pub fn function_call(mut self, function: Function) -> Self {
        self.code.push(PUSH_U8);
        self.code.push(function.into());
        self.code.push(CALL);
        self
    }
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
        if self.code.last() == Some(&NOT) {
            self.code.pop();
        } else {
            self.code.push(NOT);
        }
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
        if self.code.last() != Some(&STOP) {
            self.code.push(STOP);
        }
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
    pub fn loadsl(mut self, label: &str) -> Codegen {
        self.code.push(LOADI_I32);
        self.state_label_refs.push((label.into(), self.code.len()));
        self.code.extend_from_slice(&0i32.to_le_bytes());
        self
    }
    pub fn gostate(mut self) -> Codegen {
        self.code.push(GOSTATE);
        self
    }
    pub fn with_state_label_ptrs(mut self, label_ptrs: &HashMap<String, i32>) -> Codegen {
        self.state_labels = label_ptrs.clone();
        self
    }
    /// Finalize and return Vec of generated code. This step resolves labelled jumps to the correct
    /// offset address.
    ///
    /// # Panics
    ///
    /// Panics if a label name cannot be resolved
    pub fn finalize(mut self) -> Vec<u8> {
        for (label, pos) in self.label_refs {
            let label_pos = *self
                .labels
                .get(&label)
                .unwrap_or_else(|| panic!("could not find label {label}"));
            let offs = label_pos as i32 - pos as i32 - 4;
            self.code[pos..(pos + 4)].copy_from_slice(&offs.to_le_bytes());
        }
        for (label, pos) in self.state_label_refs {
            let label_pos = *self
                .state_labels
                .get(&label)
                .unwrap_or_else(|| panic!("could not find state_label {label}"));
            self.code[pos..(pos + 4)].copy_from_slice(&label_pos.to_le_bytes());
        }
        self.code
    }

    pub fn len(&self) -> usize {
        self.code.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn with_annotation(mut self, key: &str, value: &str) -> Self {
        self.annotations.insert(key.to_string(), value.to_string());
        self
    }
    pub fn get_annotation(&self, key: &str) -> Option<&str> {
        self.annotations.get(key).map(|s| s.as_str())
    }
    pub fn next_autolabel(&mut self) -> String {
        let label = format!("autolabel{}", self.autolabel_index);
        self.autolabel_index += 1;
        label
    }
}

#[test]
fn test_codegen() {
    let code = Codegen::default()
        .function_call(Function::ActionDie)
        .stop()
        .finalize();
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
        .finalize();

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
        .finalize();
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
        .finalize();
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
