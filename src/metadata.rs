use bevy::{
    math::{UVec2, Vec2, Vec3},
    prelude::{Color, Component, Deref, DerefMut, Handle, Image, Resource},
    reflect::{Reflect, TypePath, TypeUuid},
    sprite::TextureAtlas,
    utils::{default, HashMap},
};
use bevy_egui::egui;
use bevy_kira_audio::AudioSource;
use bevy_parallax::{LayerData, LayerSpeed};
use punchy_macros::HasLoadProgress;
use serde::Deserialize;

use crate::{animation::Clip, assets::EguiFont, attack::AttackFrames, fighter::Stats};

pub mod settings;
pub use settings::*;

pub use ui::*;
pub mod ui;

pub mod localization;
pub use localization::TranslationsMeta;

#[derive(Resource, Deref, DerefMut)]
pub struct GameHandle(pub Handle<GameMeta>);

#[derive(Resource, HasLoadProgress, TypeUuid, Deserialize, Clone, Debug, TypePath)]
#[serde(deny_unknown_fields)]
#[uuid = "eb28180f-ef68-44a0-8479-a299a3cef66e"]
pub struct GameMeta {
    pub start_level: String,
    #[serde(skip)]
    pub start_level_handle: Handle<LevelMeta>,
    pub main_menu: MainMenuMeta,
    pub ui_theme: UIThemeMeta,
    pub camera_height: u32,
    pub camera_move_right_boundary: f32,

    pub default_settings: Settings,
    pub translations: TranslationsMeta,
}

#[derive(HasLoadProgress, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MainMenuMeta {
    pub title_font: FontMeta,
    pub background_image: ImageMeta,
    pub music: String,
    #[serde(skip)]
    pub music_handle: Handle<AudioSource>,
    pub play_button_sound: String,
    #[serde(skip)]
    pub play_button_sound_handle: Handle<AudioSource>,
    pub button_sounds: Vec<String>,
    #[serde(skip)]
    pub button_sound_handles: Vec<Handle<AudioSource>>,
}

#[derive(HasLoadProgress, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ImageMeta {
    pub image: String,
    pub image_size: Vec2,
    #[serde(skip)]
    pub image_handle: Handle<Image>,
}

#[derive(Resource, Deref, DerefMut)]
pub struct LevelHandle(pub Handle<LevelMeta>);

#[derive(Resource, HasLoadProgress, TypeUuid, Deserialize, Clone, Debug, TypePath)]
#[serde(deny_unknown_fields)]
#[uuid = "32111f6e-bb9a-4ea7-8988-1220b923a059"]
pub struct LevelMeta {
    #[has_load_progress(none)]
    pub background_color: [u8; 3],
    pub parallax_background: ParallaxMeta,
    pub players: Vec<FighterSpawnMeta>,
    #[serde(default)]
    pub enemies: Vec<FighterSpawnMeta>,
    #[serde(default)]
    pub items: Vec<ItemSpawnMeta>,
    pub music: String,
    #[serde(skip)]
    pub music_handle: Handle<AudioSource>,
    pub stop_points: Vec<f32>,
}

impl LevelMeta {
    pub fn background_color(&self) -> Color {
        let [r, g, b] = self.background_color;
        Color::rgb_u8(r, g, b)
    }
}

#[derive(TypeUuid, Deserialize, Clone, Debug, Component, TypePath)]
#[serde(deny_unknown_fields)]
#[uuid = "d5e040c4-3de7-4b8a-b6c2-27f82f58d8f0"]
pub struct FighterMeta {
    pub name: String,
    #[serde(skip)]
    pub center_y: f32,
    #[serde(skip)]
    pub collision_offset: f32,
    pub stats: Stats,
    pub hud: FighterHudMeta,
    pub spritesheet: FighterSpritesheetMeta,
    pub audio: AudioMeta,
    pub hurtbox: ColliderMeta,
    pub attacks: Vec<AttackMeta>,
    pub attachment: Option<FighterSpritesheetMeta>,
}

#[derive(TypeUuid, Deserialize, Clone, Debug, Component, Reflect)]
#[serde(deny_unknown_fields)]
#[uuid = "45a912f4-ea5c-4eba-9ba9-f1a726140f28"]
pub struct AttackMeta {
    pub name: String,
    pub damage: i32,
    pub frames: AttackFrames,
    pub hitbox: ColliderMeta,
    pub hitstun_duration: f32,
    pub velocity: Option<Vec2>,
    pub item: Option<String>,
    #[serde(skip)]
    pub item_handle: Handle<ItemMeta>,
}

#[derive(TypeUuid, Deserialize, Clone, Debug, Component, TypePath)]
#[serde(deny_unknown_fields)]
#[uuid = "5e2db270-ec2e-013a-92a8-2cf05d71216b"]
pub struct ItemMeta {
    pub name: String,
    pub image: ImageMeta,
    pub kind: ItemKind,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub enum ItemKind {
    Throwable {
        damage: i32,
        gravity: f32,
        throw_velocity: Vec2,
        lifetime: f32,
        pushback: f32,
        hitstun_duration: f32,
    },
    BreakableBox {
        damage: i32,
        gravity: f32,
        throw_velocity: Vec2,
        lifetime: f32,
        pushback: f32,
        hitstun_duration: f32,
        hurtbox: ColliderMeta,
        hits: i32,
        item: String,
        #[serde(skip)]
        item_handle: Handle<ItemMeta>,
    },
    MeleeWeapon {
        attack: AttackMeta,
        audio: AudioMeta,
        spritesheet: Box<FighterSpritesheetMeta>,
        sprite_offset: Vec2,
    },
    ProjectileWeapon {
        attack: AttackMeta,
        audio: AudioMeta,
        spritesheet: Box<FighterSpritesheetMeta>,
        sprite_offset: Vec2,
        bullet_velocity: f32,
        bullet_lifetime: f32,
        ammo: usize,
        shoot_delay: f32,
    },
    // Script {
    //     /// The relative asset path to the script for this item
    //     script: String,
    //     #[serde(skip)]
    //     script_handle: Handle<JsScript>,
    // },
    Bomb {
        spritesheet: FighterSpritesheetMeta,
        attack_frames: AttackFrames,
        damage: i32,
        gravity: f32,
        throw_velocity: Vec2,
        lifetime: f32,
    },
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct FighterHudMeta {
    pub portrait: ImageMeta,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct FighterSpritesheetMeta {
    pub image: Vec<String>,
    #[serde(skip)]
    pub atlas_handle: Vec<Handle<TextureAtlas>>,
    pub tile_size: UVec2,
    pub columns: usize,
    pub rows: usize,
    pub animation_fps: f32,
    pub animations: HashMap<String, Clip>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct AudioMeta {
    pub effects: HashMap<String, HashMap<usize, String>>,
    #[serde(skip)]
    pub effect_handles: HashMap<String, HashMap<usize, Handle<AudioSource>>>,
}

#[derive(HasLoadProgress, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct FighterSpawnMeta {
    pub fighter: String,
    #[serde(skip)]
    pub fighter_handle: Handle<FighterMeta>,
    pub location: Vec3,
    // Set only for enemies.
    #[serde(default = "default_f32_min")]
    pub trip_point_x: f32,
    #[serde(default)]
    pub boss: bool,
}

fn default_f32_min() -> f32 {
    f32::MIN
}

#[derive(HasLoadProgress, TypeUuid, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
#[uuid = "f5092550-ec30-013a-92a9-2cf05d71216b"]
pub struct ItemSpawnMeta {
    pub item: String,
    #[serde(skip)]
    pub item_handle: Handle<ItemMeta>,
    pub location: Vec3,
}

#[derive(HasLoadProgress, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ParallaxMeta {
    pub layers: Vec<ParallaxLayerMeta>,
}

impl ParallaxMeta {
    pub fn get_layer_data(&self) -> Vec<LayerData> {
        self.layers.iter().cloned().map(Into::into).collect()
    }
}

#[derive(HasLoadProgress, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ParallaxLayerMeta {
    pub speed: f32,
    pub path: String,
    #[serde(skip)]
    pub image_handle: Handle<Image>,
    pub tile_size: Vec2,
    pub cols: usize,
    pub rows: usize,
    pub scale: f32,
    pub z: f32,
    #[serde(default)]
    pub position: Vec2,
}

impl From<ParallaxLayerMeta> for LayerData {
    fn from(meta: ParallaxLayerMeta) -> Self {
        Self {
            speed: LayerSpeed::Horizontal(meta.speed),
            path: meta.path,
            tile_size: meta.tile_size,
            cols: meta.cols,
            rows: meta.rows,
            scale: meta.scale,
            z: meta.z,
            position: meta.position,
            ..default()
        }
    }
}

#[derive(HasLoadProgress, Deserialize, Default, Copy, Clone, Debug, Reflect)]
#[serde(deny_unknown_fields)]
pub struct ColliderMeta {
    //TODO: Add type of collider with different properties.
    pub size: Vec2,
    pub offset: Vec2,
}
