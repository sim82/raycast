use crate::voxel::prelude::*;

pub struct Chopper {
    vel_x: f32,
    vel_y: f32,
    vel_z: f32,

    roll: f32,
    pitch: f32,
    yaw: f32,

    target_altitude: f32,

    climb: f32,
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
            target_altitude: 15.0,
            climb: 0.0,
        }
    }
}
impl Chopper {
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
        self.vel_z *= Self::VEL_DECAY;

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
        self.climb = 0.0;
        if input.up {
            self.climb = 5.0 * Self::DT;
        }
        if input.down {
            self.climb = -5.0 * Self::DT;
        }
        self.target_altitude = self.target_altitude.clamp(5.0, 100.0);
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
    pub fn apply_altitude(&mut self, camera: &Camera, map: &MapFile) {
        // sample ground altitude at current and forward position
        let here_ground_height = get_ground_height(camera.x, camera.y, map);
        let probe_x = camera.x + self.vel_x * 4.0;
        let probe_y = camera.y + self.vel_y * 4.0;
        let forward_ground_height = get_ground_height(probe_x, probe_y, map);

        // use maximum height
        let ground = here_ground_height.max(forward_ground_height);

        // caclulate deviation from target alititude and correction velocity
        let altitude_over_ground = camera.height - ground;
        let delta_hysteresis = if self.climb != 0.0 {
            self.target_altitude += self.climb;
            0.0
        } else {
            5.0
        };
        let delta = altitude_over_ground - self.target_altitude;
        if delta > delta_hysteresis {
            self.vel_z = -(1.0 + delta / 10.0);
        } else if delta < -delta_hysteresis {
            self.vel_z = 1.0 - delta / 10.0;
        }
        // else {
        //     self.vel_z = 0.0;
        // }
        // println!("xy: {} {} {} {}", xi, yi, delta, self.vel_z);
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

fn get_ground_height(probe_x: f32, probe_y: f32, map: &MapFile) -> f32 {
    let xi = probe_x.round().rem_euclid(1024.0) as usize;
    let yi = probe_y.round().rem_euclid(1024.0) as usize;

    let ground = map.height_map[xi + yi * 1024] as f32;
    ground
}
