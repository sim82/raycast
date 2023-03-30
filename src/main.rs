use std::time::Instant;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use rand::random;
// use minifb::{Key, KeyRepeat, Window, WindowOptions};
use raycast::{
    ms::{Loadable, Writable},
    palette::PALETTE,
    prelude::*,
    wl6, Resources,
};
use sdl2::{event::Event, keyboard::Scancode, mouse::MouseButton, pixels::PixelFormatEnum, EventPump};

struct StaticMapData {
    level_id: i32,
    map: Map,
    thing_defs: ThingDefs,
}

enum SpawnInfo {
    StartLevel(i32, Option<StaticMapData>),
    LoadSavegame(Option<StaticMapData>),
}

#[derive(Default)]
struct InputState {
    // one-shot events
    quit: bool,
    restart: bool,
    prev_level: bool,
    next_level: bool,
    save: bool,
    load: bool,
    toggle_automap: bool,
    toggle_stop_the_world: bool,
    toggle_mouse_grab: bool,

    // press state
    forward: bool,
    backward: bool,
    turn_left: bool,
    turn_right: bool,
    strafe_left: bool,
    strafe_right: bool,
    slow: bool,
    open: bool,
    shoot: bool,
    fast_forward_mode: bool,
    toggle_render_alternative: bool,

    // mouse
    dx: i32,
    dy: i32,
}

impl InputState {
    pub fn new(events: &mut EventPump) -> InputState {
        let mut input_state = InputState::default();
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => input_state.quit = true,
                Event::KeyDown {
                    scancode: Some(scancode),
                    repeat: false,
                    ..
                } => match scancode {
                    Scancode::Escape => input_state.quit = true,
                    Scancode::F1 => input_state.restart = true,
                    Scancode::F2 => input_state.prev_level = true,
                    Scancode::F3 => input_state.next_level = true,
                    Scancode::F5 => input_state.save = true,
                    Scancode::F6 => input_state.load = true,
                    Scancode::F7 => input_state.toggle_stop_the_world = true,
                    Scancode::F9 => input_state.toggle_render_alternative = true, // can be used e.g. to toggle between different draw impls at runtime
                    Scancode::Tab => input_state.toggle_automap = true,
                    Scancode::Grave => input_state.toggle_mouse_grab = true,
                    _ => (),
                },
                Event::MouseMotion { xrel, yrel, .. } => {
                    input_state.dx += xrel;
                    input_state.dy += yrel;
                }
                _ => (),
            }
        }
        let keyboard_state = events.keyboard_state();
        input_state.forward = keyboard_state.is_scancode_pressed(Scancode::W);
        input_state.backward = keyboard_state.is_scancode_pressed(Scancode::S);
        const HOLD_STRAFE: bool = true;
        if !HOLD_STRAFE {
            input_state.turn_left = keyboard_state.is_scancode_pressed(Scancode::A);
            input_state.turn_right = keyboard_state.is_scancode_pressed(Scancode::D);
            input_state.strafe_left = keyboard_state.is_scancode_pressed(Scancode::Q);
            input_state.strafe_right = keyboard_state.is_scancode_pressed(Scancode::E);
        } else {
            input_state.turn_left = keyboard_state.is_scancode_pressed(Scancode::Q);
            input_state.turn_right = keyboard_state.is_scancode_pressed(Scancode::E);
            input_state.strafe_left = keyboard_state.is_scancode_pressed(Scancode::A);
            input_state.strafe_right = keyboard_state.is_scancode_pressed(Scancode::D);
        }
        input_state.slow = keyboard_state.is_scancode_pressed(Scancode::LShift);
        input_state.open = keyboard_state.is_scancode_pressed(Scancode::Space);
        input_state.shoot = keyboard_state.is_scancode_pressed(Scancode::LCtrl);
        input_state.fast_forward_mode = keyboard_state.is_scancode_pressed(Scancode::F8);
        let mouse_state = events.mouse_state();
        input_state.shoot |= mouse_state.is_mouse_button_pressed(MouseButton::Left);
        input_state
    }
}

fn main() -> raycast::prelude::Result<()> {
    let mut buffer: Vec<u8> = vec![0; WIDTH * HEIGHT];
    let mut zbuffer = [Fp16::default(); WIDTH];

    // let resources = Resources::load_textures("textures.txt");
    let resources = Resources::load_wl6("vswap.wl6");
    let mut maps = wl6::MapsFile::open("maphead.wl6", "gamemaps.wl6");

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window(
            "rust-sdl2_gfx: draw line & FPSManager",
            WIDTH as u32 * 4,
            HEIGHT as u32 * 4,
        )
        .position_centered()
        .build()?;

    let mut canvas = window.into_canvas().present_vsync().build()?;
    let creator = canvas.texture_creator();
    let mut texture = creator.create_texture_streaming(PixelFormatEnum::RGB24, 320, 200)?;

    let mut events = sdl_context
        .event_pump()
        .unwrap_or_else(|_| panic!("faild to get event pump"));

    let dt: Fp16 = (1.0f32 / 60.0f32).into();
    let mut automap = false;
    let mut stop_the_world_mode = false;
    let mut mouse_grabbed = false;
    let mut initial_ungrabbed = true;
    // Limit to max ~60 fps update rate
    // window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut spawn = SpawnInfo::StartLevel(0, None);
    'outer: loop {
        let mut map_dynamic;
        let level_id;
        let mut player;
        let mut things;

        match spawn {
            SpawnInfo::StartLevel(id, existing_static_map_data) => {
                match existing_static_map_data {
                    Some(StaticMapData {
                        map,
                        thing_defs,
                        level_id: y,
                    }) if id == y => {
                        println!("starting level. re-using static map data");
                        map_dynamic = MapDynamic::wrap(map);
                        things = Things::from_thing_defs(thing_defs);
                        level_id = y;
                    }
                    _ => {
                        println!("starting level. load static map data {}", maps.get_map_name(id));
                        let (plane0, plane1) = maps.get_map_planes(id);

                        map_dynamic = MapDynamic::wrap(Map::from_map_planes(&plane0, &plane1));
                        things = Things::from_thing_defs(ThingDefs::from_map_plane(&plane1));
                        level_id = id;
                    }
                }

                player = things
                    .thing_defs
                    .get_player_start()
                    .map(|(x, y, rot)| Player {
                        x,
                        y,
                        rot,
                        trigger: false,
                        shoot: false,
                        shoot_timeout: 0,
                    })
                    .unwrap_or_default();
            }

            SpawnInfo::LoadSavegame(existing_static_map_data) => {
                let mut f = std::fs::File::open("save.bin").unwrap();
                level_id = f.read_i32::<LittleEndian>().unwrap();

                player = Player::read_from(&mut f).expect("failed to load Player from savegame");
                match existing_static_map_data {
                    Some(StaticMapData {
                        map,
                        thing_defs,
                        level_id: y,
                    }) if level_id == y => {
                        println!("load savegame. re-using static map data");
                        map_dynamic =
                            MapDynamic::read_and_wrap(&mut f, map).expect("failed to load MapDynamic from savegame");
                        things = Things::read_from(&mut f, thing_defs).expect("failed to load Things from savegame");
                    }
                    _ => {
                        println!("load savegame. load static map data {}", maps.get_map_name(level_id));
                        let (plane0, plane1) = maps.get_map_planes(level_id);

                        map_dynamic = MapDynamic::read_and_wrap(&mut f, Map::from_map_planes(&plane0, &plane1))
                            .expect("failed to load MapDynamic from savegame");
                        things = Things::read_from(&mut f, ThingDefs::from_map_plane(&plane1))
                            .expect("failed to load Things from savegame");
                    }
                }
            }
        }

        let mut player_vel = PlayerVel {
            forward: 0,
            right: 0,
            rot: 0,
        };

        loop {
            let input_events = InputState::new(&mut events);

            if input_events.quit {
                break 'outer;
            }
            player_vel.forward = 0;
            player_vel.right = 0;
            player_vel.rot = 0;
            player.trigger = false;
            player.shoot = false;
            let (fwd_speed, rot_speed) = if input_events.slow { (2, 360) } else { (7, 3 * 360) };

            if input_events.forward {
                player_vel.forward += fwd_speed;
            }
            if input_events.backward {
                player_vel.forward -= fwd_speed;
            }

            if input_events.strafe_right {
                player_vel.right -= fwd_speed;
            }
            if input_events.strafe_left {
                player_vel.right += fwd_speed; // FIXME: strafe dirs seem mixed up
            }
            if input_events.turn_right {
                player_vel.rot += rot_speed;
            }
            if input_events.turn_left {
                player_vel.rot -= rot_speed;
            }
            if mouse_grabbed {
                player_vel.rot += input_events.dx * 300;
            }
            automap ^= input_events.toggle_automap;
            if input_events.open {
                player.trigger = true;
            }

            stop_the_world_mode ^= input_events.toggle_stop_the_world;
            let fast_forward = input_events.fast_forward_mode;

            player.shoot = input_events.shoot;

            let num_ticks = if stop_the_world_mode {
                0
            } else if fast_forward {
                10
            } else {
                1
            };
            for _ in 0..num_ticks {
                things.player_x = player.x.get_int();
                things.player_y = player.y.get_int();
                things.update(&player, &mut map_dynamic);
                map_dynamic.update(&player);
            }
            player.apply_vel(&player_vel, dt, &map_dynamic, !stop_the_world_mode);

            // println!("player: {:?} {:?} {:?}", player_vel, player.x, player.y);
            // println!("player: {:?}", player);

            for (i, chunk) in buffer.chunks_mut(320 * HALF_HEIGHT as usize).enumerate() {
                if i == 0 {
                    chunk.fill(29);
                } else if i == 1 {
                    chunk.fill(26);
                } else {
                    chunk.fill(155);
                }
            }

            player.draw(&mut buffer[..]);

            let _start = Instant::now();

            // for _ in 0..1000 {
            render::sweep_raycast(
                &map_dynamic,
                &mut buffer[..],
                &mut zbuffer,
                &player,
                0..WIDTH,
                &resources,
            );

            let _sprite_start = Instant::now();

            // draw_sprite(&mut buffer, &zbuffer, &resources, 8, 100, sprite_z.into());
            // if frame % 4 == 0 {
            let sprite_screen_setup = sprite::setup_screen_pos_for_player(things.get_sprites(), &player);

            let mut hit_thing = None;
            if player.shoot && player.shoot_timeout <= 0 {
                player.shoot_timeout = 30;

                for sprite in &sprite_screen_setup {
                    const WIDTH_HALF: i32 = (WIDTH as i32) / 2;

                    let zbound = zbuffer[WIDTH_HALF as usize];
                    if !(things.things[sprite.owner].actor.can_be_shot() && sprite.z < zbound) {
                        continue;
                    }
                    // FIXME: this is quite redundant with the calculations in sprite drawings. Maybe store the bounds in the screenspace setup struct.
                    const C: i32 = MID;
                    let offs = if sprite.z > FP16_ZERO {
                        (C << FP16_SCALE) / sprite.z.v
                    } else {
                        C
                    };
                    if !((sprite.screen_x + offs >= 0) && (sprite.screen_x - offs < WIDTH as i32)) {
                        continue;
                    }

                    println!(
                        "offs: {offs} {} {:?}",
                        sprite.screen_x, things.things[sprite.owner].actor
                    );
                    let offs_scale = 2; // fixme: general fettgesicht is probably wider...
                    let min = (WIDTH as i32 / 2) - offs / offs_scale;
                    let max = (WIDTH as i32 / 2) + offs / offs_scale;
                    if (min..max).contains(&sprite.screen_x) {
                        hit_thing = Some(sprite.owner);
                    }
                }
            }
            if player.shoot_timeout > 0 {
                player.shoot_timeout -= 1;
            }

            // let sprite_start = Instant::now();
            sprite::draw(sprite_screen_setup, &mut buffer[..], &zbuffer, &resources);

            // println!("sprite: {}us", sprite_start.elapsed().as_micros());

            if automap {
                map_dynamic.map.draw_automap(&mut buffer[..]);
                things.draw_automap(&mut buffer[..]);
            }

            buffer.point(320 / 2, 80, 4);
            if player.shoot {
                if let Some(hit_thing) = hit_thing {
                    if let Some((x, y)) = &things.things[hit_thing].actor.get_pos() {
                        let dx = player.x.get_int().abs_diff(x.get_int());
                        let dy = player.y.get_int().abs_diff(y.get_int());
                        let boost = 5 - dx.max(dy).min(5);
                        let base_hitpoints = 7;
                        let hitpoints = base_hitpoints + ((boost * 7) * (random::<u8>() as u32)) / 255;
                        println!("hit: {}", hitpoints - base_hitpoints);
                        things.things[hit_thing].actor.shoot(hitpoints as i32);
                    }
                }
            }

            // }
            // println!(
            //     "time: {}us\t({}us sprite)",
            //     start.elapsed().as_micros(),
            //     sprite_start.elapsed().as_micros()
            // );

            // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
            // window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();

            texture
                .with_lock(None, |tex_buffer: &mut [u8], pitch: usize| {
                    for y in 0..200 {
                        for x in 0..320 {
                            let offset = y * pitch + x * 3;
                            let s_offset = y * 320 + x;
                            let c32 = PALETTE[buffer[s_offset] as usize];
                            tex_buffer[offset] = (c32 >> 16) as u8;
                            tex_buffer[offset + 1] = (c32 >> 8) as u8;
                            tex_buffer[offset + 2] = c32 as u8;
                        }
                    }
                })
                .unwrap();
            canvas.clear();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();

            if input_events.toggle_mouse_grab || (input_events.shoot && initial_ungrabbed) {
                mouse_grabbed = !mouse_grabbed;
                canvas.window_mut().set_grab(mouse_grabbed);
                sdl_context.mouse().set_relative_mouse_mode(mouse_grabbed);
                initial_ungrabbed = false;
            }

            if input_events.restart {
                spawn = SpawnInfo::StartLevel(
                    level_id,
                    Some(StaticMapData {
                        level_id,
                        map: map_dynamic.release(),
                        thing_defs: things.release(),
                    }),
                );
                break;
            }
            if input_events.prev_level && level_id > 0 {
                spawn = SpawnInfo::StartLevel(
                    level_id - 1,
                    Some(StaticMapData {
                        level_id,
                        map: map_dynamic.release(),
                        thing_defs: things.release(),
                    }),
                );
                break;
            }
            if input_events.next_level && level_id < 99 {
                spawn = SpawnInfo::StartLevel(
                    level_id + 1,
                    Some(StaticMapData {
                        level_id,
                        map: map_dynamic.release(),
                        thing_defs: things.release(),
                    }),
                );
                break;
            }
            if input_events.save {
                let mut f = std::fs::File::create("save.bin").unwrap();
                f.write_i32::<LittleEndian>(level_id).unwrap();
                player.write(&mut f).expect("failed to write Player to savegame");
                map_dynamic
                    .write(&mut f)
                    .expect("failed to write MapDynamic to savegame");
                things.write(&mut f).expect("failed to write Things to savegame");
            }
            if input_events.load {
                spawn = SpawnInfo::LoadSavegame(Some(StaticMapData {
                    level_id,
                    map: map_dynamic.release(),
                    thing_defs: things.release(),
                }));
                break;
            }
        }
    }
    Ok(())
}
