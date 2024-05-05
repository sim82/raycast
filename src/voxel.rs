use crate::prelude::*;

// inspired by https://github.com/s-macke/VoxelSpace/blob/master/VoxelSpace.html

pub mod res;

pub struct Camera {
    x: f32,
    y: f32,
    angle: f32,
    height: f32,
    horizon: f32,
    rot: f32,
}
impl Camera {
    pub fn spawn_at(map: &res::MapFile, x: f32, y: f32) -> Camera {
        let height = map.height_at(x, y);
        Camera {
            x,
            y,
            angle: 0.0,
            height: height as f32 + 25.0,
            horizon: 100.0,
            rot: 0.0,
        }
    }
}

pub struct Chopper {
    vel_x: f32,
    vel_y: f32,
    vel_z: f32,

    roll: f32,
    pitch: f32,
    yaw: f32,

    target_altitude: f32,
}

impl Default for Chopper {
    fn default() -> Self {
        Chopper {
            vel_x: 0.0,
            vel_y: 0.0,
            vel_z: 0.0,
            roll: 0.0,
            pitch: 0.0,
            yaw: 0.0,
            target_altitude: 30.0,
        }
    }
}
impl Chopper {
    // if input.forward {
    //     self.vel_x += forward_x * vel_scale;
    //     self.vel_y += forward_y * vel_scale;
    // }
    // if input.backward {
    //     self.vel_x -= forward_x * vel_scale;
    //     self.vel_y -= forward_y * vel_scale;
    // }
    const DT: f32 = 60.0 / 1000.0;

    const INPUT_PITCH_RATE: f32 = 25.0;
    const PITCH_DECAY: f32 = 0.9;
    const MAX_PITCH: f32 = 20.0;

    const INPUT_ROLL_RATE: f32 = 5.0;
    const ROLL_DECAY: f32 = 0.9;
    const MAX_ROLL: f32 = 20.0;

    const ROLL_YAW_RATE: f32 = 0.1;

    const PITCH_FORWARD_VEL_RATE: f32 = 0.3;
    const ROLL_RIGHT_VEL_RATE: f32 = 1.0;
    const VEL_DECAY: f32 = 0.95;

    const INPUT_YAW_RATE: f32 = 0.2;

    pub fn apply_input(&mut self, input: &InputState) {
        self.roll *= Self::ROLL_DECAY;
        self.pitch *= Self::PITCH_DECAY;

        self.vel_x *= Self::VEL_DECAY;
        self.vel_y *= Self::VEL_DECAY;

        if input.turn_right {
            self.yaw -= Self::INPUT_YAW_RATE * Self::DT;
        }
        if input.turn_left {
            self.yaw += Self::INPUT_YAW_RATE * Self::DT;
        }
        if input.forward {
            self.pitch += Self::INPUT_PITCH_RATE * Self::DT;
        }
        if input.backward {
            self.pitch -= Self::INPUT_PITCH_RATE * Self::DT;
        }
        if input.strafe_left {
            self.roll += Self::INPUT_ROLL_RATE * Self::DT;
        }
        if input.strafe_right {
            self.roll -= Self::INPUT_ROLL_RATE * Self::DT;
        }

        self.pitch = self.pitch.clamp(-Self::MAX_PITCH, Self::MAX_PITCH);
        self.roll = self.roll.clamp(-Self::MAX_ROLL, Self::MAX_ROLL);

        self.yaw += self.roll * Self::ROLL_YAW_RATE * Self::DT;
        let forward_x = -self.yaw.sin();
        let forward_y = -self.yaw.cos();
        let right_x = -self.yaw.cos();
        let right_y = self.yaw.sin();

        self.vel_x += forward_x * self.pitch * Self::PITCH_FORWARD_VEL_RATE * Self::DT;
        self.vel_y += forward_y * self.pitch * Self::PITCH_FORWARD_VEL_RATE * Self::DT;
        self.vel_x += right_x * self.roll * Self::ROLL_RIGHT_VEL_RATE * Self::DT;
        self.vel_y += right_y * self.roll * Self::ROLL_RIGHT_VEL_RATE * Self::DT;

        if self.vel_x.abs() < 0.001 {
            self.vel_x = 0.0;
        }
        if self.vel_y.abs() < 0.001 {
            self.vel_y = 0.0;
        }
    }
    pub fn apply_altitude(&mut self, camera: &Camera, map: &res::MapFile) {
        // sample ground altitude

        let probe_x = camera.x + self.vel_x * 4.0;
        let probe_y = camera.y + self.vel_y * 4.0;
        let xi = probe_x.round().rem_euclid(1024.0) as usize;
        let yi = probe_y.round().rem_euclid(1024.0) as usize;

        let ground = map.height_map[xi + yi * 1024] as f32;
        let altitude_over_ground = camera.height - ground;
        let delta = altitude_over_ground - self.target_altitude;

        if delta > 5.0 {
            self.vel_z = -(1.0 + delta / 10.0);
        } else if delta < -5.0 {
            self.vel_z = (1.0 - delta / 10.0);
        } else {
            self.vel_z = 0.0;
        }
        println!("xy: {} {} {} {}", xi, yi, delta, self.vel_z);
        // println!("alt delta: {}", camera.height - ground_altitude);
        // self.target_altitude = ground_altitude + 20.0;
    }
    pub fn apply_to_camera(&self, camera: &mut Camera) {
        camera.x += self.vel_x * 0.166;
        camera.y += self.vel_y * 0.166;
        camera.height += self.vel_z * 0.166;
        camera.angle = self.yaw;

        // ultra crappy linear approximation: directly offset horizon by pitch angle. Looks close enough for the early 90s
        camera.horizon = 100.0 - self.pitch * 4.0;
        camera.rot = self.roll * 10.0;
    }
}

pub struct Voxel {
    level: i32,
    drawing_distance: u32,
    pub map: res::MapFile,
    camera: Camera,
    show_automap: bool,
    chopper: Chopper,
}

impl Voxel {
    pub fn spawn(spawn_info: SpawnInfo, res: &res::VoxelRes) -> Voxel {
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
        // if input_events.strafe_right {
        //     self.camera.x += 1.0;
        // }
        // if input_events.strafe_left {
        //     self.camera.x -= 1.0;
        // }
        // if input_events.forward {
        //     self.camera.y -= 1.0;
        // }
        // if input_events.backward {
        //     self.camera.y += 1.0;
        // }
        // if input_events.up {
        //     self.camera.height += 1.0;
        // }
        // if input_events.down {
        //     self.camera.height -= 1.0;
        // }
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
        let mapwidthperiod = self.map.width as i32 - 1;
        let mapheightperiod = self.map.height as i32 - 1;

        let screenwidth = 320.;
        let sinang = self.camera.angle.sin();
        let cosang = self.camera.angle.cos();

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
        while zi < self.drawing_distance {
            let z: f32 = zi as f32;
            // 90 degree field of view
            let mut plx = -cosang * z - sinang * z;
            let mut ply = sinang * z - cosang * z;
            let prx = cosang * z - sinang * z;
            let pry = -sinang * z - cosang * z;

            let dx = (prx - plx) / screenwidth;
            let dy = (pry - ply) / screenwidth;
            plx += self.camera.x;
            ply += self.camera.y;
            let height_scale = 100.;
            let invz = 1. / z * height_scale;
            // for(var i=0; i<screenwidth|0; i=i+1|0)
            let mut horizon_cur = self.camera.horizon - self.camera.rot;
            let horizon_inc = (self.camera.rot * 4.0) / 320.0;

            for i in 0..320 {
                let mapoffset = ((ply.floor() as i32 & mapwidthperiod) * self.map.width as i32)
                    + (plx.floor() as i32 & mapheightperiod);
                let heightonscreen = ((self.camera.height as f32
                    - self.map.height_map[mapoffset as usize] as f32)
                    * invz
                    + horizon_cur) as u32;
                draw_veritcal_line(
                    i,
                    heightonscreen,
                    hiddeny[i],
                    self.map.map[mapoffset as usize],
                    buffer,
                );
                if heightonscreen < hiddeny[i] {
                    hiddeny[i] = heightonscreen
                };
                plx += dx;
                ply += dy;
                horizon_cur += horizon_inc;
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
