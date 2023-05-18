use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};

use state_bc::ms::endian::ReadExt;

pub struct ChunksFile {
    f: File,
    offsets: Vec<u32>,
}
impl ChunksFile {
    pub fn open<P: AsRef<Path>, Q: AsRef<Path>>(
        header_name: P,
        data_name: Q,
    ) -> crate::Result<Self> {
        let mut headf = File::open(header_name)?;
        let mut offsets = Vec::new();
        while let Ok(offs) = headf.readu32() {
            offsets.push(offs);
        }
        let f = File::open(data_name)?;
        Ok(Self { offsets, f })
    }
    pub fn read_chunk_raw(&mut self, id: i32) -> Option<Vec<u8>> {
        if id < 0 || id >= self.offsets.len() as i32 - 1 {
            return None;
        }
        let offset = self.offsets[id as usize] as u64;
        self.f.seek(SeekFrom::Start(offset)).ok()?;
        let size = self.offsets[id as usize + 1] as u64 - offset;
        let mut buf = vec![0; size as usize];
        self.f.read_exact(&mut buf).ok()?;
        Some(buf)
    }
    pub fn len(&self) -> i32 {
        self.offsets.len() as i32 - 1
    }
}
#[test]
fn test_vga() {
    let cf = ChunksFile::open("wl6/vgahead.wl6", "wl6/vgagraph.wl6").unwrap();
}
#[test]
fn test_audio() {
    let mut cf = ChunksFile::open("wl6/audiohed.wl6", "wl6/audiot.wl6").unwrap();
    let chunk = cf.read_chunk_raw(163).unwrap();
    println!("chunk: {chunk:?}");
}
// #[test]
// fn test_dump_vga() {
//     let mut cf = ChunksFile::open("wl6/vgahead.wl6", "wl6/vgagraph.wl6").unwrap();
//     for i in 0..cf.len() {
//         println!("i {i}");
//         let chunk = cf.read_chunk_raw(i).unwrap();
//         let mut f = File::create(format!("wl6/vga.{i:03}")).unwrap();
//         f.write_all(&chunk).unwrap();
//     }
// }
// #[test]
// fn test_dump_audio() {
//     let mut cf = ChunksFile::open("wl6/audiohed.wl6", "wl6/audiot.wl6").unwrap();
//     for i in 0..cf.len() {
//         let chunk = cf.read_chunk_raw(i).unwrap();
//         let mut f = File::create(format!("wl6/audio.{i:03}")).unwrap();
//         f.write_all(&chunk).unwrap();
//     }
// }
