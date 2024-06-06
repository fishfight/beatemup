#![allow(clippy::type_complexity)]
#![allow(clippy::forget_non_drop)]
#![allow(clippy::too_many_arguments)]

use std::time::Duration;

use bevy::{asset::ChangeWatcher, prelude::*, window::WindowResolution};
use bevy_parallax::ParallaxPlugin;
use bevy_rapier2d::prelude::*;
use fighter::Stats;
use input::MenuAction;
// use iyes_loopless::prelude::*;
use leafwing_input_manager::prelude::*;
use player::*;

use bevy_inspector_egui::quick::WorldInspectorPlugin;
// use bevy_inspector_egui_rapier::InspectableRapierPlugin;

mod animation;
mod assets;
mod attack;
mod audio;
mod camera;
mod collision;
mod config;
mod consts;
mod damage;
mod enemy;
mod enemy_ai;
mod fighter;
mod fighter_state;
mod input;
mod item;
mod lifetime;
mod loading;
mod localization;
mod metadata;
mod movement;
mod platform;
mod player;
// mod scripting;
mod ui;
mod utils;

use animation::*;
use attack::AttackPlugin;
use audio::*;
use camera::*;
use enemy_ai::WalkTarget;
use metadata::GameMeta;
use ui::UIPlugin;
use utils::ResetController;

use crate::{
    damage::DamagePlugin, fighter::FighterPlugin, fighter_state::FighterStatePlugin,
    input::PlayerAction, item::ItemPlugin, lifetime::LifetimePlugin, loading::LoadingPlugin,
    localization::LocalizationPlugin, metadata::GameHandle, movement::MovementPlugin,
    platform::PlatformPlugin, ui::debug_tools::YSortDebugPlugin,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum GameState {
    #[default]
    LoadingStorage,
    LoadingGame,
    MainMenu,
    LoadingLevel,
    InGame,
    Paused,
    //Editor,
}

fn main() {
    // Load engine config. This will parse CLI arguments or web query string so we want to do it
    // before we create the app to make sure everything is in order.
    let engine_config = &*config::ENGINE_CONFIG;

    let mut app = App::new();

    app.add_plugins({
        let mut builder = DefaultPlugins.build();

        // Configure Window
        builder = builder
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Fish Folk Punchy".to_string(),
                    // scale_factor_override: Some(1.0),
                    resolution: WindowResolution::default(),
                    // resolution: WindowResolution::set_scale_factor_override,
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest());

        let watch_for_changes = if engine_config.hot_reload {
            ChangeWatcher::with_delay(Duration::from_millis(200))
        } else {
            None
        };
        // Configure asset server
        let mut asset_plugin = AssetPlugin {
            watch_for_changes,
            ..default()
        };
        if let Some(asset_folder) = &engine_config.asset_dir {
            asset_plugin.asset_folder.clone_from(asset_folder);
        }
        builder = builder.set(asset_plugin);

        // Configure log level
        builder = builder.set(bevy::log::LogPlugin {
            filter: engine_config.log_level.clone(),
            ..default()
        });

        #[cfg(feature = "schedule_graph")]
        {
            builder.disable::<bevy::log::LogPlugin>()
        }

        #[cfg(not(feature = "schedule_graph"))]
        builder
    });

    // Add other systems and resources
    app.insert_resource(ClearColor(Color::BLACK))
        .add_state::<GameState>()
        // .add_loopless_state(GameState::LoadingStorage)
        // .add_plugin(ScriptingPlugin)
        .add_plugins((
            PlatformPlugin,
            LocalizationPlugin,
            LoadingPlugin,
            RapierPhysicsPlugin::<NoUserData>::default(),
            InputManagerPlugin::<PlayerAction>::default(),
            InputManagerPlugin::<MenuAction>::default(),
            (
                AttackPlugin,
                AnimationPlugin,
                ParallaxPlugin,
                UIPlugin,
                FighterStatePlugin,
                MovementPlugin,
                AudioPlugin,
                DamagePlugin,
                LifetimePlugin,
                CameraPlugin,
                ItemPlugin,
                FighterPlugin,
            ),
        ))
        // .insert_resource(ParallaxResource::default())
        .add_systems(
            PostUpdate,
            (
                game_over_on_players_death.run_if(in_state(GameState::InGame)),
                main_menu_sounds
                    .run_if(resource_exists::<GameMeta>())
                    .before(bevy_egui::EguiSet::ProcessOutput),
            ),
        );
    // .add_system_set_to_stage(
    //     CoreStage::PostUpdate,
    //     ConditionSet::new()
    //         .run_in_state(GameState::InGame)
    //         .with_system(game_over_on_players_death)
    //         .into(),
    // )
    //this should be moved to AudioPlugin, it also causes a panic in egui_inspector when
    //using the color picker widget currently
    // .add_system_to_stage(
    //     CoreStage::PostUpdate,
    //     main_menu_sounds
    //         .run_if_resource_exists::<GameMeta>()
    //         .before(bevy_egui::EguiSystem::ProcessOutput),
    // );

    // Register reflect types that don't come from plugins
    app.register_type::<Stats>().register_type::<WalkTarget>();

    // Add debug plugins if enabled
    if engine_config.debug_tools {
        app.insert_resource(DebugRenderContext {
            enabled: false,
            ..default()
        })
        .add_plugins(YSortDebugPlugin)
        // .add_plugins(InspectableRapierPlugin)
        //TODO: now need to configure worldinspector to be disabled by default
        // .insert_resource(WorldInspectorParams {
        //     enabled: false,
        //     ..default()
        // })
        .add_plugins(WorldInspectorPlugin::new());
    }

    // Register assets and loaders
    assets::register(&mut app);

    debug!(?engine_config, "Starting game");

    // Get the game handle
    let asset_server = app.world.get_resource::<AssetServer>().unwrap();
    let game_asset = &engine_config.game_asset;
    let game_handle: Handle<GameMeta> = asset_server.load(game_asset);

    // Insert game handle resource
    app.world.insert_resource(GameHandle(game_handle));

    // Print the graphviz schedule graph
    #[cfg(feature = "schedule_graph")]
    bevy_mod_debugdump::print_schedule(&mut app);

    app.run();
}

/// Transition back to main menu and reset world when all players have died
fn game_over_on_players_death(
    // mut commands: Commands,
    query: Query<(), With<Player>>,
    reset_controller: ResetController,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if query.is_empty() {
        next_state.set(GameState::MainMenu);
        // commands.insert_resource(NextState(GameState::MainMenu));

        reset_controller.reset_world();
    }
}
