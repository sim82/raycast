use std::collections::HashSet;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::prelude::*;

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

        match self.exec_ctx.state.think {
            Function::ThinkStand if trigger => {
                self.exec_ctx
                    .jump_label("door::open")
                    .unwrap_or_else(|err| panic!("failed to jump to state door::open: {err:?}"));
            }
            Function::ThinkPath if trigger => {
                self.exec_ctx
                    .jump_label("door::blocked")
                    .unwrap_or_else(|err| panic!("failed to jump to state door::close: {err:?}"));
            }
            Function::ThinkChase => self.open_f -= FP16_FRAC_64,
            Function::ThinkDogChase => self.open_f += FP16_FRAC_64,
            Function::ActionShoot if !blocked => {
                self.exec_ctx
                    .jump_label("door::close")
                    .unwrap_or_else(|err| panic!("failed to jump to state door::close: {err:?}"));
            }
            _ => (),
        }
    }
}

impl ms::Loadable for Door {
    fn read_from(r: &mut dyn std::io::Read) -> Result<Self> {
        let open_f = Fp16::read_from(r)?;
        let exec_ctx = ExecCtx::read_from(r, &IMG_WL6)?;

        let num_blockers = r.read_u32::<LittleEndian>()?;
        let mut blockers = HashSet::new();
        for _ in 0..num_blockers {
            blockers.insert(r.read_i32::<LittleEndian>()?);
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
        w.write_u32::<LittleEndian>(self.blockers.len() as u32)?;
        for blocker in &self.blockers {
            w.write_i32::<LittleEndian>(*blocker)?;
        }
        Ok(())
    }
}
