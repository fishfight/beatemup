use bevy::prelude::*;
use leafwing_input_manager::InputManagerBundle;

use crate::{
    animation::Facing,
    consts,
    fighter::Inventory,
    input::PlayerAction,
    metadata::{FighterMeta, FighterSpawnMeta, GameMeta, Settings},
};

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerIndex(pub usize);

#[derive(Bundle)]
pub struct PlayerBundle {
    player: Player,
    index: PlayerIndex,
    facing: Facing,
    inventory: Inventory,
    transform_bundle: TransformBundle,
    fighter_handle: Handle<FighterMeta>,
    input_manager_bundle: InputManagerBundle<PlayerAction>,
}

impl PlayerBundle {
    pub fn new(
        player_meta: &FighterSpawnMeta,
        player_i: usize,
        game_meta: &GameMeta,
        settings: Option<&Settings>,
    ) -> Self {
        let ground_offset = Vec3::new(0.0, consts::GROUND_Y, 0.0);
        let player_pos = player_meta.location + ground_offset;

        let transform_bundle =
            TransformBundle::from_transform(Transform::from_translation(player_pos));

        let fighter_handle = player_meta.fighter_handle.clone();

        let input_manager_bundle = InputManagerBundle {
            input_map: settings
                .unwrap_or(&game_meta.default_settings)
                .player_controls
                .get_input_map(player_i),
            ..default()
        };

        PlayerBundle {
            player: Player,
            index: PlayerIndex(player_i),
            facing: Facing::Right,
            transform_bundle,
            fighter_handle,
            input_manager_bundle,
            inventory: Inventory(None),
        }
    }
}
