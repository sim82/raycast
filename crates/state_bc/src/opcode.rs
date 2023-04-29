use std::io::Read;

use crate::{Function, Result};
use anyhow::anyhow;
use byteorder::ReadBytesExt;
const PUSH_U8: u8 = 1;
const CALL: u8 = 2;
const STOP: u8 = 3;

pub enum Value {
    None,
    U8(u8),
}

#[derive(Default)]
pub struct Env {
    pub stack: Vec<Value>,
}

pub enum Event {
    Stop,
    Call(Function),
}

pub fn exec(bc: &mut dyn Read, env: &mut Env) -> Result<Event> {
    loop {
        let Ok(c) = bc.read_u8() else { return Ok(Event::Stop)};
        match c {
            STOP => return Ok(Event::Stop),
            PUSH_U8 => env.stack.push(Value::U8(bc.read_u8()?)),
            CALL => {
                match env.stack.pop() {
                    Some(Value::U8(function)) => return Ok(Event::Call(function.try_into()?)),
                    None => return Err(anyhow!("stack underflow")),
                    _ => return Err(anyhow!("expect U8")),
                };
            }
            x => panic!("unhandled opcode {x:?}"),
        }
    }
}

#[derive(Default)]
pub struct Codegen {
    code: Vec<u8>,
}

impl Codegen {
    pub fn function_call(mut self, function: Function) -> Self {
        self.code.push(PUSH_U8);
        self.code.push(function.into());
        self.code.push(CALL);
        self
    }
    pub fn stop(mut self) -> Self {
        self.code.push(STOP);
        self
    }
    pub fn into_code(self) -> Vec<u8> {
        self.code
    }
}

#[test]
fn test_codegen() {
    let code = Codegen::default()
        .function_call(Function::ActionDie)
        .stop()
        .into_code();
    assert_eq!(code[..], [0x1, 0x7, 0x2, 0x3]);
}
#[test]
fn test_exec() {
    let mut env = Env::default();
    let bc = [0x1, 0x7, 0x2, 0x3];

    let mut c = std::io::Cursor::new(bc);
    let e = exec(&mut c, &mut env);
    assert!(matches!(e, Ok(Event::Call(Function::ActionDie))));

    let e = exec(&mut c, &mut env);
    assert!(matches!(e, Ok(Event::Stop)));
}
