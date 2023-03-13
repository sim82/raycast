use std::time::Instant;

use minifb::{Key, KeyRepeat, Window, WindowOptions};
use raycast::map::MapDynamic;
use raycast::ms::{Loadable, Writable};
use raycast::{wl6, Resources};

use raycast::prelude::*;

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut zbuffer = [Fp16::default(); WIDTH];

    // let resources = Resources::load_textures("textures.txt");
    let resources = Resources::load_wl6("vswap.wl6");
    let mut maps = wl6::MapsFile::open("maphead.wl6", "gamemaps.wl6");

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions {
            scale: minifb::Scale::X4,
            ..Default::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let dt: Fp16 = (1.0f32 / 60.0f32).into();
    let mut automap = false;
    let mut stop_the_world_mode = false;
    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    let mut level_id = 0;
    'outer: loop {
        println!("start level: {level_id} {}", maps.get_map_name(level_id));
        let (plane0, plane1) = maps.get_map_planes(level_id);

        let mut player_vel = PlayerVel {
            forward: 0,
            right: 0,
            rot: 0,
        };
        // let mut player = Player {
        //     x: Fp16 { v: 1933113 },
        //     y: Fp16 { v: 3793254 },
        //     rot: 3476,
        // };

        let mut things = Things::from_map_plane(&plane1);

        // let mut sprites = Sprites::from_map_plane(&plane1);
        // sprites.sprites.append(&mut things.get_sprites());
        // let mut sprites = Sprites::default();

        let mut player = things
            .get_player_start()
            .map(|(x, y, rot)| Player {
                x,
                y,
                rot,
                trigger: false,
            })
            .unwrap_or_default();

        let mut map_dynamic = MapDynamic::wrap(Map::from_map_planes(&plane0, &plane1));

        // let map = Map::default();

        let mut frame = 0;

        loop {
            if !window.is_open() || window.is_key_down(Key::Escape) {
                break 'outer;
            }

            for (i, chunk) in buffer.chunks_mut(320 * HALF_HEIGHT as usize).enumerate() {
                if i == 0 {
                    chunk.fill(0x38383838);
                } else if i == 1 {
                    chunk.fill(0x64646464);
                } else {
                    chunk.fill(0x00000064);
                }
            }

            // for i in buffer.iter_mut() {
            //     // *i = 0x40404040; // write something more funny here!

            //     *i = 0x84848484; // write something more funny here!
            // }

            for i in 0..16 {
                buffer.point(10 + i, 10 + i, i);
            }

            player_vel.forward = 0;
            player_vel.right = 0;
            player_vel.rot = 0;
            player.trigger = false;
            let (fwd_speed, rot_speed) = if window.is_key_down(Key::LeftShift) {
                (2, 360)
            } else {
                (7, 3 * 360)
            };

            if window.is_key_down(Key::W) {
                player_vel.forward += fwd_speed;
            }
            if window.is_key_down(Key::S) {
                player_vel.forward -= fwd_speed;
            }

            if window.is_key_down(Key::Q) {
                player_vel.right += fwd_speed;
            }
            if window.is_key_down(Key::E) {
                player_vel.right -= fwd_speed;
            }
            if window.is_key_down(Key::D) {
                player_vel.rot += rot_speed;
            }
            if window.is_key_down(Key::A) {
                player_vel.rot -= rot_speed;
            }
            if window.is_key_released(Key::F1) {
                player = Player::default();
            }

            if window.is_key_pressed(Key::Tab, KeyRepeat::No) {
                automap = !automap;
            }
            if window.is_key_pressed(Key::F7, KeyRepeat::No) {
                stop_the_world_mode = !stop_the_world_mode;
            }

            if window.is_key_down(Key::Space) {
                player.trigger = true;
            }

            if !stop_the_world_mode {
                map_dynamic.update(&player);
                things.update();
            }
            player.apply_vel(&player_vel, dt, &map_dynamic, !stop_the_world_mode);

            // println!("player: {:?} {:?} {:?}", player_vel, player.x, player.y);
            // println!("player: {:?}", player);

            player.draw(&mut buffer);

            let _start = Instant::now();

            // for _ in 0..1000 {
            render::sweep_raycast(
                &map_dynamic,
                &mut buffer,
                &mut zbuffer,
                &player,
                0..WIDTH,
                &resources,
            );

            let _sprite_start = Instant::now();

            // draw_sprite(&mut buffer, &zbuffer, &resources, 8, 100, sprite_z.into());
            // if frame % 4 == 0 {
            let sprite_screen_setup =
                sprite::setup_screen_pos_for_player(things.get_sprites(), &player);
            sprite::draw(sprite_screen_setup, &mut buffer, &zbuffer, &resources);

            if automap {
                map_dynamic.map.draw_automap(&mut buffer);
            }

            // }
            // println!(
            //     "time: {}us\t({}us sprite)",
            //     start.elapsed().as_micros(),
            //     sprite_start.elapsed().as_micros()
            // );

            // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
            window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
            if window.is_key_pressed(Key::F2, KeyRepeat::No) && level_id > 0 {
                level_id -= 1;
                break;
            }
            if window.is_key_pressed(Key::F3, KeyRepeat::No) && level_id < 99 {
                level_id += 1;
                break;
            }
            if window.is_key_pressed(Key::F5, KeyRepeat::No) {
                let mut f = std::fs::File::create("save.bin").unwrap();
                player.write(&mut f);
                map_dynamic.write(&mut f);
            }
            if window.is_key_pressed(Key::F6, KeyRepeat::No) {
                let map = map_dynamic.release();
                let mut f = std::fs::File::open("save.bin").unwrap();

                player = Player::read_from(&mut f);
                map_dynamic = MapDynamic::read_and_wrap(&mut f, map);
            }

            frame += 1;
        }
    }
}
