use std::time::Instant;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use rand::random;
// use minifb::{Key, KeyRepeat, Window, WindowOptions};
use crate::{
    ms::{Loadable, Writable},
    prelude::*,
    sprite::SpriteSceenSetup,
    wl6::MapsFile,
    Resources,
};

include!("out.img.enums");

pub struct StaticMapData {
    pub level_id: i32,
    pub map: Map,
    pub thing_defs: ThingDefs,
}

pub enum SpawnInfo {
    StartLevel(i32, Option<StaticMapData>),
    LoadSavegame(Option<StaticMapData>),
}

#[derive(Default)]
pub struct InputState {
    // one-shot events
    pub quit: bool,
    pub restart: bool,
    pub prev_level: bool,
    pub next_level: bool,
    pub save: bool,
    pub load: bool,
    pub toggle_automap: bool,
    pub toggle_stop_the_world: bool,
    pub toggle_mouse_grab: bool,
    pub select_weapon: Option<i32>,

    // press state
    pub forward: bool,
    pub backward: bool,
    pub turn_left: bool,
    pub turn_right: bool,
    pub strafe_left: bool,
    pub strafe_right: bool,
    pub slow: bool,
    pub open: bool,
    pub shoot: bool,
    pub fast_forward_mode: bool,
    pub toggle_render_alternative: bool,

    // mouse
    pub dx: i32,
    pub dy: i32,

    pub misc_selection: i32,
}

impl InputState {
    pub fn is_deconstruct(&self) -> bool {
        self.load || self.next_level || self.prev_level || self.restart
    }
}

pub struct Mainloop {
    // resources: Resources,
    map_dynamic: MapDynamic,
    things: Things,
    pub player: Player,
    level_id: i32,
    pub map_name: String,
    player_vel: PlayerVel,
    automap: bool,
    stop_the_world_mode: bool,
    pub use_mouse_move: bool, // needs to be managed from outside
    entities: Entities,
}

impl Mainloop {
    pub fn spawn(spawn: SpawnInfo, maps: &mut MapsFile) -> Mainloop {
        let map_dynamic;
        let level_id;
        let player;
        let things;

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
                        weapon: Default::default(), // TODO
                        health: 100,
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

        let player_vel = PlayerVel {
            forward: 0,
            right: 0,
            rot: 0,
        };

        let entities = Entities::new();
        Mainloop {
            map_dynamic,
            things,
            player,
            level_id,
            map_name: maps.get_map_name(level_id).to_string(),
            player_vel,
            automap: false,
            stop_the_world_mode: false,
            use_mouse_move: false,
            entities,
        }
    }

    pub fn run(&mut self, input_events: &InputState, buffer: &mut [u8], resources: &Resources) {
        let dt: Fp16 = (1.0f32 / 60.0f32).into();
        let mut zbuffer = [Fp16::default(); WIDTH];

        if input_events.quit {
            return;
        }
        self.player_vel.forward = 0;
        self.player_vel.right = 0;
        self.player_vel.rot = 0;
        self.player.trigger = false;
        self.player.shoot = false;
        let (fwd_speed, rot_speed) = if input_events.slow { (2, 360) } else { (7, 3 * 360) };

        if input_events.forward {
            self.player_vel.forward += fwd_speed;
        }
        if input_events.backward {
            self.player_vel.forward -= fwd_speed;
        }

        if input_events.strafe_right {
            self.player_vel.right -= fwd_speed;
        }
        if input_events.strafe_left {
            self.player_vel.right += fwd_speed; // FIXME: strafe dirs seem mixed up
        }
        if input_events.turn_right {
            self.player_vel.rot += rot_speed;
        }
        if input_events.turn_left {
            self.player_vel.rot -= rot_speed;
        }
        if self.use_mouse_move {
            self.player_vel.rot += input_events.dx * 300;
        }
        self.automap ^= input_events.toggle_automap;
        if input_events.open {
            self.player.trigger = true;
        }

        self.stop_the_world_mode ^= input_events.toggle_stop_the_world;
        let fast_forward = input_events.fast_forward_mode;

        self.player.shoot = input_events.shoot;

        let num_ticks = if self.stop_the_world_mode {
            0
        } else if fast_forward {
            10
        } else {
            1
        };

        for room_id in &self.map_dynamic.notifications {
            println!("notify room {room_id:x}");
        }

        for _ in 0..num_ticks {
            self.things.player_x = self.player.x.get_int();
            self.things.player_y = self.player.y.get_int();
            self.things.update(&mut self.player, &mut self.map_dynamic);
            self.map_dynamic.update(&self.player);
        }
        self.player
            .apply_vel(&self.player_vel, dt, &self.map_dynamic, !self.stop_the_world_mode);

        // println!("player: {:?} {:?} {:?}", self.player_vel, player.x, player.y);
        // println!("player: {:?}", player);

        let ceiling_color = [
            0x1d, 0x1d, 0x1d, 0x1d, 0x1d, 0x1d, 0x1d, 0x1d, 0x1d, 0xbf, 0x4e, 0x4e, 0x4e, 0x1d, 0x8d, 0x4e, 0x1d, 0x2d,
            0x1d, 0x8d, 0x1d, 0x1d, 0x1d, 0x1d, 0x1d, 0x2d, 0xdd, 0x1d, 0x1d, 0x98, 0x1d, 0x9d, 0x2d, 0xdd, 0xdd, 0x9d,
            0x2d, 0x4d, 0x1d, 0xdd, 0x7d, 0x1d, 0x2d, 0x2d, 0xdd, 0xd7, 0x1d, 0x1d, 0x1d, 0x2d, 0x1d, 0x1d, 0x1d, 0x1d,
            0xdd, 0xdd, 0x7d, 0xdd, 0xdd, 0xdd,
        ];
        for (i, chunk) in buffer.chunks_mut(320 * HALF_HEIGHT as usize).enumerate() {
            if i == 0 {
                // chunk.fill(29);
                chunk.fill(ceiling_color[self.level_id as usize]);
            } else if i == 1 {
                chunk.fill(26);
            } else {
                chunk.fill(155);
            }
        }

        self.player.draw(&mut buffer[..]);

        let _start = Instant::now();

        // for _ in 0..1000 {
        render::sweep_raycast(
            &self.map_dynamic,
            &mut buffer[..],
            &mut zbuffer,
            &self.player,
            0..WIDTH,
            resources,
        );

        let _sprite_start = Instant::now();

        // draw_sprite(&mut buffer, &zbuffer, &resources, 8, 100, sprite_z.into());
        // if frame % 4 == 0 {
        let mut sprite_screen_setup = sprite::setup_screen_pos_for_player(self.things.get_sprites(), &self.player);

        let mut hit_thing = None;
        if self.player.weapon.run(input_events.shoot, input_events.select_weapon) {
            if let Some(room_id) = self
                .map_dynamic
                .get_room_id(self.player.x.get_int(), self.player.y.get_int())
            {
                self.map_dynamic.notifications.insert(room_id);
            }

            for sprite in &sprite_screen_setup {
                const WIDTH_HALF: i32 = (WIDTH as i32) / 2;

                let zbound = zbuffer[WIDTH_HALF as usize];
                if !(self.things.things[sprite.owner].actor.can_be_shot() && sprite.z < zbound) {
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
                    sprite.screen_x, self.things.things[sprite.owner].actor
                );
                let offs_scale = 2; // fixme: general fettgesicht is probably wider...
                let min = (WIDTH as i32 / 2) - offs / offs_scale;
                let max = (WIDTH as i32 / 2) + offs / offs_scale;
                if (min..max).contains(&sprite.screen_x) {
                    hit_thing = Some(sprite.owner);
                }
            }
        }
        self.entities.update();
        sprite_screen_setup.push(self.player.weapon.get_sprite());
        sprite_screen_setup.append(&mut self.entities.get_sprites());
        if input_events.misc_selection > 0 {
            sprite_screen_setup.push(SpriteSceenSetup {
                z: FP16_ZERO,
                screen_x: WIDTH as i32 / 2,
                id: input_events.misc_selection,
                owner: 0,
            });
            let name = ENUM_NAMES
                .iter()
                .find(|(_, id)| *id == input_events.misc_selection)
                .unwrap()
                .0;

            draw_string8x8(name, &mut buffer[..], 100, 160);
        }

        if self.player.shoot_timeout > 0 {
            self.player.shoot_timeout -= 1;
        }

        // let sprite_start = Instant::now();
        sprite::draw(sprite_screen_setup, &mut buffer[..], &zbuffer, resources);

        // println!("sprite: {}us", sprite_start.elapsed().as_micros());

        if self.automap {
            self.map_dynamic.map.draw_automap(&mut buffer[..]);
            self.things.draw_automap(&mut buffer[..]);
        }

        buffer.point(320 / 2, 80, 4);

        // draw_string8x8("Get Psyched!", &mut buffer[..], 100, 160);
        hud::draw_status_bar(&mut buffer[..], self);

        if self.player.shoot {
            if let Some(hit_thing) = hit_thing {
                if let Some((x, y)) = &self.things.things[hit_thing].actor.get_pos() {
                    let dx = self.player.x.get_int().abs_diff(x.get_int());
                    let dy = self.player.y.get_int().abs_diff(y.get_int());
                    let boost = 5 - dx.max(dy).min(5);
                    let base_hitpoints = 7;
                    let hitpoints = base_hitpoints + ((boost * 7) * (random::<u8>() as u32)) / 255;
                    println!("hit: {}", hitpoints - base_hitpoints);
                    self.things.things[hit_thing].actor.shoot(hitpoints as i32);
                }
            }
        }

        self.map_dynamic.propagate_notifications();

        // }
        // println!(
        //     "time: {}us\t({}us sprite)",
        //     start.elapsed().as_micros(),
        //     sprite_start.elapsed().as_micros()
        // );

        if input_events.save {
            let mut f = std::fs::File::create("save.bin").unwrap();
            f.write_i32::<LittleEndian>(self.level_id).unwrap();
            self.player.write(&mut f).expect("failed to write Player to savegame");
            self.map_dynamic
                .write(&mut f)
                .expect("failed to write MapDynamic to savegame");
            self.things.write(&mut f).expect("failed to write Things to savegame");
        }
    }

    pub fn deconstruct(self, input_events: &InputState) -> SpawnInfo {
        if input_events.prev_level && self.level_id > 0 {
            SpawnInfo::StartLevel(
                self.level_id - 1,
                Some(StaticMapData {
                    level_id: self.level_id,
                    map: self.map_dynamic.release(),
                    thing_defs: self.things.release(),
                }),
            )
        } else if input_events.next_level && self.level_id < 59 {
            SpawnInfo::StartLevel(
                self.level_id + 1,
                Some(StaticMapData {
                    level_id: self.level_id,
                    map: self.map_dynamic.release(),
                    thing_defs: self.things.release(),
                }),
            )
        } else if input_events.load {
            SpawnInfo::LoadSavegame(Some(StaticMapData {
                level_id: self.level_id,
                map: self.map_dynamic.release(),
                thing_defs: self.things.release(),
            }))
        } else {
            if !input_events.restart {
                // once this mehtod is called there is no turning back...
                println!("WARNING: fallback to level restart. level_id out of bounds?")
            }

            SpawnInfo::StartLevel(
                self.level_id,
                Some(StaticMapData {
                    level_id: self.level_id,
                    map: self.map_dynamic.release(),
                    thing_defs: self.things.release(),
                }),
            )
        }
    }
}
