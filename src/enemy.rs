use bevy::prelude::*;

use crate::{
    animation::Facing,
    consts,
    metadata::{FighterMeta, FighterSpawnMeta},
};

#[derive(Component)]
pub struct Enemy;

/// Temporary marker component used to differentiate bosses from other fighters.
#[derive(Component)]
pub struct Boss;

/// X coordinate of the level that requires to be trespassed in order for the enemies to move.
/// For simplicy, once a given trip point is trespassed for the first time, it's set to f32::MIN.
#[derive(Component)]
pub struct TripPointX(pub f32);

#[derive(Component)]
pub struct SpawnLocationX(pub f32);

#[derive(Bundle)]
pub struct EnemyBundle {
    enemy: Enemy,
    facing: Facing,
    spawn_location_x: SpawnLocationX,
    transform_bundle: TransformBundle,
    fighter_handle: Handle<FighterMeta>,
    trip_point_x: TripPointX,
}

impl EnemyBundle {
    pub fn new(enemy_meta: &FighterSpawnMeta) -> Self {
        let ground_offset = Vec3::new(0.0, consts::GROUND_Y, 0.0);
        let enemy_pos = enemy_meta.location + ground_offset;

        let transform_bundle =
            TransformBundle::from_transform(Transform::from_translation(enemy_pos));

        let fighter_handle = enemy_meta.fighter_handle.clone();

        EnemyBundle {
            enemy: Enemy,
            facing: Facing::Left,
            spawn_location_x: SpawnLocationX(enemy_pos.x),
            transform_bundle,
            fighter_handle,
            trip_point_x: TripPointX(enemy_meta.trip_point_x),
        }
    }
}
