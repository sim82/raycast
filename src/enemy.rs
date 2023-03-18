use crate::{prelude::*, thing_def::EnemyType};

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
    (40, 10, think_nil, action_nil, 9, true),
    // 8
    (44, 10, think_nil, action_nil, 9, true),
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
