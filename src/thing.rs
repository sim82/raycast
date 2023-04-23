use std::collections::HashSet;

use crate::{enemy::Enemy, ms::Loadable, prelude::*};
use anyhow::anyhow;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

#[derive(Debug, Default)]
pub enum Actor {
    Item {
        collected: bool,
        item: Item,
    },
    Enemy {
        enemy: Enemy,
    },
    #[default]
    None,
}

impl Actor {
    pub fn can_be_shot(&self) -> bool {
        matches!(self, Actor::Enemy { enemy: Enemy { health, .. } } if *health > 0)
    }
    pub fn get_pos(&self) -> Option<(Fp16, Fp16)> {
        match self {
            Actor::Enemy { enemy } => Some((enemy.x, enemy.y)),
            _ => None,
        }
    }

    pub fn shoot(&mut self, hitpoints: i32) {
        if let Actor::Enemy { enemy } = self {
            enemy.hit(hitpoints)
        }
    }
}

impl ms::Writable for Actor {
    fn write(&self, w: &mut dyn std::io::Write) -> Result<()> {
        match self {
            Actor::Item { collected, item } => {
                w.write_u8(0)?;
                w.write_u8(if *collected { 1 } else { 0 })?;
                item.write(w)?;
            }
            Actor::Enemy { enemy } => {
                w.write_u8(1)?;
                enemy.write(w)?;
            }
            Actor::None => w.write_u8(2)?,
        }
        Ok(())
    }
}

impl ms::Loadable for Actor {
    fn read_from(r: &mut dyn std::io::Read) -> Result<Self> {
        Ok(match r.read_u8()? {
            0 => Actor::Item {
                collected: r.read_u8()? != 0,
                item: Item::read_from(r)?,
            },
            1 => Actor::Enemy {
                enemy: Enemy::read_from(r)?,
            },
            2 => Actor::None,
            x => return Err(anyhow!("unhandled Actor discriminator {x}")),
        })
    }
}

pub struct Thing {
    pub actor: Actor,
    pub unique_id: usize,
}

impl ms::Writable for Thing {
    fn write(&self, w: &mut dyn std::io::Write) -> Result<()> {
        w.write_i32::<LittleEndian>(self.unique_id as i32)?; // FIXME
        self.actor.write(w)?;
        Ok(())
    }
}

impl ms::Loadable for Thing {
    fn read_from(r: &mut dyn std::io::Read) -> Result<Self> {
        let unique_id = r.read_i32::<LittleEndian>()? as usize;
        let actor = Actor::read_from(r)?;
        Ok(Self { actor, unique_id })
    }
}

pub struct Things {
    pub thing_defs: ThingDefs,
    pub things: Vec<Thing>,
    pub anim_timeout: i32,
    pub blockmap: BlockMap,
    pub player_x: i32,
    pub player_y: i32,
}

impl ms::Writable for Things {
    fn write(&self, w: &mut dyn std::io::Write) -> Result<()> {
        w.write_u32::<LittleEndian>(self.things.len() as u32)?;
        for thing in &self.things {
            thing.write(w)?;
        }
        w.write_i32::<LittleEndian>(self.anim_timeout)?;
        self.blockmap.write(w)?;
        Ok(())
    }
}

impl Things {
    fn spawn_from_thing_def(
        thing_def: &ThingDef,
        blockmap: &mut BlockMap,
        unique_id: usize,
    ) -> Option<Thing> {
        let thing = match &thing_def.thing_type {
            ThingType::Enemy(enemy_spawn_info) => {
                let enemy = Enemy::spawn(enemy_spawn_info, thing_def);

                blockmap.insert(unique_id, enemy.x, enemy.y);
                Thing {
                    unique_id,
                    actor: Actor::Enemy { enemy },
                }
            }
            ThingType::Prop(sprite_index) => {
                let actor = try_to_collectible(*sprite_index)
                    .map(|collectible| Actor::Item {
                        collected: false,
                        item: Item {
                            collectible,
                            id: *sprite_index,
                            x: thing_def.x,
                            y: thing_def.y,
                        },
                    })
                    .unwrap_or_default();
                Thing { unique_id, actor }
            }
            ThingType::PlayerStart(_) => return None,
        };
        Some(thing)
    }

    pub fn read_from(r: &mut dyn std::io::Read, thing_defs: ThingDefs) -> Result<Self> {
        let num_things = r.read_u32::<LittleEndian>()?;
        let mut things = Vec::new();
        for _ in 0..num_things {
            things.push(Thing::read_from(r)?);
        }
        let anim_timeout = r.read_i32::<LittleEndian>()?;
        let blockmap = BlockMap::read_from(r)?;

        Ok(Self {
            thing_defs,
            things,
            anim_timeout,
            blockmap,
            player_x: 0,
            player_y: 0,
        })
    }
    pub fn from_thing_defs(thing_defs: ThingDefs) -> Self {
        let mut things = Vec::new();
        let mut blockmap = BlockMap::default();
        for (i, thing_def) in thing_defs.thing_defs.iter().enumerate() {
            let thing = match Self::spawn_from_thing_def(thing_def, &mut blockmap, i) {
                Some(value) => value,
                None => continue,
            };

            things.push(thing);
        }

        Things {
            thing_defs,
            things,
            anim_timeout: 0,
            blockmap,
            player_x: 0,
            player_y: 0,
        }
    }

    pub fn update(&mut self, player: &mut Player, map_dynamic: &mut MapDynamic) {
        // temporarily take out things during mutation
        let mut things = std::mem::take(&mut self.things);
        let mut new_notifications = HashSet::new();
        let mut spawn_thing_defs = Vec::new();

        for thing in &mut things {
            // let thing_def = &self.thing_defs.thing_defs[thing.static_index];
            #[allow(clippy::single_match)]
            match &mut thing.actor {
                Actor::Item { collected, item } if !*collected => {
                    // (ThingType::Prop(_id), _) => {
                    let dx = player.x - item.x + FP16_HALF;
                    let dy = player.y - item.y + FP16_HALF;
                    if dx.get_int().abs() == 0 && dy.get_int().abs() == 0 {
                        match item.collectible {
                            Collectible::Ammo if player.weapon.ammo < 100 => {
                                player.weapon.ammo = (player.weapon.ammo + 8).min(100);
                                *collected = true;
                            }
                            Collectible::Food | Collectible::Medkit | Collectible::DogFood
                                if player.health < 100 =>
                            {
                                let add = match item.collectible {
                                    Collectible::Food => 8,
                                    Collectible::DogFood => 4,
                                    Collectible::Medkit => 25,
                                    _ => 0,
                                };
                                player.health = (player.health + add).min(100);
                                *collected = true;
                            }
                            _ => (),
                        }
                        // if *collected {
                        println!("collected: {:?}", item.collectible);
                        // println!("collected: {:?} ", thing_def);
                        // }
                    }
                }
                Actor::Enemy { enemy } if !enemy.dead => {
                    let old_x = enemy.x;
                    let old_y = enemy.y;

                    let was_notify = enemy.notify;
                    // check if enemy gets notified by the room for this frame
                    if !enemy.notify {
                        match map_dynamic.get_room_id(old_x.get_int(), old_y.get_int()) {
                            Some(room_id)
                                if map_dynamic.notifications.contains(&room_id)
                                    && !enemy.notify =>
                            {
                                println!("enemy {} notified by room {room_id}", thing.unique_id);
                                enemy.notify = true;
                            }
                            _ => (),
                        }
                    }
                    enemy.update(map_dynamic, self, thing.unique_id, player);

                    // update blockmal link
                    if !enemy.dead {
                        self.blockmap
                            .update(thing.unique_id, old_x, old_y, enemy.x, enemy.y);
                    } else {
                        self.blockmap.remove(thing.unique_id, old_x, old_y);

                        let thing_def = self.thing_defs.thing_defs.get(thing.unique_id);
                        match thing_def {
                            Some(ThingDef {
                                thing_type:
                                    ThingType::Enemy(EnemySpawnInfo {
                                        spawn_on_death: Some(id),
                                        ..
                                    }),
                                ..
                            }) => {
                                if let Some(thing_def) =
                                    ThingDef::from_map_id(*id, enemy.x, enemy.y)
                                {
                                    spawn_thing_defs.push(thing_def);
                                }
                            }
                            _ => (),
                        }
                    }

                    // if enemy raised the notify flag this frame, forward it to the room
                    if enemy.notify && !was_notify {
                        if let Some(room_id) =
                            map_dynamic.get_room_id(enemy.x.get_int(), enemy.y.get_int())
                        {
                            new_notifications.insert(room_id);
                        }
                    }
                }
                Actor::Enemy { enemy } if enemy.dead => {
                    enemy.update(map_dynamic, self, thing.unique_id, player);
                }
                _ => (),
            }
        }
        map_dynamic.notifications = new_notifications;

        // for actor in spawn_actors {
        //     things.push(Thing {
        //         actor,
        //         unique_id: things.len(), // TODO: rethink: as long as nothing is ever deleted from things this is probably good enough
        //     })
        // }
        for thing_def in spawn_thing_defs {
            // TODO: rethink: as long as nothing is ever deleted from things this is probably good enough
            if let Some(thing) =
                Self::spawn_from_thing_def(&thing_def, &mut self.blockmap, things.len())
            {
                things.push(thing);
            }
        }

        self.things = things;
    }
    pub fn get_sprites(&self) -> Vec<SpriteDef> {
        self.things
            .iter()
            .enumerate()
            .filter_map(|(i, thing)| {
                match &thing.actor {
                    Actor::Enemy { enemy } => {
                        let (id, x, y) = enemy.get_sprite(); // + enemy_type.sprite_offset();
                        Some(SpriteDef { id, x, y, owner: i })
                    }
                    Actor::Item {
                        collected: false,
                        item,
                    } => Some(SpriteDef {
                        id: sprite::SpriteIndex::Undirectional(item.id - 22 + 2),
                        x: item.x,
                        y: item.y,
                        owner: i,
                    }),
                    Actor::None => {
                        let thing_def = self.thing_defs.thing_defs.get(thing.unique_id);
                        if let Some(ThingDef {
                            thing_type: ThingType::Prop(id),
                            x,
                            y,
                        }) = thing_def
                        {
                            Some(SpriteDef {
                                id: sprite::SpriteIndex::Undirectional(id - 22 + 2),
                                x: *x,
                                y: *y,
                                owner: i,
                            })
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            })
            .collect()
    }

    pub fn draw_automap<D: Draw + ?Sized>(&self, screen: &mut D) {
        for thing in &self.things {
            if let Actor::Enemy { enemy } = &thing.actor {
                screen.point_world(enemy.x, enemy.y, 1);
            }
        }
    }

    pub fn release(self) -> ThingDefs {
        self.thing_defs
    }
}

fn try_to_collectible(sprite_index: i32) -> Option<Collectible> {
    Some(match sprite_index {
        29 => Collectible::DogFood,
        43 => Collectible::Key(0),
        44 => Collectible::Key(1),
        47 => Collectible::Food,
        48 => Collectible::Medkit,
        49 => Collectible::Ammo,
        50 => Collectible::Machinegun,
        51 => Collectible::Chaingun,
        52..=55 => Collectible::Treasure(sprite_index - 52),
        56 => Collectible::LifeUp,
        _ => return None,
    })
}
#[derive(Debug, Clone)]
pub enum Collectible {
    DogFood,
    Key(i32),
    Food,
    Medkit,
    Ammo,
    Machinegun,
    Chaingun,
    Treasure(i32),
    LifeUp,
}
impl ms::Loadable for Collectible {
    fn read_from(r: &mut dyn std::io::Read) -> Result<Self> {
        let id = r.read_u8()?;
        try_to_collectible(id as i32)
            .ok_or_else(|| anyhow!("unsupported discriminator for Collectible: {id}"))
    }
}

impl ms::Writable for Collectible {
    fn write(&self, w: &mut dyn std::io::Write) -> Result<()> {
        w.write_u8(match self {
            Collectible::DogFood => 29,
            Collectible::Key(t) => 43 + *t as u8,
            Collectible::Food => 47,
            Collectible::Medkit => 48,
            Collectible::Ammo => 49,
            Collectible::Machinegun => 50,
            Collectible::Chaingun => 51,
            Collectible::Treasure(t) => 52 + *t as u8,
            Collectible::LifeUp => 56,
        })?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Item {
    pub collectible: Collectible,
    pub id: i32,
    pub x: Fp16,
    pub y: Fp16,
}

impl ms::Loadable for Item {
    fn read_from(r: &mut dyn std::io::Read) -> Result<Self> {
        let collectible = Collectible::read_from(r)?;
        let id = r.read_i32::<LittleEndian>()?;
        let x = Fp16::read_from(r)?;
        let y = Fp16::read_from(r)?;
        Ok(Self {
            collectible,
            id,
            x,
            y,
        })
    }
}

impl ms::Writable for Item {
    fn write(&self, w: &mut dyn std::io::Write) -> Result<()> {
        self.collectible.write(w)?;
        w.write_i32::<LittleEndian>(self.id)?;
        self.x.write(w)?;
        self.y.write(w)?;
        Ok(())
    }
}
