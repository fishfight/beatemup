// Multiple sounds can be played by one channel, but splitting music/effects is cleaner.
// Also for cleanness (named channels have evident function), we don't use the default channel.
use rand::{prelude::SliceRandom, thread_rng};

use bevy::{prelude::*, utils::HashMap, window::PrimaryWindow};
use bevy_egui::{egui::output::OutputEvent, EguiOutput};
use bevy_kira_audio::{AudioApp, AudioChannel, AudioControl, AudioSource};
// use iyes_loopless::prelude::*;

use crate::{
    animation::Animation,
    config::ENGINE_CONFIG,
    metadata::{GameMeta, LevelHandle, LevelMeta},
    GameState,
};

/// For readability.
const IMPOSSIBLE_ANIMATION_I: usize = usize::MAX;

#[derive(Resource)]
pub struct MusicChannel;

#[derive(Resource)]
pub struct EffectsChannel;

pub fn set_audio_channels_volume(
    music_channel: Res<AudioChannel<MusicChannel>>,
    effects_channel: Res<AudioChannel<EffectsChannel>>,
) {
    music_channel.set_volume(0.5);
    effects_channel.set_volume(0.5);
}

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_kira_audio::AudioPlugin)
            .add_audio_channel::<MusicChannel>()
            .add_audio_channel::<EffectsChannel>()
            .add_systems(Startup, set_audio_channels_volume)
            .add_systems(OnEnter(GameState::InGame), play_level_music)
            .add_systems(OnExit(GameState::InGame), stop_level_music)
            .add_systems(
                PostUpdate,
                animation_audio_playback.run_if(in_state(GameState::InGame)),
            );
    }
}

/// Add this to a fighter, when want to play sound effects attached to certain animation indexes.
#[derive(Component)]
pub struct AnimationAudioPlayback {
    pub animation_name: String,
    pub effects: HashMap<usize, Handle<AudioSource>>,
    pub last_played: Option<usize>,
}

impl AnimationAudioPlayback {
    pub fn new(animation_name: String, effects: HashMap<usize, Handle<AudioSource>>) -> Self {
        Self {
            animation_name,
            effects,
            last_played: None,
        }
    }
}

pub fn animation_audio_playback(
    mut commands: Commands,
    mut query: Query<(Entity, &Animation, &mut AnimationAudioPlayback)>,
    effects_channel: Res<AudioChannel<EffectsChannel>>,
) {
    for (entity, animation, mut state_effects) in query.iter_mut() {
        // The safest way to remove the sound component is on the next state, because the component
        // can be remove only at the last frame of animation, which in theory, may be skipped if
        // there is an unexpected lag.
        // Alternatively, we could just not care, since subsequent states+effects will overwrite
        // the component.
        if animation.current_animation.as_ref() != Some(&state_effects.animation_name) {
            commands.entity(entity).remove::<AnimationAudioPlayback>();

            continue;
        }

        if let Some(fighter_animation_i) = animation.get_current_index() {
            if let Some(audio_handle) = state_effects.effects.get(&fighter_animation_i) {
                if state_effects.last_played.unwrap_or(IMPOSSIBLE_ANIMATION_I)
                    != fighter_animation_i
                {
                    effects_channel.play(audio_handle.clone());
                    state_effects.last_played = Some(fighter_animation_i);
                }
            }
        }
    }
}

/// Plays main menu sounds
pub fn main_menu_sounds(
    game: Res<GameMeta>,
    // mut context: Query<&EguiContext, With<PrimaryWindow>>,
    mut egui_output: Query<&EguiOutput, With<PrimaryWindow>>,
    effects_channel: Res<AudioChannel<EffectsChannel>>,
) {
    if let Ok(output) = egui_output.get_single_mut() {
        for event in output.platform_output.events.as_slice() {
            // for event in &context.get_single_mut().unwrap().get_mut().output().events {
            if let OutputEvent::Clicked(info) = event {
                // if let Ok(info.label.as_ref()) {}
                if let Some(label_ref) = info.label.as_ref() {
                    if label_ref == "Start Game" {
                        // == "Start Game" {
                        //Play down_play_button
                        effects_channel.play(game.main_menu.play_button_sound_handle.clone_weak());
                    } else {
                        //Play one of the down button audios, except down_play_button
                        effects_channel.play(
                            game.main_menu
                                .button_sound_handles
                                .choose(&mut thread_rng())
                                .expect("No button sounds")
                                .clone_weak(),
                        );
                    }
                }
            }
        }
    }
}

pub fn play_menu_music(game_meta: Res<GameMeta>, music_channel: Res<AudioChannel<MusicChannel>>) {
    // This is a workaround for a Bevy Kira bug where stopping a sound immediately after
    // playing it doesn't work. We run into this issue when the menu starts and immediately
    // stops because the auto-start flag skips the menu. See issue #121 for context.
    if !ENGINE_CONFIG.auto_start {
        music_channel.play(game_meta.main_menu.music_handle.clone());
    }
}

pub fn stop_menu_music(music_channel: Res<AudioChannel<MusicChannel>>) {
    music_channel.stop();
}

pub fn play_level_music(
    level_handle: Res<LevelHandle>,
    assets: Res<Assets<LevelMeta>>,
    music_channel: Res<AudioChannel<MusicChannel>>,
) {
    if let Some(level) = assets.get(&level_handle) {
        music_channel.play(level.music_handle.clone());
    }
}

pub fn stop_level_music(music_channel: Res<AudioChannel<MusicChannel>>) {
    music_channel.stop();
}
