use std::{
    fs::File,
    io::{BufRead, BufReader, Cursor, Seek, SeekFrom},
    path::Path,
};

use byteorder::{LittleEndian, ReadBytesExt};
use image::{ImageBuffer, Rgb};
use wl6::VswapFile;

pub mod wl6;

const TEX_SIZE: usize = 64;

// column first order
pub type Texture = [[u32; TEX_SIZE]; TEX_SIZE];

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

    pub fn load_wl6<P: AsRef<Path>>(name: P) -> Resources {
        let mut vs = VswapFile::open(name);

        let mut textures = Vec::new();

        for i in 0..vs.num_walls {
            textures.push(wall_chunk_to_texture(
                &vs.read_chunk(wl6::ChunkId::Wall(i as usize)),
            ));
        }

        for i in 0..vs.num_sprites {
            println!("sprite {}", i);
            textures.push(sprite_chunk_to_texture(
                &vs.read_chunk(wl6::ChunkId::Sprite(i as usize)),
            ));
        }

        Resources {
            textures,
            ..Default::default()
        }
    }
}

fn sprite_chunk_to_texture(buf: &[u8]) -> Texture {
    let mut cursor = Cursor::new(buf);
    // UInt16LE 	FirstCol: Index of leftmost non-empty column
    // UInt16LE 	LastCol: Index of rightmost non-empty column
    // UInt16LE[n] 	Offsets relative to beginning of chunk to the first post of each column between FirstCol and LastCol (n = LastCol - FirstCol + 1)
    // UInt8[?] 	Pixel pool: Palette indexes for all solid pixels of the sprite (size unknown when decoding)
    // UInt16[?] 	Array of values describing all posts in the sprite (size unknown when decoding)

    let mut texture = [[0; TEX_SIZE]; TEX_SIZE];
    let first_col = cursor.read_u16::<LittleEndian>().unwrap();
    let last_col = cursor.read_u16::<LittleEndian>().unwrap();
    // let n = (last_col - first_col) + 1;
    let offsets = (first_col..=last_col)
        .map(|_| cursor.read_u16::<LittleEndian>().unwrap())
        .collect::<Vec<_>>();
    let pixels = cursor.clone();
    for col_offset in offsets {
        println!("col start {}", col_offset);
        cursor.seek(SeekFrom::Start(col_offset as u64)).unwrap();
        loop {
            let end = cursor.read_u16::<LittleEndian>().unwrap();
            if end == 0 {
                break;
            }
            let _ = cursor.read_u16::<LittleEndian>().unwrap();
            let start = cursor.read_u16::<LittleEndian>().unwrap();

            println!("post: {} {}", start, end);
        }
    }
    texture
}

fn wall_chunk_to_texture(buf: &[u8]) -> Texture {
    let mut cursor = Cursor::new(buf);

    let mut texture = [[0; TEX_SIZE]; TEX_SIZE];
    for col in &mut texture {
        for c in col {
            *c = palette::PALETTE[cursor.read_u8().unwrap() as usize];
        }
    }
    texture
}

pub mod palette;
