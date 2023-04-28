use std::io::Read;

use byteorder::ReadBytesExt;

use crate::Function;

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

pub fn exec(bc: &mut dyn Read, env: &mut Env) -> Event {
    loop {
        let Ok(c) = bc.read_u8() else { return Event::Stop};
        match c {
            STOP => return Event::Stop,
            PUSH_U8 => env.stack.push(Value::U8(
                bc.read_u8().expect("unexpected end of bytecode: PUSH_U8"),
            )),
            CALL => {
                let Value::U8(function) = env.stack.pop().expect("stack underflow") else { panic!("expect U8")};
                return Event::Call(function.try_into().expect("u8 -> Function failed."));
            }
            x => panic!("unhandled opcode {x:?}"),
        }
    }
}

#[test]
fn test_exec() {
    let mut env = Env::default();
    let bc = [0x1, 0x7, 0x2, 0x3];

    let mut c = std::io::Cursor::new(bc);
    let e = exec(&mut c, &mut env);
    assert!(matches!(e, Event::Call(Function::ActionDie)));

    let e = exec(&mut c, &mut env);
    assert!(matches!(e, Event::Stop));
}
