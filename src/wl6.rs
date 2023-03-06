use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    fs::File,
    io::{Read, Seek},
    path::Path,
};

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
