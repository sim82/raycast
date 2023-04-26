use crate::{prelude::*, sprite::SpriteSceenSetup};
use rhai::{Engine, Scope, AST};

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
    ast: AST,
}
impl Entities {
    pub fn new() -> Self {
        let mut entities = Vec::new();
        entities.push(Entity { sprite_id: 1, state: 0 });

        let script_engine = ScriptEngine::new();
        let ast = script_engine.engine.compile(include_str!("test.rhai")).unwrap();
        Self {
            entities,
            script_engine,
            ast,
        }
    }
    pub fn update(&mut self) {
        for entity in &mut self.entities {
            let mut scope = Scope::new();
            scope.push("x", entity.sprite_id);

            entity.sprite_id = self
                .script_engine
                .engine
                .eval_ast_with_scope::<i32>(&mut scope, &self.ast)
                .unwrap() as i32;
        }
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
