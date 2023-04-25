use crate::{prelude::*, sprite::SpriteSceenSetup};
use rhai::Engine;

#[derive(Debug, Clone)]
struct Entity {
    sprite_id: i32,
    state: i32,
}

pub struct ScriptEngine {
    engine: Engine,
}

impl ScriptEngine {
    pub fn new() -> Self {
        let mut engine = Engine::new();
        engine.register_type_with_name::<Entity>("Entity");
        Self { engine }
    }
}
pub struct Entities {
    entities: Vec<Entity>,
    script_engine: ScriptEngine,
}
impl Entities {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            script_engine: ScriptEngine::new(),
        }
    }
    pub fn update(&mut self) {
        todo!()
    }
    pub fn get_sprites(&self) -> Vec<SpriteSceenSetup> {
        let mut ret = Vec::new();
        for entity in &self.entities {
            ret.push(SpriteSceenSetup {
                z: FP16_ZERO,
                screen_x: WIDTH as i32 / 2,
                id: entity.sprite_id,
                owner: 0,
            })
        }
        ret
    }
}
