use std::borrow::Cow;

use crate::{enemy::Enemy, ms::Loadable, prelude::*};
use anyhow::anyhow;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

#[derive(Debug, Default)]
pub enum Actor {
    Item {
        collected: bool,
        collectible: Collectible,
    },
    Enemy {
        enemy: Enemy,
    },
    #[default]
    None,
}

impl Actor {
    pub fn can_be_shot(&self) -> bool {
        matches!(self, Actor::Enemy { enemy: _ })
    }
    pub fn shoot(&mut self) {
        if let Actor::Enemy { enemy } = self {
            enemy.hit()
        }
    }
}

impl ms::Writable for Actor {
    fn write(&self, w: &mut dyn std::io::Write) -> Result<()> {
        match self {
            Actor::Item { collected, collectible } => {
                w.write_u8(0)?;
                w.write_u8(if *collected { 1 } else { 0 })?;
                collectible.write(w)?;
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
                collectible: Collectible::read_from(r)?,
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
    pub static_index: usize,
}

impl ms::Writable for Thing {
    fn write(&self, w: &mut dyn std::io::Write) -> Result<()> {
        w.write_i32::<LittleEndian>(self.static_index as i32)?; // FIXME
        self.actor.write(w)?;
        Ok(())
    }
}

impl ms::Loadable for Thing {
    fn read_from(r: &mut dyn std::io::Read) -> Result<Self> {
        let static_index = r.read_i32::<LittleEndian>()? as usize;
        let actor = Actor::read_from(r)?;
        Ok(Self { actor, static_index })
    }
}

pub struct Things {
    pub thing_defs: ThingDefs,
    pub things: Vec<Thing>,
    pub anim_timeout: i32,
}

impl ms::Writable for Things {
    fn write(&self, w: &mut dyn std::io::Write) -> Result<()> {
        w.write_u32::<LittleEndian>(self.things.len() as u32)?;
        for thing in &self.things {
            thing.write(w)?;
        }
        w.write_i32::<LittleEndian>(self.anim_timeout)?;
        Ok(())
    }
}

impl Things {
    pub fn read_from(r: &mut dyn std::io::Read, thing_defs: ThingDefs) -> Result<Self> {
        let num_things = r.read_u32::<LittleEndian>()?;
        let mut things = Vec::new();
        for _ in 0..num_things {
            things.push(Thing::read_from(r)?);
        }
        let anim_timeout = r.read_i32::<LittleEndian>()?;
        Ok(Self {
            thing_defs,
            things,
            anim_timeout,
        })
    }
    pub fn from_thing_defs(thing_defs: ThingDefs) -> Self {
        let mut things = Vec::new();

        for (i, thing_def) in thing_defs.thing_defs.iter().enumerate() {
            match thing_def.thing_type {
                ThingType::Enemy(direction, difficulty, enemy_type, state) => things.push(Thing {
                    static_index: i,
                    actor: Actor::Enemy {
                        enemy: Enemy::spawn(direction, difficulty, enemy_type, state, thing_def),
                    },
                }),
                ThingType::Prop(sprite_index) => {
                    let actor = try_to_collectible(sprite_index)
                        .map(|collectible| Actor::Item {
                            collected: false,
                            collectible,
                        })
                        .unwrap_or_default();
                    things.push(Thing { static_index: i, actor });
                }
                ThingType::PlayerStart(_) => (),
            }
        }

        Things {
            thing_defs,
            things,
            anim_timeout: 0,
        }
    }

    pub fn update(&mut self, player: &Player) {
        for thing in &mut self.things {
            if let Actor::Enemy { enemy } = &mut thing.actor {
                enemy.update();
            }

            let thing_def = &self.thing_defs.thing_defs[thing.static_index];
            #[allow(clippy::single_match)]
            match (thing_def.thing_type, &mut thing.actor) {
                (ThingType::Prop(_id), Actor::Item { collected, collectible }) if !*collected => {
                    // (ThingType::Prop(_id), _) => {
                    let dx = player.x - thing_def.x + FP16_HALF;
                    let dy = player.y - thing_def.y + FP16_HALF;
                    if dx.get_int().abs() == 0 && dy.get_int().abs() == 0 {
                        println!("collected: {thing_def:?} {collectible:?}");
                        *collected = true;
                        // println!("collected: {:?} ", thing_def);
                    }
                }
                _ => (),
            }
        }
    }
    pub fn get_sprites(&self) -> Vec<SpriteDef> {
        self.things
            .iter()
            .enumerate()
            .filter_map(|(i, thing)| {
                let thing_def = &self.thing_defs.thing_defs[thing.static_index];
                // println!("{:?} {:?}", thing_def.thing_type, thing.actor);
                match (thing_def.thing_type, &thing.actor) {
                    (ThingType::Enemy(_direction, _difficulty, _enemy_type, _state), Actor::Enemy { enemy }) => {
                        let (id, x, y) = enemy.get_sprite(); // + enemy_type.sprite_offset();
                        Some(SpriteDef { id, x, y, owner: i })
                    }
                    (ThingType::Prop(id), Actor::None) => Some(SpriteDef {
                        id: sprite::SpriteIndex::Undirectional(id - 22 + 2),
                        x: thing_def.x,
                        y: thing_def.y,
                        owner: i,
                    }),
                    _ => None,
                }
            })
            .collect()
    }

    pub fn release(self) -> ThingDefs {
        self.thing_defs
    }
}

#[derive(Debug)]
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

impl ms::Loadable for Collectible {
    fn read_from(r: &mut dyn std::io::Read) -> Result<Self> {
        let id = r.read_u8()?;
        try_to_collectible(id as i32).ok_or_else(|| anyhow!("unsupported discriminator for Collectible: {id}"))
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
