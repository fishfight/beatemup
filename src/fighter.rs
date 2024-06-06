use bevy::prelude::*;
use rand::prelude::SliceRandom;
use serde::Deserialize;

use crate::attack::Hurtbox;
use crate::consts::{self, FOOT_PADDING};
use crate::metadata::ItemMeta;
use crate::{
    animation::{AnimatedSpriteSheetBundle, Animation, Facing},
    camera::YSort,
    collision::{BodyLayers, PhysicsBundle},
    damage::{Damageable, Health},
    enemy::Enemy,
    fighter_state::{Idling, StateTransitionIntents},
    metadata::{AttackMeta, FighterMeta},
    movement::LinearVelocity,
    player::Player,
};

pub struct FighterPlugin;

impl Plugin for FighterPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AvailableAttacks>()
            .add_systems(PostUpdate, attachment_system);
    }
}

/// Bundle added to a fighter stub, in order to activate it.
#[derive(Bundle)]
pub struct ActiveFighterBundle {
    pub name: Name,
    pub animated_spritesheet_bundle: AnimatedSpriteSheetBundle,
    // #[bundle]
    // pub physics_bundle: PhysicsBundle,
    pub stats: Stats,
    pub ysort: YSort,
    pub health: Health,
    pub damageable: Damageable,
    pub inventory: Inventory,
    pub state_transition_intents: StateTransitionIntents,
    /// Fighters start off idling, but this component may be removed when the fighter state changes.
    pub idling: Idling,
    pub velocity: LinearVelocity,
    pub available_attacks: AvailableAttacks,
}

/// Component that defines the currently available attacks on a fighter, modified at runtime when
/// picking up and dropping items, or potentially on other conditions.
#[derive(Component, Reflect, Clone, Default)]
#[reflect(Component)]
pub struct AvailableAttacks {
    pub attacks: Vec<AttackMeta>,
}

impl AvailableAttacks {
    pub fn current_attack(&self) -> &AttackMeta {
        self.attacks.last().expect("No attacks available")
    }
}

#[derive(Component, Deserialize, Clone, Debug, Reflect)]
#[reflect(Component)]
#[serde(deny_unknown_fields)]
pub struct Stats {
    pub max_health: i32,
    pub movement_speed: f32,
}

/// The player inventory.
///
/// A player may be holding one item
#[derive(Component, Clone, Debug, Default, Deref, DerefMut)]
pub struct Inventory(pub Option<ItemMeta>);

impl Default for Stats {
    fn default() -> Self {
        Stats {
            max_health: 100,
            movement_speed: 17000.,
        }
    }
}

/// Turns a fighter stub data (loaded from the metadata) into a fully active fighter.
impl ActiveFighterBundle {
    pub fn activate_fighter_stub(
        commands: &mut Commands,
        fighter: &FighterMeta,
        entity: Entity,
        transform: &Transform,
        player: Option<&Player>,
        enemy: Option<&Enemy>,
    ) {
        let body_layers = if player.is_some() {
            BodyLayers::PLAYER
        } else if enemy.is_some() {
            BodyLayers::ENEMY
        } else {
            unreachable!();
        };

        let active_fighter_bundle = ActiveFighterBundle {
            name: Name::new(fighter.name.clone()),
            animated_spritesheet_bundle: AnimatedSpriteSheetBundle {
                sprite_sheet: SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        anchor: bevy::sprite::Anchor::Custom(Vec2::new(
                            0.,
                            //calculate anchor to align with feet
                            0.5 * FOOT_PADDING / fighter.center_y - 0.5,
                        )),
                        ..default()
                    },
                    texture_atlas: fighter
                        .spritesheet
                        .atlas_handle
                        .choose(&mut rand::thread_rng())
                        .unwrap()
                        .clone(),
                    transform: *transform,
                    ..Default::default()
                },
                animation: Animation::new(
                    fighter.spritesheet.animation_fps,
                    fighter.spritesheet.animations.clone(),
                ),
            },
            stats: fighter.stats.clone(),
            health: Health(fighter.stats.max_health),
            inventory: default(),
            damageable: default(),
            // physics_bundle: PhysicsBundle::new(&fighter.hurtbox, body_layers),
            idling: Idling,
            state_transition_intents: default(),
            // ysort: YSort(fighter.spritesheet.tile_size.y as f32 / 2.),
            ysort: YSort(consts::FIGHTERS_Z),
            velocity: default(),
            available_attacks: AvailableAttacks {
                attacks: fighter.attacks.clone(),
            },
        };
        let hurtbox = commands
            .spawn((
                PhysicsBundle::new(&fighter.hurtbox, body_layers),
                TransformBundle::from_transform(Transform::from_xyz(
                    0.0,
                    fighter.collision_offset,
                    0.0,
                )),
                Hurtbox,
            ))
            .id();

        let animated_spritesheet_bundle = active_fighter_bundle.animated_spritesheet_bundle.clone();

        commands
            .entity(entity)
            .insert(active_fighter_bundle)
            .push_children(&[hurtbox]);

        if let Some(attachment) = &fighter.attachment {
            //Clone fighter spritesheet
            let mut attachment_spritesheet = animated_spritesheet_bundle;

            //Change what's needed
            attachment_spritesheet.sprite_sheet.texture_atlas = attachment
                .atlas_handle
                .choose(&mut rand::thread_rng())
                .unwrap()
                .clone();
            attachment_spritesheet.animation =
                Animation::new(attachment.animation_fps, attachment.animations.clone());
            attachment_spritesheet.sprite_sheet.transform = Transform::from_xyz(
                0.,
                fighter.spritesheet.tile_size.y as f32
                    * -(0.5 * FOOT_PADDING / fighter.center_y - 0.5), //This comes from the fighter anchor
                0.1,
            );
            attachment_spritesheet.sprite_sheet.sprite.anchor = bevy::sprite::Anchor::Center;

            let attachment_ent = commands
                .spawn(attachment_spritesheet)
                .insert(Attached {
                    position_face: false,
                    sync_animation: true,
                    sync_facing: true,
                })
                .insert(Facing::default())
                .id();
            commands.entity(entity).add_child(attachment_ent);
        }
    }
}

#[derive(Component)]
pub struct Attached {
    /// Syncs facing with parent facing
    pub sync_facing: bool,
    // Syncs animation with parent animation
    pub sync_animation: bool,
    /// Change position based on facing
    pub position_face: bool,
}

pub fn attachment_system(
    mut attached: Query<(
        &Parent,
        &Attached,
        &mut Transform,
        &mut Facing,
        &mut Animation,
    )>,
    parents: Query<(Entity, &Facing, &Animation), (With<Children>, Without<Attached>)>,
) {
    for (parent_ent, parent_facing, parent_animation) in &parents {
        for (parent, attached, mut transform, mut facing, mut animation) in &mut attached {
            if parent_ent == parent.get() {
                //Sync facing
                if attached.sync_facing {
                    *facing = parent_facing.clone();
                }

                //Sync animation
                if attached.sync_animation {
                    animation.current_frame = parent_animation.current_frame;
                    animation
                        .current_animation
                        .clone_from(&parent_animation.current_animation);
                    animation.timer = parent_animation.timer.clone();
                    animation.played_once = parent_animation.played_once;
                }
            }

            // Change position
            if attached.position_face {
                transform.translation.x = if facing.is_left() {
                    -transform.translation.x.abs()
                } else {
                    transform.translation.x.abs()
                };
            }
        }
    }
}
