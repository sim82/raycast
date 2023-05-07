use lazy_static::lazy_static;
use state_bc::ExecImage;
use std::path::Path;

pub mod block_map;
pub mod door;
pub mod draw;
pub mod enemy;
pub mod fa;
pub mod font;
pub mod fp16;
pub mod hud;
pub mod mainloop;
pub mod map;
pub mod map_dynamic;
pub mod player;
pub mod render;
pub mod sprite;
pub mod thing;
pub mod thing_def;
pub mod weapon;
pub mod wl6;

// the only reason for the state_bc crate is for the compiler to be usable in build.rs.
// treat it as part of the main crate (i.e. this project is not intended to become so large it needs further modularization...)
pub use state_bc;
pub use state_bc::ms;
pub use state_bc::Result;

pub mod prelude {

    pub use crate::{
        block_map::BlockMap,
        door::Door,
        draw::Draw,
        enemy::Enemy,
        fa::{
            fa_cos, fa_cot, fa_fix_angle, fa_sin, fa_tan, FA_FRAC_PI_2, FA_PI, FA_PI_FRAC_PI_2,
            FA_SCALEF, FA_STEPS, FA_TAU, PIS_IN_180, QUADRANT_1, QUADRANT_2, QUADRANT_3,
            QUADRANT_4, TAN_CLAMP,
        },
        font::{draw_char8x8, draw_string8x8},
        fp16::{
            Fp16, FP16_F, FP16_FOUR, FP16_FRAC_128, FP16_FRAC_64, FP16_HALF, FP16_ONE, FP16_SCALE,
            FP16_ZERO,
        },
        hud,
        mainloop::{InputState, Mainloop, SpawnInfo},
        map::{bresenham_trace, DoorType, Map, MapTile, PlaneOrientation, MAP_SIZE},
        map_dynamic::{DoorAction, DoorState, MapDynamic, PushwallAction, PushwallState},
        ms,
        ms::endian::{ReadExt, WriteExt},
        player::{Player, PlayerVel},
        randu8,
        render::{self, COL_ANGLE},
        sprite::{self, Directionality, SpriteDef, SpriteIndex},
        state_bc::opcode,
        state_bc::{Direction, EnemySpawnInfo, ExecCtx, Function, StateBc},
        thing::{Actor, Collectible, Item, Thing, Things},
        thing_def::{ThingDef, ThingDefs, ThingType},
        weapon::{Weapon, WeaponType},
        Resources, Result, HALF_HEIGHT, HEIGHT, IMG_WL6, MID, VIEW_HEIGHT, WIDTH,
    };
}

pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 200;

pub const VIEW_HEIGHT: i32 = 160;
pub const MID: i32 = VIEW_HEIGHT / 2;
pub const HALF_HEIGHT: i32 = VIEW_HEIGHT / 2;

const TEX_SIZE: usize = wl6::TEX_SIZE;

// column first order
pub type Texture = [[u8; TEX_SIZE]; TEX_SIZE];
use wl6::{SpritePosts, VswapFile};

pub struct Resources {
    textures: Vec<Texture>,
    sprites: Vec<Texture>,
    sprite_posts: Vec<SpritePosts>,
    fallback_texture: Texture,
}
impl Default for Resources {
    fn default() -> Self {
        Self {
            fallback_texture: [[0x9; TEX_SIZE]; TEX_SIZE],
            textures: Default::default(),
            sprites: Default::default(),
            sprite_posts: Default::default(),
        }
    }
}

impl Resources {
    pub fn get_texture(&self, id: i32) -> &Texture {
        if id >= 0 && (id as usize) <= self.textures.len() {
            &self.textures[id as usize]
        } else {
            &self.fallback_texture
        }
    }

    pub fn get_sprite_as_texture(&self, id: i32) -> &Texture {
        if id >= 1 && (id as usize) <= self.sprites.len() {
            &self.sprites[(id - 1) as usize]
        } else {
            &self.fallback_texture
        }
    }

    pub fn get_sprite(&self, id: i32) -> &SpritePosts {
        if id >= 1 && (id as usize) <= self.sprites.len() {
            &self.sprite_posts[(id - 1) as usize]
        } else {
            panic!("unknown sprite {id}");
        }
    }

    // pub fn load_textures<P: AsRef<Path>>(list: P) -> Resources {
    // let textures = if let Ok(f) = File::open(list) {
    //     BufReader::new(f)
    //         .lines()
    //         .filter_map(|line| line.ok())
    //         .filter_map(|name| image::open(name).map(|tex| tex.into_rgb8()).ok())
    //         .map(|image| {
    //             let mut texture = [[0; TEX_SIZE]; TEX_SIZE];
    //             for (x, col) in texture.iter_mut().enumerate() {
    //                 for (y, p) in col.iter_mut().enumerate() {
    //                     let c = image.get_pixel(x as u32, y as u32);
    //                     *p = (c.0[0] as u32) << 16 | (c.0[1] as u32) << 8 | (c.0[2] as u32)
    //                 }
    //             }
    //             texture
    //         })
    //         .collect()
    // } else {
    //     Vec::new()
    // };

    // Resources {
    //     textures,
    //     ..Default::default()
    // }
    // }

    pub fn load_wl6<P: AsRef<Path>>(name: P) -> Resources {
        let mut vs = VswapFile::open(name);

        let mut textures = Vec::new();

        for i in 0..vs.num_walls {
            textures.push(wl6::wall_chunk_to_texture(
                &vs.read_chunk(wl6::ChunkId::Wall(i as usize)),
            ));
        }
        let mut sprites = Vec::new();
        for i in 0..vs.num_sprites {
            // println!("sprite {}", i);
            sprites.push(wl6::sprite_chunk_to_texture(
                &vs.read_chunk(wl6::ChunkId::Sprite(i as usize)),
            ));
        }

        let sprite_posts = (0..vs.num_sprites as usize)
            .map(|i| wl6::sprite_chunk_to_posts(&vs.read_chunk(wl6::ChunkId::Sprite(i))))
            .collect();

        Resources {
            textures,
            sprites,
            sprite_posts,
            ..Default::default()
        }
    }
}

pub mod palette;

const WL6_IMAGE: &[u8] = include_bytes!("out.img");
// const WL6_SPAWN_INFO: &[u8] = include_bytes!("out.spawn");

lazy_static! {
    pub static ref IMG_WL6: ExecImage = ExecImage::from_bytes(WL6_IMAGE).unwrap();
    // pub static ref SPAWN_INFO_WL6: SpawnInfos = SpawnInfos::from_bytes(WL6_SPAWN_INFO).unwrap();
    // pub static ref RNG: std::sync::Mutex<oorandom::Rand32> = std::sync::Mutex::new(oorandom::Rand32::new(4711));
}

thread_local! {
    pub static RNG: std::cell::RefCell<oorandom::Rand32>  = std::cell::RefCell::new(oorandom::Rand32::new(4711));
}
pub fn randu8() -> u8 {
    let v = RNG.with(|r| r.borrow_mut().rand_u32().to_ne_bytes());
    v[0] ^ v[1] ^ v[2] ^ v[3] // TODO: is this smart?
}
