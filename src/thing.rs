use std::borrow::Cow;

use crate::{enemy::Enemy, ms::Loadable, prelude::*, sprite::SpriteIndex};
use anyhow::anyhow;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

#[derive(Debug, Default)]
pub enum Actor {
    Item {
        collected: bool,
        collectible: Collectible,
    },
    Guard {
        pain: bool,
        health: i32,
    },
    Enemy {
        enemy: Enemy,
    },
    #[default]
    None,
}

impl Actor {
    pub fn can_be_shot(&self) -> bool {
        matches!(self, Actor::Guard { pain: _, health: _ })
    }
    pub fn shoot(&mut self) {
        if let Actor::Guard { pain, health: _ } = self {
            *pain = true
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
            Actor::Guard { pain, health } => {
                w.write_u8(1)?;
                w.write_u8(if *pain { 1 } else { 0 })?;
                w.write_i32::<LittleEndian>(*health)?;
            }
            Actor::Enemy { enemy } => todo!(),
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
            1 => Actor::Guard {
                pain: r.read_u8()? != 0,
                health: r.read_i32::<LittleEndian>()?,
            },
            2 => Actor::None,
            _ => panic!(),
        })
    }
}

#[derive(PartialEq, Eq)]
pub enum AnimMode {
    Oneshot(usize),
    Loop(usize),
    Singleframe,
    Finished,
}

impl ms::Loadable for AnimMode {
    fn read_from(r: &mut dyn std::io::Read) -> Result<Self> {
        Ok(match r.read_u8()? {
            0 => AnimMode::Oneshot(r.read_u32::<LittleEndian>()? as usize),
            1 => AnimMode::Loop(r.read_u32::<LittleEndian>()? as usize),
            2 => AnimMode::Singleframe,
            3 => AnimMode::Finished,
            x => return Err(anyhow!("unhandled AnimMode discriminator {x}")),
        })
    }
}

impl ms::Writable for AnimMode {
    fn write(&self, w: &mut dyn std::io::Write) -> Result<()> {
        match self {
            AnimMode::Oneshot(i) => {
                w.write_u8(0)?;
                w.write_u32::<LittleEndian>(*i as u32)?;
            }
            AnimMode::Loop(i) => {
                w.write_u8(1)?;
                w.write_u32::<LittleEndian>(*i as u32)?;
            }
            AnimMode::Singleframe => w.write_u8(2)?,
            AnimMode::Finished => w.write_u8(3)?,
        }
        Ok(())
    }
}

pub struct Thing {
    pub animation_frames: Cow<'static, [i32]>,
    pub directionality: Directionality,
    pub anim_mode: AnimMode,
    pub actor: Actor,
    pub static_index: usize,
}

impl ms::Writable for Thing {
    fn write(&self, w: &mut dyn std::io::Write) -> Result<()> {
        w.write_u32::<LittleEndian>(self.animation_frames.len() as u32)?;

        for f in self.animation_frames.iter() {
            w.write_i32::<LittleEndian>(*f)?;
        }
        self.anim_mode.write(w)?;
        self.directionality.write(w)?;
        w.write_i32::<LittleEndian>(self.static_index as i32)?; // FIXME
        self.actor.write(w)?;
        Ok(())
    }
}

impl ms::Loadable for Thing {
    fn read_from(r: &mut dyn std::io::Read) -> Result<Self> {
        let num_anim_frames = r.read_u32::<LittleEndian>()?;
        let mut animation_frames = Vec::new();
        for _ in 0..num_anim_frames {
            animation_frames.push(r.read_i32::<LittleEndian>()?);
        }
        let anim_mode = AnimMode::read_from(r)?;
        let anim_directionality = Directionality::read_from(r)?;
        let static_index = r.read_i32::<LittleEndian>()? as usize;
        let actor = Actor::read_from(r)?;
        Ok(Self {
            animation_frames: animation_frames.into(),
            anim_mode,
            directionality: anim_directionality,
            actor,
            static_index,
        })
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
                // ThingType::Enemy(direction, _, enemy_type, _) => things.push(Thing {
                //     animation_frames: enemy_type.animation_frames(AnimationPhase::Walk).into(),
                //     directionality: Directionality::Direction(direction),
                //     // sprite_index: 0,
                //     anim_mode: AnimMode::Loop(0),
                //     static_index: i,
                //     actor: Actor::Guard {
                //         pain: false,
                //         health: 20,
                //     },
                // }),
                ThingType::Enemy(direction, difficulty, enemy_type, state) => things.push(Thing {
                    animation_frames: enemy_type.animation_frames(AnimationPhase::Walk).into(),
                    directionality: Directionality::Direction(direction),
                    // sprite_index: 0,
                    anim_mode: AnimMode::Loop(0),
                    static_index: i,
                    actor: Actor::Enemy {
                        enemy: Enemy::spawn(direction, difficulty, enemy_type, state),
                    },
                }),
                ThingType::Prop(sprite_index) => {
                    let actor = try_to_collectible(sprite_index)
                        .map(|collectible| Actor::Item {
                            collected: false,
                            collectible,
                        })
                        .unwrap_or_default();
                    things.push(Thing {
                        animation_frames: Cow::Borrowed(&[]),
                        directionality: Directionality::Undirectional,
                        // sprite_index,
                        // anim_index: 0,
                        anim_mode: AnimMode::Singleframe,
                        static_index: i,
                        actor,
                    });
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
        self.anim_timeout -= 1;
        let update_anims = if self.anim_timeout <= 0 {
            self.anim_timeout = 10;

            true
        } else {
            false
        };

        for thing in &mut self.things {
            match &mut thing.actor {
                Actor::Guard { pain, health } if *pain => {
                    *health -= 7;
                    if let ThingType::Enemy(_, _, enemy_type, _) =
                        self.thing_defs.thing_defs[thing.static_index].thing_type
                    {
                        if *health > 0 {
                            thing.animation_frames = enemy_type.animation_frames(AnimationPhase::Pain).into();
                            thing.anim_mode = AnimMode::Oneshot(0);
                            thing.directionality = Directionality::Undirectional;
                        } else {
                            thing.animation_frames = enemy_type.animation_frames(AnimationPhase::Die).into();
                            thing.anim_mode = AnimMode::Oneshot(0);
                            thing.directionality = Directionality::Undirectional;
                        }
                    }
                    *pain = false;
                }
                Actor::Guard { pain: _, health } if thing.anim_mode == AnimMode::Finished => {
                    if *health <= 0 {
                        thing.actor = Actor::None;
                    } else if let ThingType::Enemy(direction, _, enemy_type, _) =
                        self.thing_defs.thing_defs[thing.static_index].thing_type
                    {
                        thing.animation_frames = enemy_type.animation_frames(AnimationPhase::Walk).into();
                        // thing.anim_index = 0;
                        thing.anim_mode = AnimMode::Loop(0);
                        thing.directionality = Directionality::Direction(direction);
                    }
                }
                Actor::Enemy { enemy } => {
                    enemy.update();
                }
                _ => (),
            }

            if update_anims {
                match &mut thing.anim_mode {
                    AnimMode::Oneshot(i) => {
                        if *i < thing.animation_frames.len() - 1 {
                            // thing.sprite_index = thing.animation_frames[*i];
                            *i += 1;
                        } else {
                            thing.anim_mode = AnimMode::Finished;
                        }
                    }
                    AnimMode::Loop(i) => {
                        // thing.sprite_index = thing.animation_frames[*i];
                        *i = (*i + 1) % thing.animation_frames.len();
                    }
                    _ => (),
                }
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
                    (ThingType::Enemy(direction, _difficulty, enemy_type, _state), Actor::Enemy { enemy }) => {
                        let id = enemy.get_sprite(&enemy_type); // + enemy_type.sprite_offset();
                        Some(SpriteDef {
                            id,
                            x: thing_def.x,
                            y: thing_def.y,
                            owner: i,
                        })
                    }
                    (ThingType::Enemy(_direction, _difficulty, _enemy_type, _state), _) => {
                        let id = match thing.anim_mode {
                            AnimMode::Oneshot(i) => thing.animation_frames[i],
                            AnimMode::Loop(i) => thing.animation_frames[i],
                            AnimMode::Singleframe => *thing.animation_frames.first().unwrap(),
                            AnimMode::Finished => *thing.animation_frames.last().unwrap(),
                        };
                        let id = match thing.directionality {
                            Directionality::Direction(d) => sprite::SpriteIndex::Directional(id, d),
                            Directionality::Undirectional => sprite::SpriteIndex::Undirectional(id),
                        };
                        Some(SpriteDef {
                            id,
                            x: thing_def.x,
                            y: thing_def.y,
                            owner: i,
                        })
                    }
                    (
                        ThingType::Prop(id),
                        Actor::Item {
                            collected: false,
                            collectible: _,
                        },
                    )
                    | (ThingType::Prop(id), Actor::None) => Some(SpriteDef {
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
