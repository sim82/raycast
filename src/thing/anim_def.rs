use super::EnemyType;
use lazy_static::lazy_static;

pub const HUMANOID_STAND: [i32; 1] = [0];
pub const HUMANOID_WALK: [i32; 4] = [8, 16, 24, 32];
pub const HUMANOID_PAIN: [i32; 2] = [40, 44];
pub const HUMANOID_DIE: [i32; 5] = [41, 42, 43, 45, 46];
pub const CANINE_STAND: [i32; 1] = [0]; // not really...
pub const CANINE_WALK: [i32; 4] = [0, 8, 16, 24];

lazy_static! {
    pub static ref BROWN_STAND: [i32; 1] = {
        let mut res = HUMANOID_STAND;
        res.iter_mut().for_each(|f| *f += EnemyType::Brown.sprite_offset());
        res
    };
    pub static ref BROWN_WALK: [i32; 4] = {
        let mut res = HUMANOID_WALK;
        res.iter_mut().for_each(|f| *f += EnemyType::Brown.sprite_offset());
        res
    };
    pub static ref BROWN_PAIN: [i32; 5] = {
        let mut res = HUMANOID_DIE;
        res.iter_mut().for_each(|f| *f += EnemyType::Brown.sprite_offset());
        res
    };
    pub static ref BLUE_STAND: [i32; 1] = {
        let mut res = HUMANOID_STAND;
        res.iter_mut().for_each(|f| *f += EnemyType::Blue.sprite_offset());
        res
    };
    pub static ref BLUE_WALK: [i32; 4] = {
        let mut res = HUMANOID_WALK;
        res.iter_mut().for_each(|f| *f += EnemyType::Blue.sprite_offset());
        res
    };
    pub static ref BLUE_PAIN: [i32; 2] = {
        let mut res = HUMANOID_PAIN;
        res.iter_mut().for_each(|f| *f += EnemyType::Blue.sprite_offset());
        res
    };
    pub static ref WHITE_STAND: [i32; 1] = {
        let mut res = HUMANOID_STAND;
        res.iter_mut().for_each(|f| *f += EnemyType::White.sprite_offset());
        res
    };
    pub static ref WHITE_WALK: [i32; 4] = {
        let mut res = HUMANOID_WALK;
        res.iter_mut().for_each(|f| *f += EnemyType::White.sprite_offset());
        res
    };
    pub static ref WHITE_PAIN: [i32; 2] = {
        let mut res = HUMANOID_PAIN;
        res.iter_mut().for_each(|f| *f += EnemyType::White.sprite_offset());
        res
    };
    pub static ref WOOF_STAND: [i32; 1] = {
        let mut res = CANINE_STAND;
        res.iter_mut().for_each(|f| *f += EnemyType::Woof.sprite_offset());
        res
    };
    pub static ref WOOF_WALK: [i32; 4] = {
        let mut res = CANINE_WALK;
        res.iter_mut().for_each(|f| *f += EnemyType::Woof.sprite_offset());
        res
    };
    pub static ref ROTTEN_STAND: [i32; 1] = {
        let mut res = HUMANOID_STAND;
        res.iter_mut().for_each(|f| *f += EnemyType::Rotten.sprite_offset());
        res
    };
    pub static ref ROTTEN_WALK: [i32; 4] = {
        let mut res = HUMANOID_WALK;
        res.iter_mut().for_each(|f| *f += EnemyType::Rotten.sprite_offset());
        res
    };
}
