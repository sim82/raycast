use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use image::{ImageBuffer, Rgb};

const TEX_SIZE: usize = 64;

// column first order
type Texture = [[u32; TEX_SIZE]; TEX_SIZE];

pub struct Resources {
    textures: Vec<Texture>,
    fallback_texture: Texture,
}
impl Default for Resources {
    fn default() -> Self {
        Self {
            textures: Default::default(),
            fallback_texture: [[0x808080; TEX_SIZE]; TEX_SIZE],
        }
    }
}

impl Resources {
    pub fn get_texture(&self, id: i32) -> &Texture {
        if id >= 1 && (id as usize) <= self.textures.len() {
            &self.textures[(id - 1) as usize]
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
}
