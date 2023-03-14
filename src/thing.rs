use crate::prelude::*;
use byteorder::{ReadBytesExt, WriteBytesExt};

pub mod anim_def;

#[derive(Clone, Copy, Debug)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

#[derive(Clone, Copy, Debug)]
pub enum EnemyType {
    Brown,
    White,
    Blue,
    Woof,
    Rotten,
}

pub struct AnimationFrames {
    pub frames: &'static [i32],
    pub direction_stride: i32,
}

const START_BROWN: i32 = 51;
const NUM_HUMANOID: i32 = 49;
const NUM_CANINE: i32 = 39;
const NUM_ROTTOID: i32 = 51;

impl EnemyType {
    pub fn sprite_offset(&self) -> i32 {
        match self {
            EnemyType::Brown => START_BROWN,
            EnemyType::White => START_BROWN + 2 * NUM_HUMANOID + NUM_CANINE + NUM_ROTTOID,
            EnemyType::Blue => START_BROWN + NUM_HUMANOID + NUM_CANINE,
            EnemyType::Woof => START_BROWN + NUM_HUMANOID,
            EnemyType::Rotten => START_BROWN + 2 * NUM_HUMANOID + NUM_CANINE,
        }
    }

    pub fn animation_frames(&self, phase: AnimationPhase) -> AnimationFrames {
        match self {
            EnemyType::Brown => match phase {
                AnimationPhase::Stand => AnimationFrames {
                    frames: &*anim_def::BROWN_STAND,
                    direction_stride: 1,
                },
                AnimationPhase::Walk => AnimationFrames {
                    frames: &*anim_def::BROWN_WALK,
                    direction_stride: 1,
                },
                AnimationPhase::Pain => todo!(),
                AnimationPhase::Die => todo!(),
                AnimationPhase::Dead => todo!(),
                AnimationPhase::Shoot => todo!(),
            },
            EnemyType::White => match phase {
                AnimationPhase::Stand => AnimationFrames {
                    frames: &*anim_def::WHITE_STAND,
                    direction_stride: 1,
                },
                AnimationPhase::Walk => AnimationFrames {
                    frames: &*anim_def::WHITE_WALK,
                    direction_stride: 1,
                },
                AnimationPhase::Pain => todo!(),
                AnimationPhase::Die => todo!(),
                AnimationPhase::Dead => todo!(),
                AnimationPhase::Shoot => todo!(),
            },
            EnemyType::Blue => match phase {
                AnimationPhase::Stand => AnimationFrames {
                    frames: &*anim_def::BLUE_STAND,
                    direction_stride: 1,
                },
                AnimationPhase::Walk => AnimationFrames {
                    frames: &*anim_def::BLUE_WALK,
                    direction_stride: 1,
                },
                AnimationPhase::Pain => todo!(),
                AnimationPhase::Die => todo!(),
                AnimationPhase::Dead => todo!(),
                AnimationPhase::Shoot => todo!(),
            },
            EnemyType::Woof => match phase {
                AnimationPhase::Stand => AnimationFrames {
                    frames: &*anim_def::WOOF_STAND,
                    direction_stride: 1,
                },
                AnimationPhase::Walk => AnimationFrames {
                    frames: &*anim_def::WOOF_WALK,
                    direction_stride: 1,
                },
                AnimationPhase::Pain => todo!(),
                AnimationPhase::Die => todo!(),
                AnimationPhase::Dead => todo!(),
                AnimationPhase::Shoot => todo!(),
            },
            EnemyType::Rotten => match phase {
                AnimationPhase::Stand => AnimationFrames {
                    frames: &*anim_def::ROTTEN_STAND,
                    direction_stride: 1,
                },
                AnimationPhase::Walk => AnimationFrames {
                    frames: &*anim_def::ROTTEN_WALK,
                    direction_stride: 1,
                },
                AnimationPhase::Pain => todo!(),
                AnimationPhase::Die => todo!(),
                AnimationPhase::Dead => todo!(),
                AnimationPhase::Shoot => todo!(),
            },
        }
    }
}

pub enum AnimationPhase {
    Stand,
    Walk,
    Pain,
    Die,
    Dead,
    Shoot,
}

pub enum AnimationState {
    Stand,
    Walk1,
    Walk2,
    Walk3,
    Walk4,
}

// trait AnimationControl {
//     fn sprite_offset(&self) -> i32;

// }

impl AnimationState {
    pub fn sprite_offset(&self) -> i32 {
        match self {
            AnimationState::Stand => 0,
            AnimationState::Walk1 => 8,
            AnimationState::Walk2 => 16,
            AnimationState::Walk3 => 24,
            AnimationState::Walk4 => 32,
        }
    }
    pub fn advance_animation(&self) -> Self {
        match self {
            AnimationState::Stand => AnimationState::Stand,
            AnimationState::Walk1 => AnimationState::Walk2,
            AnimationState::Walk2 => AnimationState::Walk3,
            AnimationState::Walk3 => AnimationState::Walk4,
            AnimationState::Walk4 => AnimationState::Walk1,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum EnemyState {
    Standing,
    Patrolling,
}

#[derive(Clone, Copy, Debug)]
pub enum ThingType {
    PlayerStart(i32),
    Enemy(Direction, Difficulty, EnemyType, EnemyState),
    Prop(i32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    N,
    E,
    S,
    W,
}

impl ms::Writable for Direction {
    fn write(&self, w: &mut dyn std::io::Write) {
        match self {
            Direction::N => w.write_u8(0).unwrap(),
            Direction::E => w.write_u8(1).unwrap(),
            Direction::S => w.write_u8(2).unwrap(),
            Direction::W => w.write_u8(3).unwrap(),
        }
    }
}

impl ms::Loadable for Direction {
    fn read_from(r: &mut dyn std::io::Read) -> Self {
        match r.read_u8().unwrap() {
            0 => Direction::N,
            1 => Direction::E,
            2 => Direction::S,
            3 => Direction::W,
            _ => panic!(),
        }
    }
}

impl Direction {
    pub fn angle(&self) -> i32 {
        match &self {
            Direction::N => FA_PI_FRAC_PI_2,
            Direction::E => 0,
            Direction::S => FA_FRAC_PI_2,
            Direction::W => FA_PI,
        }
    }
    pub fn sprite_offset(&self) -> i32 {
        match &self {
            Direction::N => 0,
            Direction::E => 2,
            Direction::S => 4,
            Direction::W => 6,
        }
    }
    pub fn tile_offset(&self) -> (i32, i32) {
        match self {
            Direction::N => (0, -1),
            Direction::E => (1, 0),
            Direction::S => (0, 1),
            Direction::W => (-1, 0),
        }
    }
}

pub struct Thing {
    pub thing_type: ThingType,
    pub x: Fp16,
    pub y: Fp16,
}

pub struct Things {
    pub things: Vec<Thing>,
    anim_timeout: i32,
}

impl Things {
    pub fn from_map_plane(plane: &[u16]) -> Self {
        let mut plane_iter = plane.iter();
        let mut things = Vec::new();

        for y in 0..64 {
            for x in 0..64 {
                let c = plane_iter.next().unwrap();

                let thing_type = if let Some(enemy) = Things::map_enemy(*c) {
                    enemy
                } else {
                    match *c {
                        19 => ThingType::PlayerStart(FA_PI_FRAC_PI_2), // NORTH means facing -y
                        20 => ThingType::PlayerStart(0),
                        21 => ThingType::PlayerStart(FA_FRAC_PI_2),
                        22 => ThingType::PlayerStart(FA_PI),
                        23..=71 => ThingType::Prop((c - 22 + 2) as i32),
                        _ => continue,
                    }
                };

                things.push(Thing {
                    thing_type,
                    x: FP16_HALF + x.into(),
                    y: FP16_HALF + y.into(),
                });
            }
        }
        Things {
            things,
            anim_timeout: 30,
        }
    }

    fn oa(o: u16) -> Direction {
        match o % 4 {
            0 => Direction::N,
            1 => Direction::E,
            2 => Direction::S,
            3 => Direction::W,
            _ => panic!(),
        }
    }
    fn os(o: u16) -> EnemyState {
        if o <= 3 {
            EnemyState::Standing
        } else {
            EnemyState::Patrolling
        }
    }

#[rustfmt::skip]
    fn map_enemy(t: u16) -> Option<ThingType> {
    Some(match t {
        // easy
        108..=115 => ThingType::Enemy(Things::oa(t - 108), Difficulty::Easy, EnemyType::Brown, Things::os(t - 108)),
        116..=123 => ThingType::Enemy(Things::oa(t - 116), Difficulty::Easy, EnemyType::White, Things::os(t - 116)),
        126..=133 => ThingType::Enemy(Things::oa(t - 126), Difficulty::Easy, EnemyType::Blue, Things::os(t - 126)),
        134..=141 => ThingType::Enemy(Things::oa(t - 134), Difficulty::Easy, EnemyType::Woof, Things::os(t - 134)),
        216..=223 => ThingType::Enemy(Things::oa(t - 134), Difficulty::Easy, EnemyType::Rotten, Things::os(t - 216)),
        // medium
        144..=151 => ThingType::Enemy(Things::oa(t - 144), Difficulty::Medium, EnemyType::Brown, Things::os(t - 144)),
        152..=159 => ThingType::Enemy(Things::oa(t - 152), Difficulty::Medium, EnemyType::White, Things::os(t - 152)),
        162..=169 => ThingType::Enemy(Things::oa(t - 162), Difficulty::Medium, EnemyType::Blue, Things::os(t - 162)),
        170..=177 => ThingType::Enemy(Things::oa(t - 170), Difficulty::Medium, EnemyType::Woof, Things::os(t - 170)),
        234..=241 => ThingType::Enemy(Things::oa(t - 234), Difficulty::Medium, EnemyType::Rotten, Things::os(t - 234)),
        // hard
        180..=187 => ThingType::Enemy(Things::oa(t - 180), Difficulty::Hard, EnemyType::Brown, Things::os(t - 180)),
        188..=195 => ThingType::Enemy(Things::oa(t - 188), Difficulty::Hard, EnemyType::White, Things::os(t - 188)),
        198..=205 => ThingType::Enemy(Things::oa(t - 198), Difficulty::Hard, EnemyType::Blue, Things::os(t - 198)),
        206..=213 => ThingType::Enemy(Things::oa(t - 206), Difficulty::Hard, EnemyType::Woof, Things::os(t - 206)),
        252..=259 => ThingType::Enemy(Things::oa(t - 252), Difficulty::Hard, EnemyType::Rotten, Things::os(t - 252)),
        _ => return None,
    })
}

    pub fn get_player_start(&self) -> Option<(Fp16, Fp16, i32)> {
        for thing in &self.things {
            match thing.thing_type {
                ThingType::PlayerStart(rot) => return Some((thing.x, thing.y, rot)),
                _ => continue,
            }
        }
        None
    }

    // pub fn get_sprites(&self) -> Vec<SpriteDef> {
    //     self.things
    //         .iter()
    //         .filter_map(|thing| match &thing.thing_type {
    //             ThingType::Enemy(direction, _difficulty, enemy_type, _state) => {
    //                 let id = enemy_type.sprite_offset()
    //                 + thing.animation_state.sprite_offset()
    //                 /*+ direction.sprite_offset()*/;
    //                 Some(SpriteDef {
    //                     id,
    //                     x: thing.x,
    //                     y: thing.y,
    //                     directionality: Directionality::Direction(*direction),
    //                 })
    //             }
    //             ThingType::Prop(id) => Some(SpriteDef {
    //                 id: *id,
    //                 x: thing.x,
    //                 y: thing.y,
    //                 directionality: Directionality::Undirectional,
    //             }),

    //             _ => None,
    //         })
    //         .collect()
    // }

    // pub fn update(&mut self) {
    //     self.anim_timeout -= 1;
    //     if self.anim_timeout > 0 {
    //         return;
    //     }
    //     self.anim_timeout = 10;

    //     for thing in &mut self.things {
    //         thing.animation_state = thing.animation_state.advance_animation();
    //     }
    // }
}

pub struct ThingDyn {
    pub x: Fp16,
    pub y: Fp16,
    pub animation_frames: Option<AnimationFrames>,
    pub sprite_index: i32,
    pub anim_index: i32,
    pub static_index: usize,
}

pub struct ThingsDyn {
    pub things: Vec<ThingDyn>,
    pub anim_timeout: i32,
}

impl ThingsDyn {
    pub fn from_things(thing_defs: &Things) -> Self {
        let mut things = Vec::new();

        for (i, thing_def) in thing_defs.things.iter().enumerate() {
            match thing_def.thing_type {
                ThingType::Enemy(_, _, enemy_type, _) => things.push(ThingDyn {
                    x: thing_def.x,
                    y: thing_def.y,
                    animation_frames: Some(enemy_type.animation_frames(AnimationPhase::Walk)),
                    sprite_index: 0,
                    anim_index: 0,
                    static_index: i,
                }),
                ThingType::Prop(sprite_index) => things.push(ThingDyn {
                    x: thing_def.x, // fixme: do not need dynamic x/y for props
                    y: thing_def.y,
                    animation_frames: None,
                    sprite_index,
                    anim_index: 0,
                    static_index: i,
                }),
                ThingType::PlayerStart(_) => (),
            }
        }

        ThingsDyn {
            things,
            anim_timeout: 0,
        }
    }

    pub fn update(&mut self) {
        self.anim_timeout -= 1;
        if self.anim_timeout > 0 {
            return;
        }
        self.anim_timeout = 10;

        for thing in &mut self.things {
            thing.anim_index += 1;

            if let Some(frames) = &thing.animation_frames {
                thing.sprite_index =
                    frames.frames[(thing.anim_index as usize) % frames.frames.len()];
            }
        }
    }
    pub fn get_sprites(&self, thing_defs: &Things) -> Vec<SpriteDef> {
        self.things
            .iter()
            .filter_map(
                |thing| match &thing_defs.things[thing.static_index].thing_type {
                    ThingType::Enemy(direction, _difficulty, _enemy_type, _state) => {
                        // let id =
                        /*+ direction.sprite_offset()*/
                        Some(SpriteDef {
                            id: thing.sprite_index,
                            x: thing.x,
                            y: thing.y,
                            directionality: Directionality::Direction(*direction),
                        })
                    }
                    ThingType::Prop(id) => Some(SpriteDef {
                        id: *id,
                        x: thing.x,
                        y: thing.y,
                        directionality: Directionality::Undirectional,
                    }),

                    _ => None,
                },
            )
            .collect()
    }
}
