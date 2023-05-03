use state_bc::ms::we::ReadExt;
use std::{
    fs::File,
    io::{Cursor, Read, Seek, SeekFrom},
    ops::{Range, RangeInclusive},
    path::Path,
};

use crate::Texture;

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
        let mut f = File::open(name).unwrap();
        let num_chunks = f.readu16().unwrap();
        let last_wall = f.readu16().unwrap();
        let last_sprite = f.readu16().unwrap();
        let mut offs = Vec::new();
        let mut size = Vec::new();

        for _ in 0..num_chunks {
            offs.push(f.readu32().unwrap());
        }
        for _ in 0..num_chunks {
            size.push(f.readu16().unwrap());
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
        read_vec_from_pos_size(&mut self.f, offs, size)
    }
}

pub struct SpritePosts {
    pub range: RangeInclusive<u16>,
    pub posts: Vec<(Vec<Range<u16>>, u16)>,
    pub pixels: Vec<u8>,
}

pub fn sprite_chunk_to_posts(buf: &[u8]) -> SpritePosts {
    let mut cursor = Cursor::new(buf);
    let first_col = cursor.readu16().unwrap();
    let last_col = cursor.readu16().unwrap();
    // let n = (last_col - first_col) + 1;
    let offsets = (first_col..=last_col)
        .map(|_| cursor.readu16().unwrap())
        .collect::<Vec<_>>();
    let mut pixels_cursor = cursor.clone();
    let mut posts = Vec::new();
    // let mut pixels_end = buf.len() as u64;
    let mut pixels = 0;

    for (_i, col_offset) in offsets.iter().enumerate() {
        // println!("col start {}", col_offset);
        cursor.seek(SeekFrom::Start(*col_offset as u64)).unwrap();
        // pixels_end = pixels_end.min(cursor.position());
        let mut col_posts = Vec::new();
        let pixel_start = pixels;
        loop {
            let end = cursor.readu16().unwrap();
            if end == 0 {
                break;
            }
            let _ = cursor.readu16().unwrap();
            let start = cursor.readu16().unwrap();
            let post = (start / 2)..(end / 2);
            pixels += post.len() as u16;
            col_posts.push(post);
            // let start = start as usize / 2;
            // let end = end as usize / 2;

            // for row in start..end {
            //     texture[first_col as usize + i][row] = pixels.readu8().unwrap();
            // }
            // println!("post: {} {}", start, end);
        }
        posts.push((col_posts, pixel_start));
    }

    // let num_pixels = pixels_end - pixels_cursor.position();
    let mut pixels = vec![0; pixels as usize];
    pixels_cursor.read_exact(&mut pixels).unwrap();
    SpritePosts {
        range: first_col..=last_col,
        posts,
        pixels,
    }
}

pub fn sprite_chunk_to_texture(buf: &[u8]) -> [[u8; TEX_SIZE]; TEX_SIZE] {
    let mut cursor = Cursor::new(buf);
    // UInt16LE 	FirstCol: Index of leftmost non-empty column
    // UInt16LE 	LastCol: Index of rightmost non-empty column
    // UInt16LE[n] 	Offsets relative to beginning of chunk to the first post of each column between FirstCol and LastCol (n = LastCol - FirstCol + 1)
    // UInt8[?] 	Pixel pool: Palette indexes for all solid pixels of the sprite (size unknown when decoding)
    // UInt16[?] 	Array of values describing all posts in the sprite (size unknown when decoding)

    let mut texture = [[255; TEX_SIZE]; TEX_SIZE];
    let first_col = cursor.readu16().unwrap();
    let last_col = cursor.readu16().unwrap();
    // let n = (last_col - first_col) + 1;
    let offsets = (first_col..=last_col)
        .map(|_| cursor.readu16().unwrap())
        .collect::<Vec<_>>();
    let mut pixels = cursor.clone();
    for (i, col_offset) in offsets.iter().enumerate() {
        // println!("col start {}", col_offset);
        cursor.seek(SeekFrom::Start(*col_offset as u64)).unwrap();
        loop {
            let end = cursor.readu16().unwrap();
            if end == 0 {
                break;
            }
            let _ = cursor.readu16().unwrap();
            let start = cursor.readu16().unwrap();

            let start = start as usize / 2;
            let end = end as usize / 2;

            for row in start..end {
                texture[first_col as usize + i][row] = pixels.readu8().unwrap();
            }
            // println!("post: {} {}", start, end);
        }
    }
    texture
}

pub fn wall_chunk_to_texture(buf: &[u8]) -> Texture {
    let mut cursor = Cursor::new(buf);

    let mut texture = [[0; TEX_SIZE]; TEX_SIZE];
    for col in &mut texture {
        for c in col {
            *c = cursor.readu8().unwrap();
        }
    }
    texture
}

// pub fn list_chunks(r: &mut dyn Read) -> (Vec<u32>, Vec<u16>) {
//     let num_chunks = r.readu16().unwrap();
//     let num_walls = r.readu16().unwrap();
//     let num_sprites = r.readu16().unwrap();
//     let num_sounds = num_chunks - num_walls - num_sprites;
//     println!("chunks: {num_chunks} walls: {num_walls} sprites: {num_sprites} sounds: {num_sounds}");
//     let mut offs = Vec::new();
//     let mut size = Vec::new();

//     for _ in 0..num_chunks {
//         offs.push(r.readu32().unwrap());
//     }
//     for _ in 0..num_chunks {
//         size.push(r.readu16().unwrap());
//     }

//     println!("{:?} {:?}", offs, size);
//     (offs, size)
// }

#[derive(Debug)]
pub struct MapHeader {
    pub plane0_offset: u32,
    pub plane1_offset: u32,
    pub plane2_offset: u32,
    pub plane0_size: u16,
    pub plane1_size: u16,
    pub plane2_size: u16,
    pub width: u16,
    pub height: u16,
    pub name: String,
}

pub struct MapsFile {
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
        let rlwe_tag = f.readu16().unwrap();
        let header_offsets = (0..100).map(|_| f.readu32().unwrap()).collect::<Vec<_>>();
        let mut f = File::open(maps).unwrap();
        let mut map_headers = Vec::new();
        for offs in &header_offsets {
            if *offs == 0 {
                continue;
            }
            f.seek(SeekFrom::Start(*offs as u64)).unwrap();
            map_headers.push(MapHeader {
                plane0_offset: f.readu32().unwrap(),
                plane1_offset: f.readu32().unwrap(),
                plane2_offset: f.readu32().unwrap(),
                plane0_size: f.readu16().unwrap(),
                plane1_size: f.readu16().unwrap(),
                plane2_size: f.readu16().unwrap(),
                width: f.readu16().unwrap(),
                height: f.readu16().unwrap(),
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
    pub fn get_map_id(&self, name: &str) -> i32 {
        self.map_headers
            .iter()
            .enumerate()
            .find_map(|(i, header)| {
                if header.name == name {
                    Some(i as i32)
                } else {
                    None
                }
            })
            .unwrap()
    }
    pub fn get_map_plane_chunks(&mut self, id: i32) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
        assert!(id >= 0 && (id as usize) < self.map_headers.len());
        let header = &self.map_headers[id as usize];

        (
            read_vec_from_pos_size(&mut self.f, header.plane0_offset, header.plane0_size),
            read_vec_from_pos_size(&mut self.f, header.plane1_offset, header.plane1_size),
            read_vec_from_pos_size(&mut self.f, header.plane2_offset, header.plane2_size),
        )
    }

    pub fn get_map_planes(&mut self, id: i32) -> (Vec<u16>, Vec<u16>) {
        let (v0, v1, v2) = self.get_map_plane_chunks(id);

        let d0 = map_decompress(&v0, self.rlwe_tag);
        let d1 = map_decompress(&v1, self.rlwe_tag);
        let d2 = map_decompress(&v2, self.rlwe_tag);
        assert!(d0.len() == 8192);
        assert!(d1.len() == 8192);
        assert!(d2.len() == 8192);

        (to_plane(&d0), to_plane(&d1))
    }
    pub fn get_map_name(&self, id: i32) -> &str {
        self.map_headers[id as usize].name.as_str()
    }
}

fn to_plane(d1: &[u8]) -> Vec<u16> {
    let mut res = Vec::new();
    res.reserve(d1.len() / 2);
    let mut c = Cursor::new(d1);
    for _ in 0..(d1.len() / 2) {
        res.push(c.readu16().unwrap());
    }

    res
}

fn read_vec_from_pos_size(f: &mut File, offset: u32, size: u16) -> Vec<u8> {
    f.seek(SeekFrom::Start(offset as u64)).unwrap();
    let mut buf = vec![0u8; size as usize];
    f.read_exact(&mut buf).unwrap();
    buf
}

fn carmack_decompress(input: &[u8]) -> Vec<u8> {
    let input_len = input.len() as u64;
    let mut output = Vec::new();
    let mut input = Cursor::new(input);
    let output_len = input.readu16().unwrap();
    output.reserve(output_len as usize);
    while input.position() < input_len {
        let x = input.readu8().unwrap();
        let y = input.readu8().unwrap();

        if y == 0xa7 {
            let z = input.readu8().unwrap();

            if x == 0 {
                output.push(z);
                output.push(y);
                continue;
            }

            let copy_size = (x * 2) as usize;
            let offset = (z as usize) * 2;
            let start = output.len() - offset;
            let end = start + copy_size;
            let mut copy = output[start..end].to_vec();
            output.append(&mut copy);
        } else if y == 0xA8 {
            if x == 0 {
                let z = input.readu8().unwrap();
                output.push(z);
                output.push(y);
                continue;
            }
            let copy_size = (x as usize) * 2;
            let offset = input.readu16().unwrap();
            let start = (offset as usize) * 2;

            let end = start + copy_size;
            let mut copy = output[start..end].to_vec();
            output.append(&mut copy);
        } else {
            output.push(x);
            output.push(y);
        }
    }
    assert_eq!(output_len as usize, output.len());
    output
}

fn rlew_decompress(input: &[u8], rlwe_tag: u16) -> Vec<u8> {
    let input_len = input.len() as u64;
    let mut output = Vec::new();
    let mut input = Cursor::new(input);
    // let output_len = input.readu16().unwrap() as usize;
    let output_len = input.readu16().unwrap() as usize;
    output.reserve(output_len);
    // assert_eq!(data_len as u64 + 2, input_len);
    while input.position() < input_len
    /*&& output.len() < output_len*/
    {
        let start_pos = input.position();
        let w = input.readu16().unwrap();
        if w == rlwe_tag {
            let num = input.readu16().unwrap();
            let d0 = input.readu8().unwrap();
            let d1 = input.readu8().unwrap();
            for _ in 0..num {
                output.push(d0);
                output.push(d1);
            }
        } else {
            // it is easier to re-read it bytewise than to mess around with endianness here...
            input.set_position(start_pos);
            output.push(input.readu8().unwrap());
            output.push(input.readu8().unwrap());
        }
    }
    // println!("output_len: {}", output_len);
    assert!(output.len() == output_len);
    output
}

fn map_decompress(input: &[u8], rlwe_tag: u16) -> Vec<u8> {
    let d1 = carmack_decompress(input);
    rlew_decompress(&d1, rlwe_tag)
}

#[test]
fn test_vswap() {
    let vs = VswapFile::open("vswap.wl6");

    println!("chunks: {:?}", vs.chunks);

    // for (offs, size) in offs.iter().zip(size.iter()) {
    //     f.seek(std::io::SeekFrom::Start(*offs as u64)).unwrap();

    //     let head = f.readu16().unwrap();
    //     println!("head: {size} {:x}", head);
    // }
}

#[test]
fn test_maps() {
    let mut maps = MapsFile::open("maphead.wl6", "gamemaps.wl6");

    let (v0, _v1, _v2) = maps.get_map_plane_chunks(maps.get_map_id("Wolf1 Map2"));

    println!("{:?}", maps.header_offsets);
    println!("{:?}", maps.map_headers);

    let x = carmack_decompress(&v0);
    println!("size: {} -> {}", v0.len(), x.len());
    println!("{:x?}", &x[0..8]);

    let y = rlew_decompress(&x, maps.rlwe_tag);
    println!("size: {} -> {}", x.len(), y.len());

    std::fs::write("test.bin", y).unwrap();
    // println!("x: {:#x?} -> {:#x?}", v2, x);
}

#[test]
fn test_all_maps() {
    let mut maps = MapsFile::open("maphead.wl6", "gamemaps.wl6");

    for id in 0..maps.map_headers.len() {
        let (v0, v1, v2) = maps.get_map_plane_chunks(id as i32);

        let d0 = map_decompress(&v0, maps.rlwe_tag);
        let d1 = map_decompress(&v1, maps.rlwe_tag);
        let d2 = map_decompress(&v2, maps.rlwe_tag);
        assert!(d0.len() == 8192);
        assert!(d1.len() == 8192);
        assert!(d2.len() == 8192);
    }
}
