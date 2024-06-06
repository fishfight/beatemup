use bevy::{
    prelude::*,
    utils::HashMap,
    window::{PrimaryWindow, WindowMode},
};
use bevy_egui::{egui, EguiContexts, EguiInput, EguiPlugin, EguiSettings};
use leafwing_input_manager::prelude::ActionState;

use crate::{
    assets::{EguiFont, EguiFontDefinitions},
    audio,
    config::ENGINE_CONFIG,
    input::MenuAction,
    metadata::GameMeta,
    GameState,
};

pub mod hud;
pub mod widgets;

pub mod debug_tools;
pub mod main_menu;
pub mod pause_menu;

pub mod extensions;
pub use extensions::*;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WidgetAdjacencies>()
            .add_plugins(EguiPlugin)
            // .add_system(handle_menu_input.run_if_resource_exists::<GameMeta>())
            .add_systems(
                OnEnter(GameState::MainMenu),
                (
                    main_menu::spawn_main_menu_background,
                    audio::play_menu_music,
                ),
            )
            // .add_enter_system(GameState::MainMenu, main_menu::spawn_main_menu_background)
            // .add_enter_system(GameState::MainMenu, audio::play_menu_music)
            .add_systems(
                OnExit(GameState::MainMenu),
                (
                    main_menu::despawn_main_menu_background,
                    audio::stop_menu_music,
                ),
            )
            // .add_exit_system(GameState::MainMenu, main_menu::despawn_main_menu_background)
            // .add_exit_system(GameState::MainMenu, audio::stop_menu_music)
            .add_systems(Update, unpause.run_if(in_state(GameState::Paused)))
            .add_systems(
                Update,
                (hud::render_hud, pause).run_if(in_state(GameState::InGame)),
            )
            // .add_system_set(
            //     ConditionSet::new()
            //         .run_in_state(GameState::InGame)
            //         .with_system(hud::render_hud)
            //         .with_system(pause)
            //         .into(),
            // )
            .add_systems(Update, update_egui_fonts)
            .add_systems(
                Update,
                (update_ui_scale, handle_menu_input).run_if(resource_exists::<GameMeta>()),
            )
            .add_systems(
                Update,
                pause_menu::pause_menu.run_if(in_state(GameState::Paused)),
            )
            // .add_system_set(
            //     ConditionSet::new()
            //         .run_in_state(GameState::Paused)
            //         .with_system(pause_menu::pause_menu)
            //         .into(),
            // )
            .add_systems(
                Update,
                main_menu::main_menu_system.run_if(in_state(GameState::MainMenu)),
            );
        // .add_system_set(
        //     ConditionSet::new()
        //         .run_in_state(GameState::MainMenu)
        //         .with_system(main_menu::main_menu_system)
        //         .into(),
        // );

        if ENGINE_CONFIG.debug_tools {
            // app.add_systems(Update, debug_tools::debug_tools_window)
            // add_systems(Last, debug_tools::rapier_debug_render);
        }
    }
}

/// Transition game to pause state
fn pause(input: Query<&ActionState<MenuAction>>, mut next_state: ResMut<NextState<GameState>>) {
    let input = input.single();
    if input.just_pressed(MenuAction::Pause) {
        next_state.set(GameState::Paused);
    }
}

// Transition game out of paused state
fn unpause(input: Query<&ActionState<MenuAction>>, mut next_state: ResMut<NextState<GameState>>) {
    let input = input.single();
    if input.just_pressed(MenuAction::Pause) {
        next_state.set(GameState::Paused);
    }
}

/// Resource that stores which ui widgets are adjacent to which other widgets.
///
/// This is used to figure out which widget to focus on next when you press a direction on the
/// gamepad, for instance.
#[derive(Debug, Clone, Default, Resource)]
pub struct WidgetAdjacencies(HashMap<egui::Id, WidgetAdjacency>);

impl std::ops::Deref for WidgetAdjacencies {
    type Target = HashMap<egui::Id, WidgetAdjacency>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for WidgetAdjacencies {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// The list of widgets in each direction from another widget
#[derive(Debug, Clone, Default)]
pub struct WidgetAdjacency {
    pub up: Option<egui::Id>,
    pub down: Option<egui::Id>,
    pub left: Option<egui::Id>,
    pub right: Option<egui::Id>,
}

impl WidgetAdjacencies {
    pub fn widget(&mut self, resp: &egui::Response) -> WidgetAdjacencyEntry {
        WidgetAdjacencyEntry {
            id: resp.id,
            adjacencies: self,
        }
    }
}

pub struct WidgetAdjacencyEntry<'a> {
    id: egui::Id,
    adjacencies: &'a mut WidgetAdjacencies,
}

#[allow(clippy::wrong_self_convention)]
impl<'a> WidgetAdjacencyEntry<'a> {
    pub fn to_left_of(self, resp: &egui::Response) -> Self {
        let other_id = resp.id;
        self.adjacencies.0.entry(self.id).or_default().right = Some(other_id);
        self.adjacencies.0.entry(other_id).or_default().left = Some(self.id);
        self
    }
    pub fn to_right_of(self, resp: &egui::Response) -> Self {
        let other_id = resp.id;
        self.adjacencies.0.entry(self.id).or_default().left = Some(other_id);
        self.adjacencies.0.entry(other_id).or_default().right = Some(self.id);
        self
    }
    pub fn above(self, resp: &egui::Response) -> Self {
        let other_id = resp.id;
        self.adjacencies.0.entry(self.id).or_default().down = Some(other_id);
        self.adjacencies.0.entry(other_id).or_default().up = Some(self.id);
        self
    }
    pub fn below(self, resp: &egui::Response) -> Self {
        let other_id = resp.id;
        self.adjacencies.0.entry(self.id).or_default().up = Some(other_id);
        self.adjacencies.0.entry(other_id).or_default().down = Some(self.id);
        self
    }
}

fn handle_menu_input(
    // mut windows: ResMut<Windows>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    input: Query<&ActionState<MenuAction>>,
    mut egui_input: Query<&mut EguiInput, With<PrimaryWindow>>,
    // mut egui_inputs: ResMut<EguiRenderInputContainer>,
    adjacencies: Res<WidgetAdjacencies>,
    // mut egui_ctx: Query<&EguiContext, With<PrimaryWindow>>,
    mut egui_contexts: EguiContexts,
) {
    let input = input.single();

    // Handle fullscreen toggling
    if input.just_pressed(MenuAction::ToggleFullscreen) {
        // let mut window: &mut bevy::prelude::Window = windows.get_single_mut().unwrap();
        let mut window = windows.get_single_mut().unwrap();
        window.mode = match window.mode {
            WindowMode::BorderlessFullscreen => WindowMode::Windowed,
            _ => WindowMode::BorderlessFullscreen,
        };
    }

    // let events = &mut egui_inputs.get_mut(&WindowId::primary()).unwrap().0.events;
    let events = &mut egui_input.get_single_mut().unwrap().0.events;

    if input.just_pressed(MenuAction::Confirm) {
        events.push(egui::Event::Key {
            key: egui::Key::Enter,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
            repeat: false,
        });
    }

    // Helper to fall back on using tab order instead of adjacency map to determine next focused
    // widget.
    let mut tab_fallback = || {
        if input.just_pressed(MenuAction::Up) || input.just_pressed(MenuAction::Left) {
            events.push(egui::Event::Key {
                key: egui::Key::Tab,
                pressed: true,
                modifiers: egui::Modifiers::SHIFT,
                repeat: false,
            });
        } else if input.just_pressed(MenuAction::Down) || input.just_pressed(MenuAction::Right) {
            events.push(egui::Event::Key {
                key: egui::Key::Tab,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
                repeat: false,
            });
        }
    };

    let ctx = egui_contexts.ctx_mut();
    if let Some(adjacency) = ctx.memory(|memory| memory.focus().and_then(|id| adjacencies.get(&id)))
    {
        // let mut memory = egui_ctx.ctx_mut().memory();
        // if let Some(adjacency) = memory.focus().and_then(|id| adjacencies.get(&id)) {
        if input.just_pressed(MenuAction::Up) {
            if let Some(adjacent) = adjacency.up {
                ctx.memory_mut(|memory| memory.request_focus(adjacent));
            } else {
                tab_fallback()
            }
        } else if input.just_pressed(MenuAction::Down) {
            if let Some(adjacent) = adjacency.down {
                ctx.memory_mut(|memory| memory.request_focus(adjacent));
            } else {
                tab_fallback()
            }
        } else if input.just_pressed(MenuAction::Left) {
            if let Some(adjacent) = adjacency.left {
                ctx.memory_mut(|memory| memory.request_focus(adjacent));
            } else {
                tab_fallback()
            }
        } else if input.just_pressed(MenuAction::Right) {
            if let Some(adjacent) = adjacency.right {
                ctx.memory_mut(|memory| memory.request_focus(adjacent));
            } else {
                tab_fallback()
            }
        }
    } else {
        tab_fallback();
    }
}

/// Watches for asset events for [`EguiFont`] assets and updates the corresponding fonts from the
/// [`GameMeta`], inserting the font data into the egui context.
fn update_egui_fonts(
    mut font_queue: Local<Vec<Handle<EguiFont>>>,
    // mut egui_ctx: Query<&EguiContext, With<PrimaryWindow>>,
    mut egui_contexts: EguiContexts,
    egui_font_definitions: Option<ResMut<EguiFontDefinitions>>,
    game: Option<Res<GameMeta>>,
    mut events: EventReader<AssetEvent<EguiFont>>,
    assets: Res<Assets<EguiFont>>,
) {
    // Add any newly updated/created fonts to the queue
    for event in events.iter() {
        if let AssetEvent::Created { handle } | AssetEvent::Modified { handle } = event {
            font_queue.push(handle.clone_weak());
        }
    }

    // Update queued fonts if the game is ready
    if let Some((game, mut egui_font_definitions)) = game.zip(egui_font_definitions) {
        for handle in font_queue.drain(..) {
            // Get the game font name associated to this handle
            let name = game
                .ui_theme
                .font_handles
                .iter()
                .find_map(|(font_name, font_handle)| {
                    if font_handle == &handle {
                        Some(font_name.clone())
                    } else {
                        None
                    }
                });

            // If we were able to find the font handle in our game fonts
            if let Some(font_name) = name {
                // Get the font asset
                if let Some(font) = assets.get(&handle) {
                    // And insert it into the Egui font definitions
                    let ctx = egui_contexts.ctx_mut();
                    egui_font_definitions
                        .font_data
                        .insert(font_name.clone(), font.0.clone());

                    egui_font_definitions
                        .families
                        .get_mut(&egui::FontFamily::Name(font_name.clone().into()))
                        .unwrap()
                        .push(font_name);

                    ctx.set_fonts(egui_font_definitions.get_fonts().clone());
                }
            }
        }
    }
}

/// This system makes sure that the UI scale of Egui matches our game scale so that a pixel in egui
/// will be the same size as a pixel in our sprites.
fn update_ui_scale(
    mut egui_settings: ResMut<EguiSettings>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    projection: Query<&OrthographicProjection, With<Camera>>,
) {
    if let Ok(window) = window_query.get_single() {
        if let Ok(projection) = projection.get_single() {
            match projection.scaling_mode {
                bevy::render::camera::ScalingMode::FixedVertical(height) => {
                    let window_height = window.height();
                    let scale = window_height / height;
                    egui_settings.scale_factor = scale as f64;
                }
                bevy::render::camera::ScalingMode::FixedHorizontal(width) => {
                    let window_width = window.width();
                    let scale = window_width / width;
                    egui_settings.scale_factor = scale as f64;
                }
                _ => (),
            }
        }
    }
}
