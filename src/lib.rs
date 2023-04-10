pub use anyhow::Result;
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
pub mod ms;
pub mod player;
pub mod render;
pub mod sprite;
pub mod state_bc;
pub mod thing;
pub mod thing_def;
pub mod weapon;
pub mod wl6;

pub mod prelude {

    pub use crate::{
        block_map::BlockMap,
        door::Door,
        draw::Draw,
        enemy::Enemy,
        fa::{
            fa_cos, fa_cot, fa_fix_angle, fa_sin, fa_tan, COL_ANGLE, FA_FRAC_PI_2, FA_PI, FA_PI_FRAC_PI_2, FA_SCALEF,
            FA_STEPS, FA_TAU, PIS_IN_180, QUADRANT_1, QUADRANT_2, QUADRANT_3, QUADRANT_4, TAN_CLAMP,
        },
        font::{draw_char8x8, draw_string8x8},
        fp16::{Fp16, FP16_F, FP16_FOUR, FP16_FRAC_128, FP16_FRAC_64, FP16_HALF, FP16_ONE, FP16_SCALE, FP16_ZERO},
        hud,
        mainloop::{InputState, Mainloop, SpawnInfo},
        map::{bresenham_trace, DoorType, Map, MapTile, PlaneOrientation, MAP_SIZE},
        map_dynamic::{DoorAction, DoorState, MapDynamic, PushwallAction, PushwallState},
        ms,
        player::{Player, PlayerVel},
        render,
        sprite::{self, Directionality, SpriteDef, SpriteIndex},
        state_bc::{Action, EnemySpawnInfo, ExecCtx, StateBc, Think, SPAWN_INFO_WL6},
        thing::{Actor, Collectible, Item, Thing, Things},
        thing_def::{Direction, ThingDef, ThingDefs, ThingType},
        weapon::{Weapon, WeaponType},
        Resources, Result, HALF_HEIGHT, HEIGHT, MID, VIEW_HEIGHT, WIDTH,
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
