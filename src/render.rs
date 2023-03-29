use std::ops::Range;

use crate::prelude::*;

pub fn sweep_raycast<D: Draw + ?Sized>(
    map_dynamic: &MapDynamic,
    screen: &mut D,
    zbuffer: &mut [Fp16; WIDTH],
    player: &Player,
    columns: Range<usize>,
    resources: &Resources,
) {
    let (x, y) = player.int_pos();

    if !map_dynamic.can_walk(x, y) {
        return;
    }
    let (dx, dy) = player.frac_pos();
    let (ex, ey) = (FP16_ONE - dx, FP16_ONE - dy);
    for column in columns {
        let alpha_full = player.rot + COL_ANGLE[column];
        let alpha_full = if alpha_full < 0 {
            alpha_full + FA_TAU
        } else if alpha_full >= FA_TAU {
            alpha_full - FA_TAU
        } else {
            alpha_full
        };

        let hstep_x;
        let hstep_y;
        let tx;
        let ty;

        let mut nx;
        let mut ny;

        if QUADRANT_1.contains(&alpha_full) {
            let alpha = alpha_full - QUADRANT_1.start;
            hstep_x = 1;
            hstep_y = 1;
            tx = fa_cot(alpha);
            ty = fa_tan(alpha);
            nx = player.x + (tx * ey);
            ny = player.y + (ty * ex);
        } else if QUADRANT_2.contains(&alpha_full) {
            let alpha = alpha_full - QUADRANT_2.start;
            hstep_x = -1;
            hstep_y = 1;
            tx = -fa_tan(alpha);
            ty = fa_cot(alpha);
            nx = player.x + (tx * ey);
            ny = player.y + (ty * dx); // - 1.into();
        } else if QUADRANT_3.contains(&alpha_full) {
            let alpha = alpha_full - QUADRANT_3.start;
            hstep_x = -1;
            hstep_y = -1;
            tx = -fa_cot(alpha);
            ty = -fa_tan(alpha);
            nx = player.x + (tx * dy);
            ny = player.y + (ty * dx);
        } else if QUADRANT_4.contains(&alpha_full) {
            let alpha = alpha_full - QUADRANT_4.start;
            hstep_x = 1;
            hstep_y = -1;
            tx = fa_tan(alpha);
            ty = -fa_cot(alpha);
            nx = player.x + (tx * dy);
            ny = player.y + (ty * ex);
        } else {
            continue;
        };

        // TODO: can't we just shift hx/nx (or hy/ny) both by -1 to get rid of lookup correction?
        // hx/hy is the x/y boundary of the next block we enter. Depending on the hstep direction this is either the left or right (upper or lower)
        // boundary.
        let mut hx = x + hstep_x.max(0);
        let mut hy = y + hstep_y.max(0);

        let hit_tile;
        let dx;
        let dy;
        let tex_u;
        let tyh = ty * FP16_HALF;
        let txh = tx * FP16_HALF;
        let hstep_x_half = FP16_HALF * hstep_x;
        let hstep_y_half = FP16_HALF * hstep_y;

        // from_door tracks if the *last* tile we traced through was a door, to easily fix up textures on door sidewalls
        let mut from_door = matches!(map_dynamic.lookup_tile(x, y), MapTile::Door(_, _, _));

        'outer: loop {
            if (hstep_y > 0 && ny <= hy.into()) || (hstep_y < 0 && ny >= hy.into()) {
                // when hstep_x is negative, hx needs to be corrected by -1 (enter block from right / below). Inverse of the correction during hx initialization.
                let lookup_tile = map_dynamic.lookup_tile(hx + hstep_x.min(0), ny.get_int());

                match lookup_tile {
                    MapTile::Wall(tile) => {
                        screen.point_world(hx.into(), ny, (tile) % 16);

                        if from_door {
                            hit_tile = 101;
                        } else {
                            hit_tile = tile + 1
                        }

                        dx = Fp16::from(hx) - player.x;
                        dy = ny - player.y;
                        tex_u = if hstep_x > 0 {
                            (ny.get_fract() >> 10) as i32
                        } else {
                            63 - (ny.get_fract() >> 10) as i32
                        };
                        break 'outer;
                    }
                    MapTile::OffsetWall(tile, direction, f) => {
                        let hit_y = ny + ty * f;

                        if (hstep_y > 0 && hit_y <= hy.into()) || (hstep_y < 0 && hit_y >= hy.into()) {
                            if direction == Direction::West && hstep_x < 0 {
                                dx = Fp16::from(hx) - player.x - f;
                            } else if direction == Direction::East && hstep_x > 0 {
                                dx = Fp16::from(hx) - player.x + f;
                            } else {
                                dx = Fp16::from(hx) - player.x;
                            }
                            hit_tile = tile + 1;
                            dy = hit_y - player.y;
                            tex_u = if hstep_x > 0 {
                                (hit_y.get_fract() >> 10) as i32
                            } else {
                                63 - (hit_y.get_fract() >> 10) as i32
                            };
                            break 'outer;
                        }
                    }
                    MapTile::Door(PlaneOrientation::X, door_type, state_index) => {
                        let door_state = &map_dynamic.door_states[state_index];
                        let door_hit = ny + tyh;
                        let door_hit_f = door_hit.fract() - door_state.open_f;
                        if door_hit_f > FP16_ZERO
                            && ((hstep_y > 0 && door_hit <= hy.into()) || (hstep_y < 0 && door_hit >= hy.into()))
                        {
                            hit_tile = door_type.get_texture_id() + 1;
                            dx = (Fp16::from(hx) + hstep_x_half) - player.x;
                            dy = door_hit - player.y;
                            tex_u = ((door_hit_f).get_fract() >> 10) as i32;
                            break 'outer;
                        }
                        from_door = true;
                    }
                    _ => {
                        from_door = false;
                    }
                }

                hx += hstep_x;
                ny += ty;
            } else {
                // when hstep_y is negative, hy needs to be corrected by -1 (enter block from right / below). Inverse of the correction during hx initialization.
                let lookup_tile = map_dynamic.lookup_tile(nx.get_int(), hy + hstep_y.min(0));
                match lookup_tile {
                    MapTile::Wall(tile) => {
                        if from_door {
                            hit_tile = 100;
                        } else {
                            hit_tile = tile;
                        }

                        screen.point_world(nx, hy.into(), (tile) % 16);
                        dx = nx - player.x;
                        dy = Fp16::from(hy) - player.y;
                        tex_u = if hstep_y < 0 {
                            (nx.get_fract() >> 10) as i32
                        } else {
                            63 - (nx.get_fract() >> 10) as i32
                        };

                        break 'outer;
                    }
                    MapTile::OffsetWall(tile, direction, f) => {
                        let hit_x = nx + tx * f;

                        if (hstep_x > 0 && hit_x <= hx.into()) || (hstep_x < 0 && hit_x >= hx.into()) {
                            if direction == Direction::North && hstep_y < 0 {
                                dy = Fp16::from(hy) - player.y - f;
                            } else if direction == Direction::South && hstep_y > 0 {
                                dy = Fp16::from(hy) - player.y + f;
                            } else {
                                dy = Fp16::from(hy) - player.y;
                            }
                            hit_tile = tile;
                            dx = hit_x - player.x;
                            tex_u = if hstep_y < 0 {
                                (hit_x.get_fract() >> 10) as i32
                            } else {
                                63 - (hit_x.get_fract() >> 10) as i32
                            };

                            break 'outer;
                        }
                    }
                    MapTile::Door(PlaneOrientation::Y, door_type, state_index) => {
                        let door_state = &map_dynamic.door_states[state_index];
                        let door_hit = nx + txh;
                        let door_hit_f = door_hit.fract() - door_state.open_f;
                        if door_hit_f > FP16_ZERO
                            && ((hstep_x > 0 && door_hit <= hx.into()) || (hstep_x < 0 && door_hit >= hx.into()))
                        {
                            hit_tile = door_type.get_texture_id();
                            dx = door_hit - player.x;
                            dy = (Fp16::from(hy) + hstep_y_half) - player.y;
                            tex_u = (door_hit_f.get_fract() >> 10) as i32;
                            break 'outer;
                        }
                        from_door = true;
                    }
                    _ => {
                        from_door = false;
                    }
                }
                hy += hstep_y;
                nx += tx;
            }
        }

        const C: i32 = MID;
        let beta = player.rot;
        let p = fa_cos(beta) * dx + fa_sin(beta) * dy;
        let offs = if p > FP16_ZERO { (C << FP16_SCALE) / p.v } else { C };
        zbuffer[column] = p;
        let line_range = (MID - offs)..(MID + offs);

        // TODO: pre-compute LUTs for the non clipped cases?
        const TEXEL_SCALE: i32 = 1; // NOTE: this currently influences the performance of the bresenham loop
        let (tex_clip, line_range_clamped) = if line_range.start < 0 {
            const HALF_TEX: i32 = 32 << TEXEL_SCALE;
            let h = HALF_HEIGHT - line_range.start; // overall (half) height of drawn column (partially offscreen)
            let x = (HALF_TEX * HALF_HEIGHT) / h; // (half) number of texture pixels inside visible range (derived via HALF_HEIGHT / h == x / HALF_TEX)
            let tex_clip = HALF_TEX - x;
            (tex_clip, 0..VIEW_HEIGHT)
        } else {
            (0, line_range)
        };
        // do not include 'endpoints' in bresenham. -1 on d_screen and d_tex to account for this.
        let d_screen = line_range_clamped.end - line_range_clamped.start - 1;
        let d_tex = (64 << TEXEL_SCALE) - 2 * tex_clip - 1;
        let mut d = 2 * d_tex - d_screen;
        let mut row_tex = 0;
        let tex_col = &resources.get_texture(hit_tile)[(tex_u) as usize];

        for row_screen in line_range_clamped {
            // screen.point_rgb(
            //     column as i32,
            //     row_screen,
            //     tex_col[(row_tex + tex_clip) as usize >> TEXEL_SCALE],
            // );
            screen.point(
                column as i32,
                row_screen,
                tex_col[(row_tex + tex_clip) as usize >> TEXEL_SCALE] as i32,
            );
            while d > 0 {
                row_tex += 1;
                d -= 2 * d_screen;
            }
            d += 2 * d_tex
        }
        // screen.point(column as i32, MID + offs, 0);
        // screen.point(column as i32, MID - offs, 0);
    }
}

pub fn draw_sprite<D: Draw + ?Sized>(
    screen: &mut D,
    zbuffer: &[Fp16],
    resources: &Resources,
    id: i32,
    x_mid: i32,
    z: Fp16,
) {
    const C: i32 = MID;
    let offs = if z > FP16_ZERO { (C << FP16_SCALE) / z.v } else { C };
    let line_range = (MID - offs)..(MID + offs);

    let tex = resources.get_sprite_as_texture(id);

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
                // screen.point_rgb(column, row, c);
                screen.point(column, row, c as i32);
            }
        }
    }
}

pub fn draw_sprite2<D: Draw + ?Sized>(
    screen: &mut D,
    zbuffer: &[Fp16],
    resources: &Resources,
    id: i32,
    x_mid: i32,
    z: Fp16,
) {
    const C: i32 = MID;
    let offs = if z > FP16_ZERO { (C << FP16_SCALE) / z.v } else { C };
    let line_range = (MID - offs)..(MID + offs);

    let tex = resources.get_sprite_as_texture(id);
    let sprite = resources.get_sprite(id);

    let target_size = 2 * offs;
    // TODO: re-think the whole thing... it kind of works without explicit clipping to screen bounds, but the redundant in-loop bounds checks are weird and inefficient

    let dx_screen = (line_range.end - line_range.start) - 1;
    let dx_tex = 64 - 1;
    let mut dx = 2 * dx_tex - dx_screen;
    let mut xtex = 0;
    // only fill first (2 * offs, screen-space target size) entries of the texcoord array, using bresenham-style interpolator
    // let mut pixels = sprite.pixels.iter();

    'outer: for x in (x_mid - offs)..(x_mid + offs) {
        // let mut pixel_i_cur = pixel_i;

        if sprite.range.contains(&xtex) {
            let dy_screen = (line_range.end - line_range.start) - 1;
            let dy_tex = 64 - 1;
            let mut dy = 2 * dy_tex - dy_screen;
            // let mut pixel = 0;
            let mut ytex = 0;
            let (posts, mut pixel_i) = &sprite.posts[(xtex - sprite.range.start()) as usize];
            let mut posts = posts.iter();
            let Some(mut post) = posts.next() else {continue};
            let mut in_post = post.0 == 0;

            'yloop: for y in line_range.clone() {
                // screen.point(x, y, xtex as i32);
                if in_post {
                    if pixel_i as usize >= sprite.pixels.len() {
                        break 'outer;
                    }
                    screen.point(x, y, sprite.pixels[pixel_i as usize] as i32);
                }

                while dy > 0 {
                    if ytex >= post.1 {
                        post = match posts.next() {
                            Some(post) => post,
                            None => break 'yloop,
                        };
                    }

                    in_post = post.0 <= ytex;

                    pixel_i += 1;
                    ytex += 1;
                    dy -= 2 * dy_screen;
                }
                dy += 2 * dy_tex;
            }
        }
        while dx > 0 {
            xtex += 1;
            // c = xtex as i32;
            // pixel_i = pixel_i_cur;
            dx -= 2 * dx_screen;
        }
        dx += 2 * dx_tex;
    }
}

#[test]
fn raycast_test() {
    let map_dynamic = MapDynamic::wrap(Map::default());
    let resources = Resources::default();
    let mut screen = vec![0; WIDTH * HEIGHT];
    let mut zbuffer = [Fp16::default(); WIDTH];
    let player = Player {
        x: Fp16 { v: 72090 },
        y: Fp16 { v: 72090 },
        rot: 0,
        trigger: false,
        shoot: false,
        shoot_timeout: 0,
    };
    let col = 10;
    sweep_raycast(
        &map_dynamic,
        &mut screen[..],
        &mut zbuffer,
        &player,
        col..(col + 1),
        &resources,
    );
}
