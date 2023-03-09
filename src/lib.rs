use std::{
    fs::File,
    io::{BufRead, BufReader, Cursor, Seek, SeekFrom},
    path::Path,
};

use byteorder::{LittleEndian, ReadBytesExt};
use image::{ImageBuffer, Rgb};
use wl6::VswapFile;

pub mod wl6;

const TEX_SIZE: usize = wl6::TEX_SIZE;

// column first order
pub type Texture = [[u32; TEX_SIZE]; TEX_SIZE];

pub struct Resources {
    textures: Vec<Texture>,
    sprites: Vec<Texture>,
    fallback_texture: Texture,
}
impl Default for Resources {
    fn default() -> Self {
        Self {
            textures: Default::default(),
            sprites: Default::default(),
            fallback_texture: [[0x808080; TEX_SIZE]; TEX_SIZE],
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

    pub fn get_sprite(&self, id: i32) -> &Texture {
        if id >= 1 && (id as usize) <= self.sprites.len() {
            &self.sprites[(id - 1) as usize]
        } else {
            &self.fallback_texture
        }
    }

    pub fn load_textures<P: AsRef<Path>>(list: P) -> Resources {
        let textures = if let Ok(f) = File::open(list) {
            BufReader::new(f)
                .lines()
                .filter_map(|line| line.ok())
                .filter_map(|name| image::open(name).map(|tex| tex.into_rgb8()).ok())
                .map(|image| {
                    let mut texture = [[0; TEX_SIZE]; TEX_SIZE];
                    for (x, col) in texture.iter_mut().enumerate() {
                        for (y, p) in col.iter_mut().enumerate() {
                            let c = image.get_pixel(x as u32, y as u32);
                            *p = (c.0[0] as u32) << 16 | (c.0[1] as u32) << 8 | (c.0[2] as u32)
                        }
                    }
                    texture
                })
                .collect()
        } else {
            Vec::new()
        };

        Resources {
            textures,
            ..Default::default()
        }
    }

    pub fn load_wl6<P: AsRef<Path>>(name: P) -> Resources {
        let mut vs = VswapFile::open(name);

        let mut textures = Vec::new();

        for i in 0..vs.num_walls {
            textures.push(wl6::wall_chunk_to_texture(
                &vs.read_chunk(wl6::ChunkId::Wall(i as usize)),
            ));
        }
        println!("textures: {}", textures.len());
        let mut sprites = Vec::new();

        for i in 0..vs.num_sprites {
            // println!("sprite {}", i);
            sprites.push(wl6::sprite_chunk_to_texture(
                &vs.read_chunk(wl6::ChunkId::Sprite(i as usize)),
            ));
        }

        Resources {
            textures,
            sprites,
            ..Default::default()
        }
    }
}

pub mod palette;
