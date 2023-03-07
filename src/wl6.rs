use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    fs::File,
    io::{Cursor, Read, Seek, SeekFrom},
    path::Path,
};

use crate::palette;

pub const TEX_SIZE: usize = 64;

pub struct VswapFile {
    pub num_chunks: u16,
    pub num_walls: u16,
    pub num_sprites: u16,
    pub chunks: Vec<(u32, u16)>,
    f: File,
}

#[derive(Debug, Clone, Copy)]
pub enum ChunkId {
    Wall(usize),
    Sprite(usize),
    Sound(usize),
}

impl VswapFile {
    pub fn open<P: AsRef<Path>>(name: P) -> VswapFile {
        let mut f = File::open("vswap.wl6").unwrap();
        let num_chunks = f.read_u16::<LittleEndian>().unwrap();
        let last_wall = f.read_u16::<LittleEndian>().unwrap();
        let last_sprite = f.read_u16::<LittleEndian>().unwrap();
        let mut offs = Vec::new();
        let mut size = Vec::new();

        for _ in 0..num_chunks {
            offs.push(f.read_u32::<LittleEndian>().unwrap());
        }
        for _ in 0..num_chunks {
            size.push(f.read_u16::<LittleEndian>().unwrap());
        }

        let chunks = offs
            .iter()
            .zip(size.iter())
            .map(|(offs, size)| (*offs, *size))
            .collect();

        VswapFile {
            num_chunks,
            num_walls: last_wall,
            num_sprites: last_sprite - last_wall,
            chunks,
            f,
        }
    }

    pub fn read_chunk(&mut self, chunk: ChunkId) -> Vec<u8> {
        let chunk_index = match chunk {
            ChunkId::Wall(id) => id,
            ChunkId::Sprite(id) => self.num_walls as usize + id,
            ChunkId::Sound(id) => (self.num_walls + self.num_sprites) as usize + id,
        };

        let (offs, size) = self.chunks[chunk_index];
        self.f.seek(std::io::SeekFrom::Start(offs as u64)).unwrap();

        let mut buf = vec![0u8; size as usize];
        self.f.read_exact(&mut buf[..]).unwrap();

        buf
    }
}

pub fn sprite_chunk_to_texture(buf: &[u8]) -> [[u32; TEX_SIZE]; TEX_SIZE] {
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
    let mut pixels = cursor.clone();
    for (i, col_offset) in offsets.iter().enumerate() {
        // println!("col start {}", col_offset);
        cursor.seek(SeekFrom::Start(*col_offset as u64)).unwrap();
        loop {
            let end = cursor.read_u16::<LittleEndian>().unwrap();
            if end == 0 {
                break;
            }
            let _ = cursor.read_u16::<LittleEndian>().unwrap();
            let start = cursor.read_u16::<LittleEndian>().unwrap();

            let start = start as usize / 2;
            let end = end as usize / 2;

            for row in start..end {
                texture[first_col as usize + i][row] =
                    0xff000000 | palette::PALETTE[pixels.read_u8().unwrap() as usize];
            }
            // println!("post: {} {}", start, end);
        }
    }
    texture
}

pub fn wall_chunk_to_texture(buf: &[u8]) -> [[u32; TEX_SIZE]; TEX_SIZE] {
    let mut cursor = Cursor::new(buf);

    let mut texture = [[0; TEX_SIZE]; TEX_SIZE];
    for col in &mut texture {
        for c in col {
            *c = 0xff000000 | palette::PALETTE[cursor.read_u8().unwrap() as usize];
        }
    }
    texture
}

// pub fn list_chunks(r: &mut dyn Read) -> (Vec<u32>, Vec<u16>) {
//     let num_chunks = r.read_u16::<LittleEndian>().unwrap();
//     let num_walls = r.read_u16::<LittleEndian>().unwrap();
//     let num_sprites = r.read_u16::<LittleEndian>().unwrap();
//     let num_sounds = num_chunks - num_walls - num_sprites;
//     println!("chunks: {num_chunks} walls: {num_walls} sprites: {num_sprites} sounds: {num_sounds}");
//     let mut offs = Vec::new();
//     let mut size = Vec::new();

//     for _ in 0..num_chunks {
//         offs.push(r.read_u32::<LittleEndian>().unwrap());
//     }
//     for _ in 0..num_chunks {
//         size.push(r.read_u16::<LittleEndian>().unwrap());
//     }

//     println!("{:?} {:?}", offs, size);
//     (offs, size)
// }

#[derive(Debug)]
struct MapHeader {
    plane0_offset: u32,
    plane1_offset: u32,
    plane2_offset: u32,
    plane0_size: u16,
    plane1_size: u16,
    plane2_size: u16,
    width: u16,
    height: u16,
    name: String,
}

struct MapsFile {
    f: File,
    pub header_offsets: Vec<u32>,
    pub map_headers: Vec<MapHeader>,
    rlwe_tag: u16,
}

impl MapsFile {
    fn read_name(r: &mut impl Read) -> String {
        let mut buf = [0u8; 16];

        r.read_exact(&mut buf).unwrap();
        let mut last_char = 16;
        while last_char > 1 {
            if buf[last_char - 1] != 0 {
                break;
            }
            last_char -= 1;
        }
        String::from_utf8_lossy(&buf[..last_char]).to_string()
    }
    pub fn open<P: AsRef<Path>, Q: AsRef<Path>>(head: P, maps: Q) -> MapsFile {
        let mut f = File::open(head).unwrap();
        let rlwe_tag = f.read_u16::<LittleEndian>().unwrap();
        let header_offsets = (0..100)
            .map(|_| f.read_u32::<LittleEndian>().unwrap())
            .collect::<Vec<_>>();
        let mut f = File::open(maps).unwrap();
        let mut map_headers = Vec::new();
        for offs in &header_offsets {
            if *offs == 0 {
                continue;
            }
            f.seek(SeekFrom::Start(*offs as u64)).unwrap();
            map_headers.push(MapHeader {
                plane0_offset: f.read_u32::<LittleEndian>().unwrap(),
                plane1_offset: f.read_u32::<LittleEndian>().unwrap(),
                plane2_offset: f.read_u32::<LittleEndian>().unwrap(),
                plane0_size: f.read_u16::<LittleEndian>().unwrap(),
                plane1_size: f.read_u16::<LittleEndian>().unwrap(),
                plane2_size: f.read_u16::<LittleEndian>().unwrap(),
                width: f.read_u16::<LittleEndian>().unwrap(),
                height: f.read_u16::<LittleEndian>().unwrap(),
                name: Self::read_name(&mut f),
            });
        }

        MapsFile {
            f,
            header_offsets,
            map_headers,
            rlwe_tag,
        }
    }
}

#[test]
fn test_vswap() {
    let mut vs = VswapFile::open("vswap.wl6");

    println!("chunks: {:?}", vs.chunks);

    // for (offs, size) in offs.iter().zip(size.iter()) {
    //     f.seek(std::io::SeekFrom::Start(*offs as u64)).unwrap();

    //     let head = f.read_u16::<LittleEndian>().unwrap();
    //     println!("head: {size} {:x}", head);
    // }
}

#[test]
fn test_maps() {
    let mut maps = MapsFile::open("maphead.wl6", "gamemaps.wl6");
    println!("{:?}", maps.header_offsets);
    println!("{:?}", maps.map_headers);
}
