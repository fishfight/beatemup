use bevy::{ecs::system::EntityCommands, prelude::*};
// use bevy_mod_js_scripting::{ActiveScripts, JsScript};
use bevy_rapier2d::prelude::*;
use rand::Rng;

use crate::{
    animation::{AnimatedSpriteSheetBundle, Animation, Facing},
    attack::{Attack, AttackFrames, Breakable, BrokeEvent},
    collision::{BodyLayers, PhysicsBundle},
    consts,
    fighter::Inventory,
    lifetime::{Lifetime, LifetimeExpired},
    metadata::{AttackMeta, ItemKind, ItemMeta, ItemSpawnMeta},
    movement::{AngularVelocity, Force, LinearVelocity},
};

pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (drop_system, explodable_system));
    }
}

#[derive(Component)]
pub struct Item {
    /// Prevent the spawning of a Sprite component by load_items by setting this to false
    pub spawn_sprite: bool,
}

#[derive(Bundle)]
pub struct ItemBundle {
    pub item: Item,
    pub item_meta_handle: Handle<ItemMeta>,
    pub name: Name,
}

impl ItemBundle {
    pub fn new(item_spawn_meta: &ItemSpawnMeta) -> Self {
        Self {
            item: Item { spawn_sprite: true },
            item_meta_handle: item_spawn_meta.item_handle.clone(),
            // TODO: Actually include the item's name at some point
            name: Name::new("Map Item"),
        }
    }

    pub fn spawn(
        mut commands: EntityCommands,
        item_spawn_meta: &ItemSpawnMeta,
        items_assets: &mut ResMut<Assets<ItemMeta>>,
        // active_scripts: &mut ActiveScripts,
    ) {
        let ground_offset = Vec3::new(0.0, consts::GROUND_Y, consts::ITEM_LAYER);
        let transform_bundle = TransformBundle::from_transform(Transform::from_translation(
            item_spawn_meta.location + ground_offset,
        ));

        commands.insert(transform_bundle);

        let mut item = None;
        let item_meta = items_assets
            .get_mut(&item_spawn_meta.item_handle)
            .expect("Item not loaded!");
        match &item_meta.kind {
            ItemKind::BreakableBox {
                hurtbox,
                hits,
                item_handle,
                ..
            } => {
                item = Some(item_handle.clone());

                let mut physics_bundle = PhysicsBundle::new(hurtbox, BodyLayers::BREAKABLE_ITEM);
                physics_bundle.collision_groups.filters = BodyLayers::PLAYER_ATTACK;

                commands.insert((physics_bundle, Breakable::new(*hits, false)));
            }
            // ItemKind::Script { script_handle, .. } => {
            //     active_scripts.insert(script_handle.clone());
            // }
            _ => (),
        }

        if let Some(item) = item {
            commands.insert(Drop {
                item: items_assets.get(&item).expect("Item not loaded!").clone(),
            });
        }
    }
}

#[derive(Bundle)]
pub struct Projectile {
    sprite_bundle: SpriteBundle,
    velocity: LinearVelocity,
    angular_velocity: AngularVelocity,
    force: Force,
    collider: Collider,
    sensor: Sensor,
    events: ActiveEvents,
    collision_types: ActiveCollisionTypes,
    collision_groups: CollisionGroups,
    attack: Attack,
    lifetime: Lifetime,
    breakable: Breakable,
}

impl Projectile {
    pub fn from_thrown_item(
        translation: Vec3,
        item_meta: &ItemMeta,
        facing: &Facing,
        enemy: bool,
    ) -> Self {
        let direction_mul = if facing.is_left() {
            Vec2::new(-1.0, 1.0)
        } else {
            Vec2::ONE
        };

        let item_vars = match item_meta.kind {
            crate::metadata::ItemKind::Throwable {
                damage,
                gravity,
                throw_velocity,
                lifetime,
                pushback,
                hitstun_duration,
                ..
            }
            | crate::metadata::ItemKind::BreakableBox {
                damage,
                gravity,
                throw_velocity,
                lifetime,
                pushback,
                hitstun_duration,
                ..
            } => Some((
                damage,
                gravity,
                throw_velocity,
                lifetime,
                pushback,
                hitstun_duration,
            )),
            _ => None,
        }
        .expect("Non throwable item");

        Self {
            sprite_bundle: SpriteBundle {
                texture: item_meta.image.image_handle.clone(),
                transform: Transform::from_xyz(translation.x, translation.y, consts::PROJECTILE_Z),
                ..default()
            },
            attack: Attack {
                damage: item_vars.0,
                pushback: Vec2::new(item_vars.4, 0.0) * direction_mul,
                hitstun_duration: item_vars.5,
                hitbox_meta: None,
            },
            velocity: LinearVelocity(item_vars.2 * direction_mul),
            // Gravity
            force: Force(Vec2::new(0.0, -item_vars.1)),
            angular_velocity: AngularVelocity(consts::THROW_ITEM_ROTATION_SPEED * direction_mul.x),
            collider: Collider::cuboid(consts::ITEM_WIDTH / 2., consts::ITEM_HEIGHT / 2.),
            sensor: Sensor,
            events: ActiveEvents::COLLISION_EVENTS,
            collision_types: ActiveCollisionTypes::default() | ActiveCollisionTypes::STATIC_STATIC,
            //TODO: define collision layer based on the fighter shooting projectile, load for asset
            //files of fighter which "team" they are on
            collision_groups: CollisionGroups::new(
                if enemy {
                    BodyLayers::ENEMY_ATTACK
                } else {
                    BodyLayers::PLAYER_ATTACK
                },
                if enemy {
                    BodyLayers::PLAYER
                } else {
                    BodyLayers::ENEMY | BodyLayers::BREAKABLE_ITEM
                },
            ),
            lifetime: Lifetime(Timer::from_seconds(item_vars.3, TimerMode::Once)),
            breakable: Breakable::new(0, false),
        }
    }
}

/// A component that with Breakable, drops a item when broke.
#[derive(Component, Clone)]
pub struct Drop {
    /// Item data
    pub item: ItemMeta,
}

fn drop_system(
    mut items_assets: ResMut<Assets<ItemMeta>>,
    mut commands: Commands,
    mut broke_event: EventReader<BrokeEvent>,
    mut lifetime_event: EventReader<LifetimeExpired>,
    // mut active_scripts: ResMut<ActiveScripts>,
) {
    let mut drops = vec![];
    for event in lifetime_event.iter() {
        if let Some(drop) = event.drop.clone() {
            drops.push((drop, event.transform.expect("Needs transform to drop!")));
        }
    }
    for event in broke_event.iter() {
        if let Some(drop) = event.drop.clone() {
            drops.push((drop, event.transform.expect("Needs transform to drop!")));
        }
    }

    for (drop, transform) in drops {
        let ground_offset = Vec3::new(0.0, consts::GROUND_Y, consts::ITEM_LAYER);

        let item_spawn_meta = ItemSpawnMeta {
            location: transform.translation - ground_offset,
            item: String::new(),
            item_handle: items_assets.add(drop.item.clone()),
        };
        let item_commands = commands.spawn(ItemBundle::new(&item_spawn_meta));
        ItemBundle::spawn(
            item_commands,
            &item_spawn_meta,
            &mut items_assets,
            // &mut active_scripts,
        );
    }
}

/// A component that with Breakable, explodes.
#[derive(Component, Clone)]
pub struct Explodable {
    pub attack: AttackMeta,
    pub timer: Timer,
    pub fusing: bool,
    pub animated_sprite: AnimatedSpriteSheetBundle,
    pub explosion_frames: AttackFrames,
    pub attack_enemy: bool,
}

fn explodable_system(
    mut commands: Commands,
    mut broke_event: EventReader<BrokeEvent>,
    mut explodables: Query<(
        &mut Explodable,
        &mut Force,
        &mut LinearVelocity,
        &mut AngularVelocity,
        &mut Transform,
        &GlobalTransform,
        &mut Animation,
        Entity,
        Option<&Parent>,
    )>,
    time: Res<Time>,
    mut inventory: Query<&mut Inventory>,
) {
    let mut explosions = Vec::new();

    for event in broke_event.iter() {
        if let Some((transform, explodable)) = event.transform.zip(event.explodable.clone()) {
            explosions.push((transform, explodable))
        }
    }

    for (
        mut explodable,
        mut force,
        mut velocity,
        mut ang_vel,
        mut transform,
        g_transform,
        mut animation,
        entity,
        parent,
    ) in &mut explodables
    {
        explodable.timer.tick(time.delta());

        if !explodable.fusing && explodable.timer.finished() {
            // Stop bomb and start fusing
            force.0 = Vec2::ZERO;
            velocity.0 = Vec2::ZERO;
            ang_vel.0 = 0.;
            transform.rotation.z = 0.;

            animation.play("bomb_fuse", false);
            explodable.fusing = true;

            //Remove explosion on contact
            commands.entity(entity).remove::<Collider>();
        } else if animation.is_finished() && explodable.fusing {
            explosions.push((g_transform.compute_transform(), explodable.clone()));

            if let Some(parent) = parent {
                if let Ok(mut inventory) = inventory.get_mut(parent.get()) {
                    **inventory = None;
                }
            }

            commands.entity(entity).despawn_recursive();
        }
    }

    for (transform, explodable) in explosions {
        // Spawn explosion
        let mut animated_sprite = explodable.animated_sprite.clone();
        animated_sprite.sprite_sheet.transform = transform;
        animated_sprite.sprite_sheet.transform.rotation.z = 0.;
        animated_sprite.animation.play("explosion", false);

        let attack = explodable.attack.clone();
        let seconds = animated_sprite
            .animation
            .animations
            .get("explosion")
            .expect("No explosion animation");
        let seconds = (seconds.frames.end - seconds.frames.start) as f32
            * animated_sprite.animation.timer.duration().as_secs_f32();

        let attack_ent = commands
            .spawn((
                Sensor,
                ActiveEvents::COLLISION_EVENTS,
                ActiveCollisionTypes::default() | ActiveCollisionTypes::STATIC_STATIC,
                CollisionGroups::new(
                    if explodable.attack_enemy {
                        BodyLayers::PLAYER_ATTACK
                    } else {
                        BodyLayers::ENEMY_ATTACK
                    },
                    if explodable.attack_enemy {
                        BodyLayers::PLAYER | BodyLayers::ENEMY | BodyLayers::BREAKABLE_ITEM
                    } else {
                        BodyLayers::PLAYER
                    },
                ),
                Attack {
                    damage: attack.damage,
                    pushback: attack.velocity.unwrap_or(Vec2::ZERO),
                    hitstun_duration: attack.hitstun_duration,
                    hitbox_meta: Some(explodable.attack.hitbox),
                },
                explodable.explosion_frames,
                transform,
            ))
            .id();

        commands
            .spawn(animated_sprite)
            .insert(Lifetime(Timer::from_seconds(seconds, TimerMode::Once)))
            .insert(explodable)
            .push_children(&[attack_ent]);
    }
}

#[derive(Bundle)]
pub struct AnimatedProjectile {
    sprite_bundle: AnimatedSpriteSheetBundle,
    velocity: LinearVelocity,
    angular_velocity: AngularVelocity,
    force: Force,
    collider: Collider,
    sensor: Sensor,
    events: ActiveEvents,
    collision_types: ActiveCollisionTypes,
    collision_groups: CollisionGroups,
    attack: Attack,
    breakable: Breakable,
}

impl AnimatedProjectile {
    pub fn new(
        item_meta: &ItemMeta,
        facing: &Facing,
        animated_sprite: AnimatedSpriteSheetBundle,
    ) -> Self {
        let direction_mul = if facing.is_left() {
            Vec2::new(-1.0, 1.0)
        } else {
            Vec2::ONE
        };
        let mut rng = rand::thread_rng();

        let item_vars = match item_meta.kind {
            crate::metadata::ItemKind::Bomb {
                damage,
                gravity,
                throw_velocity,
                ..
            } => Some((damage, gravity, throw_velocity)),
            _ => None,
        }
        .expect("Non bomb");

        Self {
            sprite_bundle: animated_sprite,
            attack: Attack {
                damage: item_vars.0,
                pushback: Vec2::new(consts::ITEM_ATTACK_VELOCITY, 0.0) * direction_mul,
                hitstun_duration: consts::HITSTUN_DURATION,
                hitbox_meta: None,
            },
            velocity: LinearVelocity(item_vars.2 * direction_mul * rng.gen_range(0.8..1.2)),
            // Gravity
            force: Force(Vec2::new(0.0, -item_vars.1)),
            angular_velocity: AngularVelocity(
                consts::THROW_ITEM_ROTATION_SPEED * direction_mul.x * rng.gen_range(0.8..1.2),
            ),
            collider: Collider::cuboid(consts::ITEM_WIDTH / 2., consts::ITEM_HEIGHT / 2.),
            sensor: Sensor,
            events: ActiveEvents::COLLISION_EVENTS,
            collision_types: ActiveCollisionTypes::default() | ActiveCollisionTypes::STATIC_STATIC,
            //TODO: define collision layer based on the fighter shooting projectile, load for asset
            //files of fighter which "team" they are on
            collision_groups: CollisionGroups::new(BodyLayers::ENEMY_ATTACK, BodyLayers::PLAYER),
            breakable: Breakable::new(0, false),
        }
    }
}
