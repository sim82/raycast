use crate::prelude::*;

// inspired by https://github.com/s-macke/VoxelSpace/blob/master/VoxelSpace.html

pub mod res {
    use anyhow::Context;

    use crate::prelude::*;
    use std::{
        fs::File,
        io::{Seek, SeekFrom},
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
            #[rustfmt::skip]
            let map_names = [
                ("C1W",  "D1"),
                ("C2W",  "D2"),
                ("C3",   "D3"),
                ("C4",   "D4"),
                ("C5W",  "D5"),
                ("C6W",  "D6"),
                ("C7W",  "D7"),
                ("C8",   "D6"),
                ("C9W",  "D9"),
                ("C10W", "D10"),
                ("C11W", "D11"),
                ("C12W", "D11"),
                ("C13",  "D13"),
                ("C14",  "D14"),
                ("C14W", "D14"),
                ("C15",  "D15"),
                ("C16W", "D16"),
                ("C17W", "D17"),
                ("C18W", "D18"),
                ("C19W", "D19"),
                ("C20W", "D20"),
                ("C21",  "D21"),
                ("C22W", "D22"),
                ("C23W", "D21"),
                ("C24W", "D24"),
                ("C25W", "D25"),
                ("C26W", "D18"),
                ("C27W", "D15"),
                ("C28W", "D25"),
                ("C29W", "D16")
            ];
            let mut path = self.path.clone();
            path.push(map_names[index % map_names.len()].0);
            path.set_extension("DTA");

            let mut height_path = self.path.clone();
            height_path.push(map_names[index % map_names.len()].1);
            height_path.set_extension("DTA");
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
            let mut f = File::open(path)?;

            f.seek(SeekFrom::Start(8))?;
            let width = (f.readu16()? + 1) as usize;
            let height = (f.readu16()? + 1) as usize;
            println!("voxel map size: {}x{}", width, height);
            f.seek(SeekFrom::Start(0x80))?;
            let map = read_map(width, height, &mut f)?;
            let palette = read_palette(&mut f)?;

            let mut f = File::open(height_path)?;

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
            // let height_palette = gen_height_palette();
            let height_palette = (0..255).map(|v| v << 16 | v << 8 | v).collect::<Vec<u32>>();
            let pos = f.seek(SeekFrom::Current(0))?;
            println!("end pos: {}", pos);
            Ok(MapFile {
                width,
                height,
                map,
                height_map,
                palette,
                height_palette,
            })
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

    fn read_palette(f: &mut File) -> Result<Vec<u32>, anyhow::Error> {
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

    fn read_map(width: usize, height: usize, f: &mut File) -> Result<Vec<u8>> {
        let mut data = vec![0; width * height];
        let mut line = 0;
        let mut pos = 0;
        loop {
            let x = f.readu8()?;
            let (len, color) = if x > 0xc0 {
                (x & 0x3f, f.readu8().context(format!("{} {}", pos, line))?)
            } else {
                (1, x)
            };
            // println!("run: {} {} @ {} {}", len, color, pos, line);
            for _ in 0..len {
                data[pos + line * width] = color;
                pos += 1;
                if pos >= width {
                    pos = 0;
                    line += 1;
                }
            }
            if line >= height {
                break;
            }
        }
        Ok(data)
    }
}

pub struct Voxel {
    x: usize,
    y: usize,
    camera_angle: f32,
    camera_distance: u32,
    camera_height: i32,
    camera_horizon: i32,
    //
}
impl Default for Voxel {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            camera_angle: 0.0,
            camera_distance: 1600,
            camera_height: 78,
            camera_horizon: 100,
        }
    }
}
impl Voxel {
    pub fn run(&mut self, input_events: &InputState, map: &res::MapFile, buffer: &mut [u8]) {
        // for i in 0..1024 {
        //     buffer[i] = (i % 256) as u8;
        // }
        if input_events.strafe_right {
            self.x += 10;
        }
        if input_events.strafe_left {
            self.x = self.x.saturating_sub(10);
        }
        if input_events.forward {
            self.y = self.y.saturating_sub(10);
        }
        if input_events.backward {
            self.y += 10;
        }
        self.x = self.x.clamp(0, 1024 - 200);
        self.y = self.y.clamp(0, 1024 - 200);

        if input_events.up {
            self.camera_height += 1;
        }
        if input_events.down {
            self.camera_height -= 1;
        }
        // for y in 0..200 {
        //     for x in 0..200 {
        //         buffer[x + y * 320] = map.height_map[x + self.x + (y + self.y) * 1024];
        //     }
        // }
        self.render(input_events, map, buffer);
    }
    pub fn render(&mut self, input_events: &InputState, map: &res::MapFile, buffer: &mut [u8]) {
        let mapwidthperiod = map.width as i32 - 1;
        let mapheightperiod = map.height as i32 - 1;

        let screenwidth = 320.;
        let sinang = self.camera_angle.sin();
        let cosang = self.camera_angle.cos();

        let mut hiddeny = [200u32; 320];
        // var hiddeny = new Int32Array(screenwidth);
        // for(var i=0; i<screendata.canvas.width|0; i=i+1|0)
        // hiddeny[i] = screendata.canvas.height;

        let mut deltaz = 1.;

        // Draw from front to back
        // for(var z=1; z<camera.distance; z+=deltaz)
        // for zi in 1..self.camera_distance {
        let mut zi = 1;
        let mut z_inc = 1;
        while zi < self.camera_distance {
            let z: f32 = zi as f32;
            // 90 degree field of view
            let mut plx = -cosang * z - sinang * z;
            let mut ply = sinang * z - cosang * z;
            let prx = cosang * z - sinang * z;
            let pry = -sinang * z - cosang * z;

            let dx = (prx - plx) / screenwidth;
            let dy = (pry - ply) / screenwidth;
            plx += self.x as f32;
            ply += self.y as f32;
            let height_scale = 100.;
            let invz = 1. / z * height_scale;
            // for(var i=0; i<screenwidth|0; i=i+1|0)
            for i in 0..320 {
                let mapoffset = ((ply.floor() as i32 & mapwidthperiod) * map.width as i32)
                    + (plx.floor() as i32 & mapheightperiod);
                let heightonscreen =
                    ((self.camera_height as f32 - map.height_map[mapoffset as usize] as f32) * invz
                        + self.camera_horizon as f32) as u32;
                draw_veritcal_line(
                    i,
                    heightonscreen,
                    hiddeny[i],
                    map.map[mapoffset as usize],
                    buffer,
                );
                if heightonscreen < hiddeny[i] {
                    hiddeny[i] = heightonscreen
                };
                plx += dx;
                ply += dy;
            }
            deltaz += 0.005;
            zi += z_inc;
            if zi >= 200 {
                z_inc = 2;
            }
            if zi >= 400 {
                z_inc = 4;
            }
            if zi >= 800 {
                z_inc = 8;
            }
            // else if zi >= 200 {
            //     z_inc = 40;
            // } else if zi >= 400 {
            //     z_inc = 80;
            // } else if zi >= 800 {
            //     z_inc = 160;
            // }
        }
    }
}

fn draw_veritcal_line(x: usize, ytop: u32, ybottom: u32, color: u8, buffer: &mut [u8]) {
    assert!(x < 320);
    let ytop = ytop.max(0) as usize;
    let ybottom = ybottom as usize;
    if ytop > ybottom {
        return;
    }
    for y in ytop..ybottom {
        buffer[y * 320 + x as usize] = color;
    }
}
#[test]
fn test_dta() {
    let voxel_res = res::VoxelRes::from_dir("comanche2").unwrap();
    let _map = voxel_res.get_map(0).unwrap();
}
