use bevy::prelude::*;

pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DamageEvent>().register_type::<Health>();
    }
}

/// A component indicating how much health something has, or in other words, how much damage
/// something can take before being destroyed.
#[derive(Reflect, Component, Deref, DerefMut)]
pub struct Health(pub i32);

/// A component that indicates whether an entity can be damaged.
///
/// In other words, something that has [`Health`] but isn't [`Damageable`] is currently invincible.
#[derive(Component, Deref, DerefMut)]
#[component(storage = "SparseSet")]
pub struct Damageable(pub bool);

impl Default for Damageable {
    fn default() -> Self {
        Self(true)
    }
}

/// Event emitted when an entity is damaged
#[derive(Event)]
pub struct DamageEvent {
    pub damage_velocity: Vec2,
    pub damageing_entity: Entity,
    pub damaged_entity: Entity,
    pub damage: i32,
    pub hitstun_duration: f32,
}
