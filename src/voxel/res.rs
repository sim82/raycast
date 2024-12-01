use anyhow::Context;

use crate::prelude::*;
use std::{
    fs::File,
    io::{BufRead, BufReader, Seek, SeekFrom},
    iter::repeat,
    path::{Path, PathBuf},
};

pub struct VoxelRes {
    path: PathBuf,
}
impl VoxelRes {
    pub fn from_dir<P: AsRef<Path>>(path: P) -> Result<VoxelRes> {
        let mut px = PathBuf::new();
        px.push(path);
        Ok(VoxelRes { path: px })
    }
    pub fn get_map(&self, index: usize) -> Result<MapFile> {
        let map_names1 = [
            ("c1.dta", "d1.dta"),
            ("c2.dta", "d2.dta"),
            ("c3.dta", "d3.dta"),
            ("c4.dta", "d4.dta"),
        ];
        #[rustfmt::skip]
        let map_names2 = [
            ("C1W.DTA",  "D1.DTA"),
            ("C2W.DTA",  "D2.DTA"),
            ("C3.DTA",   "D3.DTA"),
            ("C4.DTA",   "D4.DTA"),
            ("C5W.DTA",  "D5.DTA"),
            ("C6W.DTA",  "D6.DTA"),
            ("C7W.DTA",  "D7.DTA"),
            ("C8.DTA",   "D6.DTA"),
            ("C9W.DTA",  "D9.DTA"),
            ("C10W.DTA", "D10.DTA"),
            ("C11W.DTA", "D11.DTA"),
            ("C12W.DTA", "D11.DTA"),
            ("C13.DTA",  "D13.DTA"),
            ("C14.DTA",  "D14.DTA"),
            ("C14W.DTA", "D14.DTA"),
            ("C15.DTA",  "D15.DTA"),
            ("C16W.DTA", "D16.DTA"),
            ("C17W.DTA", "D17.DTA"),
            ("C18W.DTA", "D18.DTA"),
            ("C19W.DTA", "D19.DTA"),
            ("C20W.DTA", "D20.DTA"),
            ("C21.DTA",  "D21.DTA"),
            ("C22W.DTA", "D22.DTA"),
            ("C23W.DTA", "D21.DTA"),
            ("C24W.DTA", "D24.DTA"),
            ("C25W.DTA", "D25.DTA"),
            ("C26W.DTA", "D18.DTA"),
            ("C27W.DTA", "D15.DTA"),
            ("C28W.DTA", "D25.DTA"),
            ("C29W.DTA", "D16.DTA")
        ];
        let map_names = map_names2;
        let mut path = self.path.clone();
        path.push(map_names[index % map_names.len()].0);
        // path.set_extension("DTA");

        let mut height_path = self.path.clone();
        height_path.push(map_names[index % map_names.len()].1);
        // height_path.set_extension("DTA");
        println!("voxel map: {:?} {:?}", path, height_path);
        MapFile::read(path, height_path)
        // if index == 0 {
        //     let mut path = self.path.clone();
        //     path.push("C11W.DTA");

        //     let mut height_path = self.path.clone();
        //     height_path.push("D11.DTA");
        //     MapFile::read(&path, &height_path)
        // } else {
        //     todo!()
        // }
    }
}
pub struct MapFile {
    pub width: usize,
    pub height: usize,
    pub map: Vec<u8>,
    pub height_map: Vec<u8>,
    pub palette: Vec<u32>,
    pub height_palette: Vec<u32>,
}

impl MapFile {
    pub fn read<P: AsRef<Path>>(path: P, height_path: P) -> Result<MapFile> {
        let mut f = BufReader::new(File::open(path)?);

        f.seek(SeekFrom::Start(8))?;
        let width = (f.readu16()? + 1) as usize;
        let height = (f.readu16()? + 1) as usize;
        println!("voxel map size: {}x{}", width, height);
        f.seek(SeekFrom::Start(0x80))?;
        let map = read_map(width, height, &mut f)?;
        let palette = read_palette(&mut f)?;

        let pos = f.seek(SeekFrom::Current(0))?;
        let file_size = f.seek(SeekFrom::End(0))?;
        println!("end pos: {:x} of {:x}", pos, file_size);
        let mut f = BufReader::new(File::open(height_path)?);

        f.seek(SeekFrom::Start(8))?;
        let width_hm = (f.readu16()? + 1) as usize;
        let height_hm = (f.readu16()? + 1) as usize;
        println!("voxel map size: {}x{}", width_hm, height_hm);
        f.seek(SeekFrom::Start(0x80))?;
        let height_map = read_map(width_hm, height_hm, &mut f)?;
        let height_map = if width_hm < 1024 {
            scale2x(&height_map, width_hm, height_hm)
        } else {
            height_map
        };
        // let height_palette = read_palette(&mut f)?; // not sure for what the palette in the file is useful. Generate a linear one for debug draw.
        let height_palette = (0..255).map(|v| v << 16 | v << 8 | v).collect::<Vec<u32>>();
        Ok(MapFile {
            width,
            height,
            map,
            height_map,
            palette,
            height_palette,
        })
    }
    pub fn height_at(&self, x: f32, y: f32) -> u8 {
        let x0 = (x.floor() as usize) % self.width;
        let y0 = (y.floor() as usize) % self.height;
        let x1 = (x0 + 1) % self.width;
        let y1 = (y0 + 1) % self.height;

        let height = (self.height_map[x0 + y0 * self.width] as u16
            + self.height_map[x0 + y1 * self.width] as u16
            + self.height_map[x1 + y0 * self.width] as u16
            + self.height_map[x1 + y1 * self.width] as u16)
            / 4;

        height as u8
    }
}

fn scale2x(map: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut out = Vec::new();
    out.resize(width * height * 4, 0);
    for y in 0..height {
        for x in 0..width {
            let c = map[x + y * width];
            let pos1 = (x * 2) + y * 2 * width * 2;
            let pos2 = (x * 2) + (y * 2 + 1) * width * 2;

            out[pos1] = c;
            out[pos1 + 1] = c;
            out[pos2] = c;
            out[pos2 + 1] = c;
        }
    }
    out
}

fn read_palette(f: &mut dyn BufRead) -> Result<Vec<u32>, anyhow::Error> {
    let _ = f.readu8()?;
    let mut palette = Vec::new();
    for c in 0..256 {
        let r: u32 = f.readu8()?.into();
        let g: u32 = f.readu8()?.into();
        let b: u32 = f.readu8()?.into();
        // palette[c] = (r, g, b);
        palette.push((r << 16) | (g << 8) | b);
        println!("pal {}: {} {} {}", c, r, g, b);
    }
    Ok(palette)
}

fn read_map(width: usize, height: usize, f: &mut dyn BufRead) -> Result<Vec<u8>> {
    let mut data = Vec::with_capacity(width * height);
    loop {
        // 'kind of' rle compression:
        // If the highest two bits are set, use the lowest 6 bits as run length and
        // read another byte for the color.
        // otherwise use the color directly.
        let color = f.readu8()?;
        if color > 0b11000000 {
            let run_length = color & 0b00111111;
            let run_color = f.readu8()?;
            data.extend(repeat(run_color).take(run_length.into()))
        } else {
            data.push(color);
        }
        if data.len() == width * height {
            break;
        }
    }
    Ok(data)
}
