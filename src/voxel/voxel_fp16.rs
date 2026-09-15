use crate::voxel::prelude::*;
pub struct VoxelFp16 {
    level: i32,
    drawing_distance: u32,
    pub map: MapFile,
    camera: Camera,
    show_automap: bool,
    chopper: Chopper,
}

impl VoxelFp16 {
    pub fn spawn(spawn_info: SpawnInfo, res: &VoxelRes) -> VoxelFp16 {
        match spawn_info {
            SpawnInfo::StartLevel(index, _) => {
                let map = res.get_map(index as usize).unwrap();
                let camera = Camera::spawn_at(&map, 0.0, 0.0);

                Self {
                    level: index as i32,
                    drawing_distance: 1600,
                    camera,
                    map,
                    show_automap: false,
                    chopper: Chopper::default(),
                }
            }
            SpawnInfo::LoadSavegame(_) => todo!(),
        }
    }
    pub fn run(&mut self, input_events: &InputState, buffer: &mut [u8]) {
        self.chopper.apply_input(input_events);
        self.chopper.apply_altitude(&self.camera, &self.map);
        self.chopper.apply_to_camera(&mut self.camera);
        self.render(input_events, buffer);
        self.show_automap ^= input_events.toggle_automap;
        if self.show_automap {
            for y in 0..200 {
                let yf = (y * 1024) / 200;
                for x in 0..200 {
                    let xf = (x * 1024) / 200;
                    buffer[x + y * 320] = self.map.map[xf + yf * 1024];
                    if xf as i32 == (self.camera.x as i32) % 1024
                        || yf as i32 == (self.camera.y as i32) % 1024
                    {
                        buffer[x + y * 320] = 10;
                    }
                }
            }
        }
    }
    pub fn render(&mut self, _input_events: &InputState, buffer: &mut [u8]) {
        let screenwidth = 320.;
        let sinang = self.camera.angle.sin();
        let cosang = self.camera.angle.cos();

        let mut hiddeny = [200u32; 320];

        let mut zi = 1;
        let mut z_inc = 1;
        while zi < self.drawing_distance {
            let z: f32 = zi as f32;
            // 90 degree field of view
            let mut plx = -cosang * z - sinang * z;
            let mut ply = sinang * z - cosang * z;
            let prx = cosang * z - sinang * z;
            let pry = -sinang * z - cosang * z;

            let dx: Fp16 = ((prx - plx) / screenwidth).into();
            let dy: Fp16 = ((pry - ply) / screenwidth).into();
            let mut plx_fp: Fp16 = (plx + self.camera.x).into();
            let mut ply_fp: Fp16 = (ply + self.camera.y).into();
            let height_scale = 100.;
            let invz = 1. / z * height_scale;
            let mut horizon_cur = self.camera.horizon - self.camera.rot;
            let horizon_inc = (self.camera.rot * 4.0) / 320.0;

            for i in 0..320 {
                let x_wrapped = plx_fp.get_int().rem_euclid(self.map.width as i32) as usize;
                let y_wrapped = ply_fp.get_int().rem_euclid(self.map.height as i32) as usize;
                let mapoffset = y_wrapped * self.map.width + x_wrapped;

                let heightonscreen = ((self.camera.height as f32
                    - self.map.height_map[mapoffset as usize] as f32)
                    * invz
                    + horizon_cur) as u32;
                draw_vertical_line(
                    i,
                    heightonscreen,
                    hiddeny[i],
                    self.map.map[mapoffset as usize],
                    buffer,
                );
                if heightonscreen < hiddeny[i] {
                    hiddeny[i] = heightonscreen
                };
                plx_fp += dx;
                ply_fp += dy;
                horizon_cur += horizon_inc;
            }
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
        }
    }

    pub fn deconstruct(&self, input_state: &InputState) -> SpawnInfo {
        if input_state.next_level {
            return SpawnInfo::StartLevel(self.level.wrapping_add(1), None);
        } else if input_state.prev_level {
            return SpawnInfo::StartLevel(self.level.saturating_sub(1), None);
        }
        todo!()
    }
}
