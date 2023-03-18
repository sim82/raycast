use crate::{prelude::*, thing_def::EnemyType};

type State = (i32, i32, fn(&mut Thing), fn(&mut Thing) -> (), usize, bool);
// const BROWN_STAND: [State; 1] = [(0, 0, think_stand, action_nil, 0, true)];
// const BROWN_WALK: [State; 6] = [
//     (8, 20, think_path, action_nil, 1, true),
//     (8, 5, think_nil, action_nil, 2, true),
//     (16, 15, think_path, action_nil, 3, true),
//     (24, 20, think_path, action_nil, 4, true),
//     (24, 5, think_nil, action_nil, 5, true),
//     (32, 15, think_path, action_nil, 0, true),
// ];
// const BROWN_PAIN: [State; 1] = [(40, 10, think_path, action_nil, 1, true)];

const HUMANOID_FRAMES: [State; 14] = [
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
    (40, 10, think_path, action_nil, 8, true),
    // 8
    (8, 20, think_chase, action_nil, 9, true),
    (8, 5, think_nil, action_nil, 10, true),
    (16, 15, think_chase, action_nil, 11, true),
    (24, 20, think_chase, action_nil, 12, true),
    (24, 5, think_nil, action_nil, 13, true),
    (32, 15, think_chase, action_nil, 8, true),
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
