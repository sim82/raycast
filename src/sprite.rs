pub use crate::prelude::*;

pub fn draw_sprite(
    screen: &mut Vec<u32>,
    zbuffer: &[Fp16],
    resources: &Resources,
    id: i32,
    x_mid: i32,
    z: Fp16,
) {
    const C: i32 = MID;
    let offs = if z > FP16_ZERO {
        (C << FP16_SCALE) / z.v
    } else {
        C
    };
    let line_range = (MID - offs)..(MID + offs);

    let tex = resources.get_sprite(id);

    let target_size = 2 * offs;
    // TODO: re-think the whole thing... it kind of works without explicit clipping to screen bounds, but the redundant in-loop bounds checks are weird and inefficient

    let texcoord = {
        // pre-calc screen -> tex coords
        let mut texcoord = [0u8; WIDTH];

        let d_screen = (line_range.end - line_range.start) - 1;
        let d_tex = 64 - 1;
        let mut d = 2 * d_tex - d_screen;
        let mut tex = 0;
        // only fill first (2 * offs, screen-space target size) entries of the texcoord array, using bresenham-style interpolator
        for tx in texcoord.iter_mut().take(target_size as usize) {
            *tx = tex;

            while d > 0 {
                tex += 1;
                d -= 2 * d_screen;
            }
            d += 2 * d_tex;
        }
        texcoord
    };
    for x in 0..(target_size.min(WIDTH as i32)) {
        let column = x + (x_mid - offs);
        // TODO: clip to screen bounds
        if column < 0 || column >= WIDTH as i32 || zbuffer[column as usize] <= z {
            continue;
        }
        let tex_col = tex[texcoord[x as usize] as usize];
        for y in 0..(target_size.min(WIDTH as i32)) {
            let row = y + (MID - offs);
            if !(0..VIEW_HEIGHT).contains(&row) {
                continue;
            }
            if (0..VIEW_HEIGHT).contains(&row) {
                let c = tex_col[texcoord[y as usize] as usize];
                if c != 0 {
                    screen.point_rgb(column, row, c);
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Directionality {
    Direction(Direction),
    Undirectional,
}

pub struct Sprite {
    pub x: Fp16,
    pub y: Fp16,
    pub id: i32,
    pub directionality: Directionality,
}

#[derive(Default)]
pub struct Sprites {
    pub sprites: Vec<Sprite>,
    pub screen_pos: Vec<(Fp16, i32, i32)>,
}

impl Sprites {
    pub fn from_map_plane(plane: &[u16]) -> Sprites {
        assert_eq!(plane.len(), 64 * 64);
        let mut plane_iter = plane.iter();
        let props_range = 23..=70;
        let props_sprite_offset = 2;
        let mut sprites = Vec::new();
        for y in 0..64 {
            for x in 0..64 {
                let c = (*plane_iter.next().unwrap()) + 1;

                if props_range.contains(&c) {
                    sprites.push(Sprite {
                        x: FP16_HALF + x.into(),
                        y: FP16_HALF + y.into(),
                        id: (c - props_range.start() + props_sprite_offset) as i32,
                        directionality: Directionality::Undirectional,
                    })
                }
            }
        }
        Sprites {
            sprites,
            ..Default::default()
        }
    }

    pub fn setup_screen_pos_for_player(&mut self, player: &Player) {
        self.screen_pos.clear();
        self.screen_pos.reserve(self.sprites.len());
        let inv_sin = fa_sin(fa_fix_angle(-player.rot));
        let inv_cos = fa_cos(fa_fix_angle(-player.rot));

        for sprite in &self.sprites {
            let x = sprite.x - player.x;
            let y = sprite.y - player.y;

            let tx = x * inv_cos - y * inv_sin;
            let ty = y * inv_cos + x * inv_sin;

            if tx <= FP16_HALF
            /*|| tx.get_int() < 1 */
            {
                continue;
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
            self.screen_pos.push((z, screen_x, id));
        }
        self.screen_pos.sort_by_key(|(z, _, _)| -(*z));
    }
    pub fn draw(
        &self,
        screen: &mut Vec<u32>,
        zbuffer: &[Fp16],
        resources: &Resources,
        _frame: i32,
    ) {
        for (z, mid, id) in &self.screen_pos {
            draw_sprite(
                screen, zbuffer, resources, *id, /*+ ((frame / 30) % 8)*/
                *mid, *z,
            );
        }
    }
}
