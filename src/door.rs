use crate::prelude::*;
use anyhow::anyhow;
use state_bc::opcode::Value;
use std::{collections::HashSet, io::Cursor};

impl From<Fp16> for opcode::Value {
    fn from(value: Fp16) -> Self {
        opcode::Value::I32(value.v)
    }
}
impl TryFrom<opcode::Value> for Fp16 {
    type Error = anyhow::Error;

    fn try_from(value: opcode::Value) -> Result<Self> {
        match value {
            opcode::Value::I32(v) => Ok(Fp16 { v }),
            x => Err(anyhow!("cannot convert to Fp16: {x:?}")),
        }
    }
}
pub struct Door {
    exec_ctx: ExecCtx,
    pub open_f: Fp16,
    pub blockers: HashSet<i32>,
}

impl Default for Door {
    fn default() -> Self {
        Self {
            exec_ctx: ExecCtx::new("door::closed", &IMG_WL6).unwrap(),
            open_f: FP16_ZERO,
            blockers: Default::default(),
        }
    }
}

impl Door {
    pub fn update(&mut self, trigger: bool, blocked: bool) {
        if self.exec_ctx.state.ticks <= 0 {
            self.exec_ctx.jump(self.exec_ctx.state.next).unwrap();
        }
        self.exec_ctx.state.ticks -= 1;

        // let function = self.exec_ctx.state.think;
        // self.dispatch_call(function, trigger, blocked);
        self.exec_code(self.exec_ctx.state.think_offs, trigger, blocked)
            .expect("exec_code failed");
    }

    fn exec_code(&mut self, code_offs: i32, trigger: bool, blocked: bool) -> Result<()> {
        let mut env = opcode::Env::default();
        let mut cursor = Cursor::new(&self.exec_ctx.image.code[code_offs as usize..]);
        loop {
            let state = opcode::exec(&mut cursor, &mut env).expect("error on bytecode exec");
            match state {
                opcode::Event::Stop => break,
                opcode::Event::Call(function) => self.dispatch_call(function, trigger, blocked),
                opcode::Event::Trap => match env.stack.pop() {
                    Some(Value::U8(0)) => env.stack.push(self.open_f.into()),
                    Some(Value::U8(1)) => match env.stack.pop() {
                        Some(opcode::Value::I32(v)) => self.open_f.v = v,
                        Some(x) => return Err(anyhow!("unhandled opcode::Value {x:?}")),
                        None => return Err(anyhow!("stack underflow")),
                    },
                    Some(Value::U8(2)) => env.stack.push(Value::Bool(trigger)),
                    Some(Value::U8(3)) => env.stack.push(Value::Bool(blocked)),
                    x => panic!("unexpected trap code {x:?}"),
                },
                opcode::Event::GoState(offs) => {
                    self.exec_ctx.jump(offs);
                    break;
                } // x => todo!("unhandled opcode::Event {x:?}"),
            }
        }
        Ok(())
    }
    fn dispatch_call(&mut self, function: Function, trigger: bool, blocked: bool) {
        match function {
            // Function::ThinkStand => {
            //     self.exec_ctx
            //         .jump_label("door::open")
            //         .unwrap_or_else(|err| panic!("failed to jump to state door::open: {err:?}"));
            // }
            // Function::ThinkPath => {
            //     self.exec_ctx
            //         .jump_label("door::blocked")
            //         .unwrap_or_else(|err| panic!("failed to jump to state door::close: {err:?}"));
            // }
            // Function::ActionShoot => {
            //     self.exec_ctx
            //         .jump_label("door::close")
            //         .unwrap_or_else(|err| panic!("failed to jump to state door::close: {err:?}"));
            // }
            _ => todo!(),
        }
    }
}

impl ms::Loadable for Door {
    fn read_from(r: &mut dyn std::io::Read) -> Result<Self> {
        let open_f = Fp16::read_from(r)?;
        let exec_ctx = ExecCtx::read_from(r, &IMG_WL6)?;

        let num_blockers = r.readu32()?;
        let mut blockers = HashSet::new();
        for _ in 0..num_blockers {
            blockers.insert(r.readi32()?);
        }
        Ok(Self {
            open_f,
            exec_ctx,
            blockers,
        })
    }
}

impl ms::Writable for Door {
    fn write(&self, w: &mut dyn std::io::Write) -> Result<()> {
        self.open_f.write(w)?;
        self.exec_ctx.write(w)?;
        w.writeu32(self.blockers.len() as u32)?;
        for blocker in &self.blockers {
            w.writei32(*blocker)?;
        }
        Ok(())
    }
}
