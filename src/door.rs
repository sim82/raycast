use std::collections::HashSet;

use crate::prelude::*;

pub struct Door {
    exec_ctx: ExecCtx,
    open_f: Fp16,
    pub blockers: HashSet<i32>,
}

impl Default for Door {
    fn default() -> Self {
        Self {
            exec_ctx: ExecCtx::new("door::closed").unwrap(),
            open_f: FP16_ZERO,
            blockers: Default::default(),
        }
    }
}

impl Door {
    pub fn update(&mut self, trigger: bool, blocked: bool) {}
}
