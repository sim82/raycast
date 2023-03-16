pub use crate::prelude::*;

#[derive(Clone, Copy, Debug)]
pub enum Directionality {
    Direction(Direction),
    Undirectional,
}

pub struct SpriteDef {
    pub x: Fp16,
    pub y: Fp16,
    pub id: i32,
    pub directionality: Directionality,
    pub owner: usize,
}

pub struct SpriteSceenSetup {
    pub z: Fp16,
    pub screen_x: i32,
    id: i32,
    pub owner: usize,
}

pub fn draw(
    sprite_screen_setup: impl IntoIterator<Item = SpriteSceenSetup>,
    screen: &mut Vec<u32>,
    zbuffer: &[Fp16],
    resources: &Resources,
) {
    for SpriteSceenSetup {
        z,
        screen_x,
        id,
        owner: _,
    } in sprite_screen_setup.into_iter()
    {
        render::draw_sprite(screen, zbuffer, resources, id, screen_x, z);
    }
}

pub fn setup_screen_pos_for_player(
    sprites: impl IntoIterator<Item = SpriteDef>,
    player: &Player,
) -> Vec<SpriteSceenSetup> {
    let inv_sin = fa_sin(fa_fix_angle(-player.rot));
    let inv_cos = fa_cos(fa_fix_angle(-player.rot));

    let mut screen_pos = sprites
        .into_iter()
        .filter_map(|sprite| {
            let x = sprite.x - player.x;
            let y = sprite.y - player.y;

            let tx = x * inv_cos - y * inv_sin;
            let ty = y * inv_cos + x * inv_sin;

            if tx <= FP16_HALF {
                return None;
            }

            let z = tx;
            // let screen_x = (WIDTH as i32 / 2) + (ty * (WIDTH as i32 / 2)).get_int();
            let screen_x = ((FP16_ONE + (ty / z)) * (WIDTH as i32 / 2)).get_int();

            let id = if let Directionality::Direction(direction) = sprite.directionality {
                // calculate sprite orientation from player view angle
                // 1. rotate sprite by inverse of camera rotation
                // 2. turn by 'half an octant' to align steps to expected position
                let mut viewangle = -player.rot + (FA_STEPS as i32) / 16;
                // 3. approximate angle offset according to screen x position (who needs trigonometry whan you can fake it...)
                // 4. turn by 180 deg to look along x axis
                let cor = (160 - screen_x) * 2;
                viewangle += cor + 1800;

                while viewangle < 0 {
                    viewangle += FA_STEPS as i32;
                }
                while viewangle >= FA_STEPS as i32 {
                    viewangle -= FA_STEPS as i32;
                }
                viewangle /= FA_STEPS as i32 / 8;
                viewangle += direction.sprite_offset();
                sprite.id + viewangle % 8
            } else {
                sprite.id
            };
            // println!("viewangle: {viewangle}");
            Some(SpriteSceenSetup {
                z,
                screen_x,
                id,
                owner: sprite.owner,
            })
        })
        .collect::<Vec<_>>();
    screen_pos.sort_by_key(|SpriteSceenSetup { z, .. }| -(*z));
    screen_pos
}
