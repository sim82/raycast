use std::io::Write;

use crate::{fa::FA_FRAC_PI_4, prelude::*};
use anyhow::anyhow;
use byteorder::{ReadBytesExt, WriteBytesExt};

#[derive(Clone, Copy, Debug)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

pub struct EnemyCapabilities {
    pub can_open_doors: bool,
}

pub fn get_capabilities_by_name(name: &str) -> EnemyCapabilities {
    EnemyCapabilities {
        can_open_doors: name != "furry",
    }
}

#[derive(Clone, Debug)]
pub enum ThingType {
    PlayerStart(i32),
    Enemy(EnemySpawnInfo),
    Prop(i32),
}

pub fn direction_angle(d: &Direction) -> i32 {
    match d {
        Direction::North => FA_PI_FRAC_PI_2,
        Direction::NorthEast => FA_PI_FRAC_PI_2 + FA_FRAC_PI_4,
        Direction::East => 0,
        Direction::SouthEast => FA_FRAC_PI_4,
        Direction::South => FA_FRAC_PI_2,
        Direction::SouthWest => FA_FRAC_PI_2 + FA_FRAC_PI_4,
        Direction::West => FA_PI,
        Direction::NorthWest => FA_PI + FA_FRAC_PI_4,
    }
}

#[derive(Debug)]
pub struct ThingDef {
    pub thing_type: ThingType,
    pub x: Fp16,
    pub y: Fp16,
}

impl ThingDef {
    pub fn from_map_id(c: i32, x: Fp16, y: Fp16) -> Option<ThingDef> {
        let thing_type = if let Some(spawn_info) = IMG_WL6.spawn_infos.find_spawn_info(c) {
            // println!("spawn info: {spawn_info:?}");
            ThingType::Enemy(spawn_info.clone())
        } else {
            match c {
                19 => ThingType::PlayerStart(FA_PI_FRAC_PI_2), // NORTH means facing -y
                20 => ThingType::PlayerStart(0),
                21 => ThingType::PlayerStart(FA_FRAC_PI_2),
                22 => ThingType::PlayerStart(FA_PI),
                23..=71 => ThingType::Prop(c),
                _ => return None,
            }
        };
        let thing_def = ThingDef { thing_type, x, y };
        Some(thing_def)
    }
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
                let c = *plane_iter.next().unwrap() as i32;
                let x = FP16_HALF + x.into();
                let y = FP16_HALF + y.into();

                let thing_def = match ThingDef::from_map_id(c, x, y) {
                    Some(value) => value,
                    None => continue,
                };
                thing_defs.push(thing_def);
            }
        }
        ThingDefs { thing_defs }
    }

    // keep for reference:
    // #[rustfmt::skip]
    //     fn map_enemy(t: u16) -> Option<ThingType> {
    //         Some(match t {
    //             // easy
    //             108..=115 => ThingType::Enemy(ThingDefs::oa(t - 108), Difficulty::Easy, EnemyType::Brown, ThingDefs::os(t - 108)),
    //             116..=123 => ThingType::Enemy(ThingDefs::oa(t - 116), Difficulty::Easy, EnemyType::White, ThingDefs::os(t - 116)),
    //             126..=133 => ThingType::Enemy(ThingDefs::oa(t - 126), Difficulty::Easy, EnemyType::Blue, ThingDefs::os(t - 126)),
    //             134..=141 => ThingType::Enemy(ThingDefs::oa(t - 134), Difficulty::Easy, EnemyType::Woof, ThingDefs::os(t - 134)),
    //             216..=223 => ThingType::Enemy(ThingDefs::oa(t - 216), Difficulty::Easy, EnemyType::Rotten, ThingDefs::os(t - 216)),
    //             // medium
    //             144..=151 => ThingType::Enemy(ThingDefs::oa(t - 144), Difficulty::Medium, EnemyType::Brown, ThingDefs::os(t - 144)),
    //             152..=159 => ThingType::Enemy(ThingDefs::oa(t - 152), Difficulty::Medium, EnemyType::White, ThingDefs::os(t - 152)),
    //             162..=169 => ThingType::Enemy(ThingDefs::oa(t - 162), Difficulty::Medium, EnemyType::Blue, ThingDefs::os(t - 162)),
    //             170..=177 => ThingType::Enemy(ThingDefs::oa(t - 170), Difficulty::Medium, EnemyType::Woof, ThingDefs::os(t - 170)),
    //             234..=241 => ThingType::Enemy(ThingDefs::oa(t - 234), Difficulty::Medium, EnemyType::Rotten, ThingDefs::os(t - 234)),
    //             // hard
    //             180..=187 => ThingType::Enemy(ThingDefs::oa(t - 180), Difficulty::Hard, EnemyType::Brown, ThingDefs::os(t - 180)),
    //             188..=195 => ThingType::Enemy(ThingDefs::oa(t - 188), Difficulty::Hard, EnemyType::White, ThingDefs::os(t - 188)),
    //             198..=205 => ThingType::Enemy(ThingDefs::oa(t - 198), Difficulty::Hard, EnemyType::Blue, ThingDefs::os(t - 198)),
    //             206..=213 => ThingType::Enemy(ThingDefs::oa(t - 206), Difficulty::Hard, EnemyType::Woof, ThingDefs::os(t - 206)),
    //             252..=259 => ThingType::Enemy(ThingDefs::oa(t - 252), Difficulty::Hard, EnemyType::Rotten, ThingDefs::os(t - 252)),

    //             0xd6 => ThingType::Enemy(Direction::South, Difficulty::Easy, EnemyType::Hans, EnemyState::Standing),
    //             _ => return None,
    //         })
    //     }

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
