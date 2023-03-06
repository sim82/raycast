use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    fs::File,
    io::{Read, Seek},
};

pub fn list_chunks(r: &mut dyn Read) -> (Vec<u32>, Vec<u16>) {
    let num_chunks = r.read_u16::<LittleEndian>().unwrap();
    let num_walls = r.read_u16::<LittleEndian>().unwrap();
    let num_sprites = r.read_u16::<LittleEndian>().unwrap();
    let num_sounds = num_chunks - num_walls - num_sprites;
    println!("chunks: {num_chunks} walls: {num_walls} sprites: {num_sprites} sounds: {num_sounds}");
    let mut offs = Vec::new();
    let mut size = Vec::new();

    for _ in 0..num_chunks {
        offs.push(r.read_u32::<LittleEndian>().unwrap());
    }
    for _ in 0..num_chunks {
        size.push(r.read_u16::<LittleEndian>().unwrap());
    }

    println!("{:?} {:?}", offs, size);
    (offs, size)
}

#[test]
fn test_vswap() {
    let mut f = File::open("vswap.wl6").unwrap();
    let (offs, size) = list_chunks(&mut f);
    for (offs, size) in offs.iter().zip(size.iter()) {
        f.seek(std::io::SeekFrom::Start(*offs as u64)).unwrap();

        let head = f.read_u16::<LittleEndian>().unwrap();
        println!("head: {size} {:x}", head);
    }
}
