use byteorder::{ReadBytesExt, WriteBytesExt};

use crate::prelude::*;

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

    pub fn animation_frames(&self, phase: AnimationPhase) -> &'static [i32] {
        match self {
            EnemyType::Brown => match phase {
                AnimationPhase::Stand => &*anim_def::BROWN_STAND,
                AnimationPhase::Walk => &*anim_def::BROWN_WALK,
                AnimationPhase::Pain => &*anim_def::BROWN_PAIN,
                AnimationPhase::Die => &*anim_def::BROWN_DIE,
                AnimationPhase::Dead => todo!(),
                AnimationPhase::Shoot => todo!(),
            },
            EnemyType::White => match phase {
                AnimationPhase::Stand => &*anim_def::WHITE_STAND,
                AnimationPhase::Walk => &*anim_def::WHITE_WALK,
                AnimationPhase::Pain => &*anim_def::WHITE_PAIN,
                AnimationPhase::Die => todo!(),
                AnimationPhase::Dead => todo!(),
                AnimationPhase::Shoot => todo!(),
            },
            EnemyType::Blue => match phase {
                AnimationPhase::Stand => &*anim_def::BLUE_STAND,
                AnimationPhase::Walk => &*anim_def::BLUE_WALK,
                AnimationPhase::Pain => &*anim_def::BLUE_PAIN,
                AnimationPhase::Die => todo!(),
                AnimationPhase::Dead => todo!(),
                AnimationPhase::Shoot => todo!(),
            },
            EnemyType::Woof => match phase {
                AnimationPhase::Stand => &*anim_def::WOOF_STAND,
                AnimationPhase::Walk => &*anim_def::WOOF_WALK,
                AnimationPhase::Pain =>
                /*&*anim_def::WOOF_PAIN*/
                {
                    todo!()
                }
                AnimationPhase::Die => todo!(),
                AnimationPhase::Dead => todo!(),
                AnimationPhase::Shoot => todo!(),
            },
            EnemyType::Rotten => match phase {
                AnimationPhase::Stand => &*anim_def::ROTTEN_STAND,
                AnimationPhase::Walk => &*anim_def::BLUE_PAIN,
                AnimationPhase::Pain =>
                /*&*anim_def::ROTTEN_PAIN*/
                {
                    todo!()
                }
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

#[derive(Debug, Clone, Copy)]
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
    fn write(&self, w: &mut dyn std::io::Write) -> Result<()> {
        match self {
            Direction::N => w.write_u8(0)?,
            Direction::E => w.write_u8(1)?,
            Direction::S => w.write_u8(2)?,
            Direction::W => w.write_u8(3)?,
        }
        Ok(())
    }
}

impl ms::Loadable for Direction {
    fn read_from(r: &mut dyn std::io::Read) -> Result<Self> {
        Ok(match r.read_u8()? {
            0 => Direction::N,
            1 => Direction::E,
            2 => Direction::S,
            3 => Direction::W,
            _ => panic!(),
        })
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

#[derive(Debug)]
pub struct ThingDef {
    pub thing_type: ThingType,
    pub x: Fp16,
    pub y: Fp16,
}

pub struct ThingDefs {
    pub thing_defs: Vec<ThingDef>,
}

impl ThingDefs {
    pub fn from_map_plane(plane: &[u16]) -> Self {
        let mut plane_iter = plane.iter();
        let mut thing_defs = Vec::new();

        for y in 0..64 {
            for x in 0..64 {
                let c = plane_iter.next().unwrap();

                let thing_type = if let Some(enemy) = ThingDefs::map_enemy(*c) {
                    enemy
                } else {
                    match *c {
                        19 => ThingType::PlayerStart(FA_PI_FRAC_PI_2), // NORTH means facing -y
                        20 => ThingType::PlayerStart(0),
                        21 => ThingType::PlayerStart(FA_FRAC_PI_2),
                        22 => ThingType::PlayerStart(FA_PI),
                        23..=71 => ThingType::Prop((*c) as i32),
                        _ => continue,
                    }
                };

                thing_defs.push(ThingDef {
                    thing_type,
                    x: FP16_HALF + x.into(),
                    y: FP16_HALF + y.into(),
                });
            }
        }
        ThingDefs { thing_defs }
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
        108..=115 => ThingType::Enemy(ThingDefs::oa(t - 108), Difficulty::Easy, EnemyType::Brown, ThingDefs::os(t - 108)),
        116..=123 => ThingType::Enemy(ThingDefs::oa(t - 116), Difficulty::Easy, EnemyType::White, ThingDefs::os(t - 116)),
        126..=133 => ThingType::Enemy(ThingDefs::oa(t - 126), Difficulty::Easy, EnemyType::Blue, ThingDefs::os(t - 126)),
        134..=141 => ThingType::Enemy(ThingDefs::oa(t - 134), Difficulty::Easy, EnemyType::Woof, ThingDefs::os(t - 134)),
        216..=223 => ThingType::Enemy(ThingDefs::oa(t - 134), Difficulty::Easy, EnemyType::Rotten, ThingDefs::os(t - 216)),
        // medium
        144..=151 => ThingType::Enemy(ThingDefs::oa(t - 144), Difficulty::Medium, EnemyType::Brown, ThingDefs::os(t - 144)),
        152..=159 => ThingType::Enemy(ThingDefs::oa(t - 152), Difficulty::Medium, EnemyType::White, ThingDefs::os(t - 152)),
        162..=169 => ThingType::Enemy(ThingDefs::oa(t - 162), Difficulty::Medium, EnemyType::Blue, ThingDefs::os(t - 162)),
        170..=177 => ThingType::Enemy(ThingDefs::oa(t - 170), Difficulty::Medium, EnemyType::Woof, ThingDefs::os(t - 170)),
        234..=241 => ThingType::Enemy(ThingDefs::oa(t - 234), Difficulty::Medium, EnemyType::Rotten, ThingDefs::os(t - 234)),
        // hard
        180..=187 => ThingType::Enemy(ThingDefs::oa(t - 180), Difficulty::Hard, EnemyType::Brown, ThingDefs::os(t - 180)),
        188..=195 => ThingType::Enemy(ThingDefs::oa(t - 188), Difficulty::Hard, EnemyType::White, ThingDefs::os(t - 188)),
        198..=205 => ThingType::Enemy(ThingDefs::oa(t - 198), Difficulty::Hard, EnemyType::Blue, ThingDefs::os(t - 198)),
        206..=213 => ThingType::Enemy(ThingDefs::oa(t - 206), Difficulty::Hard, EnemyType::Woof, ThingDefs::os(t - 206)),
        252..=259 => ThingType::Enemy(ThingDefs::oa(t - 252), Difficulty::Hard, EnemyType::Rotten, ThingDefs::os(t - 252)),
        _ => return None,
    })
}

    pub fn get_player_start(&self) -> Option<(Fp16, Fp16, i32)> {
        for thing in &self.thing_defs {
            match thing.thing_type {
                ThingType::PlayerStart(rot) => return Some((thing.x, thing.y, rot)),
                _ => continue,
            }
        }
        None
    }
}
