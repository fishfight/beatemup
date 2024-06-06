use std::path::{Path, PathBuf};

use bevy::{
    asset::{Asset, AssetLoader, AssetPath, LoadedAsset},
    prelude::AddAsset,
    prelude::*,
    reflect::{TypePath, TypeUuid},
    utils::HashMap,
};
use bevy_egui::egui;

use crate::{consts::FOOT_PADDING, metadata::*};

/// Register game asset and loaders
pub fn register(app: &mut bevy::prelude::App) {
    app.register_type::<TextureAtlasSprite>()
        .add_asset::<GameMeta>()
        .add_asset_loader(GameMetaLoader)
        .add_asset::<LevelMeta>()
        .add_asset_loader(LevelMetaLoader)
        .add_asset::<FighterMeta>()
        .add_asset_loader(FighterLoader)
        .add_asset::<ItemMeta>()
        .add_asset_loader(ItemLoader)
        .add_asset::<EguiFont>()
        .add_asset_loader(EguiFontLoader);
}

/// Calculate an asset's full path relative to another asset
fn relative_asset_path(asset_path: &Path, relative_path: &str) -> PathBuf {
    let is_relative = !relative_path.starts_with('/');

    if is_relative {
        let base = asset_path.parent().unwrap_or_else(|| Path::new(""));
        base.join(relative_path)
    } else {
        Path::new(relative_path)
            .strip_prefix("/")
            .unwrap()
            .to_owned()
    }
}

/// Helper to get relative asset paths and handles
fn get_relative_asset<T: Asset>(
    load_context: &bevy::asset::LoadContext,
    self_path: &Path,
    relative_path: &str,
) -> (AssetPath<'static>, Handle<T>) {
    let asset_path = relative_asset_path(self_path, relative_path);
    let asset_path = AssetPath::new(asset_path, None);
    let handle = load_context.get_handle(asset_path.clone());

    (asset_path, handle)
}

#[derive(Default)]
pub struct GameMetaLoader;

impl AssetLoader for GameMetaLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let mut meta: GameMeta = serde_yaml::from_slice(bytes)?;
            trace!(?meta, "Loaded game asset");

            let self_path = load_context.path().to_owned();

            // Detect the system locale
            let locale = sys_locale::get_locale().unwrap_or_else(|| "en-US".to_string());
            let locale = locale.parse().unwrap_or_else(|e| {
                warn!(
                    "Could not parse system locale string ( \"{}\" ), defaulting to \"en-US\": {}",
                    locale, e
                );
                "en-US".parse().unwrap()
            });
            debug!("Detected system locale: {}", locale);
            meta.translations.detected_locale = locale;

            let mut dependencies = vec![];

            // Get locale handles
            for locale in &meta.translations.locales {
                let (path, handle) = get_relative_asset(load_context, &self_path, locale);
                dependencies.push(path);
                meta.translations.locale_handles.push(handle);
            }

            // Load the start level asset
            let (start_level_path, start_level_handle) =
                get_relative_asset(load_context, &self_path, &meta.start_level);
            meta.start_level_handle = start_level_handle;
            dependencies.push(start_level_path);

            // Load the main menu background
            let (main_menu_background_path, main_menu_background) = get_relative_asset(
                load_context,
                &self_path,
                &meta.main_menu.background_image.image,
            );
            meta.main_menu.background_image.image_handle = main_menu_background;
            dependencies.push(main_menu_background_path);

            // Load UI border images
            let mut load_border_image = |border: &mut BorderImageMeta| {
                let (path, handle) = get_relative_asset(load_context, &self_path, &border.image);
                dependencies.push(path);
                border.handle = handle;
            };
            load_border_image(&mut meta.ui_theme.hud.portrait_frame);
            load_border_image(&mut meta.ui_theme.panel.border);
            load_border_image(&mut meta.ui_theme.hud.lifebar.background_image);
            load_border_image(&mut meta.ui_theme.hud.lifebar.progress_image);
            for button in meta.ui_theme.button_styles.values_mut() {
                load_border_image(&mut button.borders.default);
                if let Some(border) = &mut button.borders.clicked {
                    load_border_image(border);
                }
                if let Some(border) = &mut button.borders.focused {
                    load_border_image(border);
                }
            }

            // Load the music
            let (music_path, music_handle) =
                get_relative_asset(load_context, &self_path, &meta.main_menu.music);
            meta.main_menu.music_handle = music_handle;
            dependencies.push(music_path);

            // Load button sounds
            let (play_button_sound_path, play_button_sound_handle) =
                get_relative_asset(load_context, &self_path, &meta.main_menu.play_button_sound);
            dependencies.push(play_button_sound_path);
            meta.main_menu.play_button_sound_handle = play_button_sound_handle;

            for button_sound in &meta.main_menu.button_sounds {
                let (path, handle) = get_relative_asset(load_context, &self_path, button_sound);
                dependencies.push(path);
                meta.main_menu.button_sound_handles.push(handle);
            }

            // Load UI fonts
            for (font_name, font_relative_path) in &meta.ui_theme.font_families {
                let (font_path, font_handle) =
                    get_relative_asset(load_context, &self_path, font_relative_path);

                dependencies.push(font_path);

                meta.ui_theme
                    .font_handles
                    .insert(font_name.clone(), font_handle);
            }

            // // Load the script handles
            // for script_relative_path in &meta.scripts {
            //     let (script_path, script_handle) =
            //         get_relative_asset(load_context, &self_path, script_relative_path);
            //     dependencies.push(script_path);
            //     meta.script_handles.push(script_handle);
            // }

            load_context.set_default_asset(LoadedAsset::new(meta).with_dependencies(dependencies));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["game.yml", "game.yaml"]
    }
}

pub struct LevelMetaLoader;

impl AssetLoader for LevelMetaLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let mut meta: LevelMeta = serde_yaml::from_slice(bytes)?;
            trace!(?meta, "Loaded level asset");

            let self_path = load_context.path();

            let mut dependencies = Vec::new();

            // Load the players
            for player in &mut meta.players {
                let (player_fighter_path, player_fighter_handle) =
                    get_relative_asset(load_context, self_path, &player.fighter);
                dependencies.push(player_fighter_path);

                player.fighter_handle = player_fighter_handle;
            }

            // Load the enemies
            for enemy in &mut meta.enemies {
                let (enemy_fighter_path, enemy_fighter_handle) =
                    get_relative_asset(load_context, self_path, &enemy.fighter);
                dependencies.push(enemy_fighter_path);

                enemy.fighter_handle = enemy_fighter_handle;
            }

            // Load the items
            for item in &mut meta.items {
                let (item_path, item_handle) =
                    get_relative_asset(load_context, self_path, &item.item);

                dependencies.push(item_path);

                item.item_handle = item_handle;
            }

            // Load parallax background layers
            for layer in &mut meta.parallax_background.layers {
                let (path, handle) = get_relative_asset(load_context, self_path, &layer.path);

                // Update the layer path to use an absolute path so that it matches the conventione
                // used by the bevy_parallax_background plugin.
                layer.path = path
                    .path()
                    .as_os_str()
                    .to_str()
                    .expect("utf8-filename")
                    .to_string();

                layer.image_handle = handle;
                dependencies.push(path);
            }

            // Load the music
            let (music_path, music_handle) =
                get_relative_asset(load_context, self_path, &meta.music);
            meta.music_handle = music_handle;
            dependencies.push(music_path);

            load_context.set_default_asset(LoadedAsset::new(meta).with_dependencies(dependencies));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["level.yml", "level.yaml"]
    }
}

pub struct FighterLoader;

impl AssetLoader for FighterLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let mut meta: FighterMeta = serde_yaml::from_slice(bytes)?;
            trace!(?meta, "Loaded fighter asset");

            let self_path = load_context.path();
            let mut dependencies = Vec::new();

            for attack in &mut meta.attacks {
                if let Some(item) = &attack.item {
                    let (item_path, item_handle) =
                        get_relative_asset(load_context, self_path, item);

                    dependencies.push(item_path);

                    attack.item_handle = item_handle;
                }
            }

            let (portrait_path, portrait_handle) =
                get_relative_asset(load_context, self_path, &meta.hud.portrait.image);
            dependencies.push(portrait_path);
            meta.hud.portrait.image_handle = portrait_handle;

            for (state, frame_audio_files) in &meta.audio.effects {
                for (animation_i, audio_file) in frame_audio_files {
                    let (asset_path, effect_handle) =
                        get_relative_asset(load_context, self_path, audio_file);

                    dependencies.push(asset_path);

                    let frame_audio_handles = meta
                        .audio
                        .effect_handles
                        .entry(state.clone())
                        .or_insert_with(HashMap::new);

                    frame_audio_handles.insert(*animation_i, effect_handle);
                }
            }

            for (index, image) in meta.spritesheet.image.iter().enumerate() {
                let (texture_path, texture_handle) =
                    get_relative_asset(load_context, load_context.path(), image);

                let atlas_handle = load_context.set_labeled_asset(
                    format!("atlas_{index}").as_str(),
                    LoadedAsset::new(TextureAtlas::from_grid(
                        texture_handle,
                        meta.spritesheet.tile_size.as_vec2(),
                        meta.spritesheet.columns,
                        meta.spritesheet.rows,
                        None,
                        None,
                    ))
                    .with_dependency(texture_path),
                );
                meta.spritesheet.atlas_handle.push(atlas_handle);
                meta.center_y = meta.spritesheet.tile_size.y as f32 / 2.;
                meta.collision_offset = meta.center_y - FOOT_PADDING;
            }

            if let Some(ref mut attachment) = meta.attachment {
                for (index, image) in attachment.image.iter().enumerate() {
                    let (texture_path, texture_handle) =
                        get_relative_asset(load_context, load_context.path(), image);

                    let atlas_handle = load_context.set_labeled_asset(
                        format!("atlas_{}", index + attachment.image.len()).as_str(),
                        LoadedAsset::new(TextureAtlas::from_grid(
                            texture_handle,
                            attachment.tile_size.as_vec2(),
                            attachment.columns,
                            attachment.rows,
                            None,
                            None,
                        ))
                        .with_dependency(texture_path),
                    );
                    attachment.atlas_handle.push(atlas_handle);
                }
            }

            load_context.set_default_asset(LoadedAsset::new(meta).with_dependencies(dependencies));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["fighter.yml", "fighter.yaml"]
    }
}

pub struct ItemLoader;

impl AssetLoader for ItemLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let mut meta: ItemMeta = serde_yaml::from_slice(bytes)?;
            trace!(?meta, "Loaded item asset");

            let self_path = load_context.path();
            let mut dependencies = Vec::new();

            let (image_path, image_handle) =
                get_relative_asset(load_context, self_path, &meta.image.image);
            dependencies.push(image_path);
            meta.image.image_handle = image_handle;

            match &mut meta.kind {
                ItemKind::BreakableBox {
                    ref mut item_handle,
                    ref item,
                    ..
                } => {
                    //Loads dropped item
                    let (item_path, new_item_handle) =
                        get_relative_asset(load_context, self_path, item);

                    dependencies.push(item_path);
                    *item_handle = new_item_handle;
                }

                ItemKind::MeleeWeapon {
                    ref mut spritesheet,
                    ref mut audio,
                    ..
                }
                | ItemKind::ProjectileWeapon {
                    ref mut spritesheet,
                    ref mut audio,
                    ..
                } => {
                    for (state, frame_audio_files) in &audio.effects {
                        for (animation_i, audio_file) in frame_audio_files {
                            let (asset_path, effect_handle) =
                                get_relative_asset(load_context, self_path, audio_file);

                            dependencies.push(asset_path);

                            let frame_audio_handles = audio
                                .effect_handles
                                .entry(state.clone())
                                .or_insert_with(HashMap::new);

                            frame_audio_handles.insert(*animation_i, effect_handle);
                        }
                    }

                    for (index, image) in spritesheet.image.iter().enumerate() {
                        let (texture_path, texture_handle) =
                            get_relative_asset(load_context, load_context.path(), image);

                        let atlas_handle = load_context.set_labeled_asset(
                            format!("atlas_{index}").as_str(),
                            LoadedAsset::new(TextureAtlas::from_grid(
                                texture_handle,
                                spritesheet.tile_size.as_vec2(),
                                spritesheet.columns,
                                spritesheet.rows,
                                None,
                                None,
                            ))
                            .with_dependency(texture_path),
                        );
                        spritesheet.atlas_handle.push(atlas_handle);
                    }
                }
                // ItemKind::Script {
                //     script,
                //     ref mut script_handle,
                // } => {
                //     let (script_path, loaded_script_handle) =
                //         get_relative_asset(load_context, load_context.path(), script);
                //     dependencies.push(script_path);
                //     *script_handle = loaded_script_handle;
                // }
                ItemKind::Bomb { spritesheet, .. } => {
                    for (index, image) in spritesheet.image.iter().enumerate() {
                        let (texture_path, texture_handle) =
                            get_relative_asset(load_context, load_context.path(), image);

                        let atlas_handle = load_context.set_labeled_asset(
                            format!("atlas_{index}").as_str(),
                            LoadedAsset::new(TextureAtlas::from_grid(
                                texture_handle,
                                spritesheet.tile_size.as_vec2(),
                                spritesheet.columns,
                                spritesheet.rows,
                                None,
                                None,
                            ))
                            .with_dependency(texture_path),
                        );
                        spritesheet.atlas_handle.push(atlas_handle);
                    }
                }
                _ => {}
            }

            load_context.set_default_asset(LoadedAsset::new(meta).with_dependencies(dependencies));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["item.yml", "item.yaml"]
    }
}

#[derive(Debug, Clone, TypeUuid, TypePath)]
#[uuid = "da277340-574f-4069-907c-7571b8756200"]
pub struct EguiFont(pub egui::FontData);

pub struct EguiFontLoader;

impl AssetLoader for EguiFontLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let path = load_context.path();
            let data = egui::FontData::from_owned(bytes.to_vec());
            trace!(?path, "Loaded font asset");

            load_context.set_default_asset(LoadedAsset::new(EguiFont(data)));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ttf"]
    }
}

#[derive(Debug, Clone, Resource, Deref, DerefMut)]
pub struct EguiFontDefinitions(pub egui::FontDefinitions);

impl EguiFontDefinitions {
    pub fn get_fonts(&self) -> &egui::FontDefinitions {
        &self.0
    }
}
