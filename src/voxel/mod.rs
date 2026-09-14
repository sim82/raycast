use crate::voxel::prelude::*;

// inspired by https://github.com/s-macke/VoxelSpace/blob/master/VoxelSpace.html

pub mod camera;
pub mod chopper;
pub mod prelude;
pub mod res;

pub mod voxel_f32;

pub fn draw_vertical_line(x: usize, ytop: u32, ybottom: u32, color: u8, buffer: &mut [u8]) {
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
