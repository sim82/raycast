use crate::voxel::prelude::*;

pub struct Camera {
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub height: f32,
    pub horizon: f32,
    pub rot: f32,
}
impl Camera {
    pub fn spawn_at(map: &MapFile, x: f32, y: f32) -> Camera {
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
