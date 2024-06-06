use bevy::prelude::*;

use crate::item::Drop;

pub struct LifetimePlugin;

impl Plugin for LifetimePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Last, lifetime_system)
            .add_event::<LifetimeExpired>();
    }
}

/// Component added to entities that should despawn after a timer.
#[derive(Component, Deref, DerefMut, Debug, Clone)]
pub struct Lifetime(pub Timer);

/// Despawn entities who's lifetime has expired
fn lifetime_system(
    mut commands: Commands,
    mut entities: Query<(Entity, &mut Lifetime, Option<&Drop>, Option<&Transform>)>,
    time: Res<Time>,
    mut event_writer: EventWriter<LifetimeExpired>,
) {
    for (entity, mut lifetime, drop, transform) in &mut entities {
        lifetime.tick(time.delta());

        if lifetime.finished() {
            event_writer.send(LifetimeExpired {
                drop: drop.cloned(),
                transform: transform.cloned(),
            });
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[derive(Event)]
pub struct LifetimeExpired {
    pub drop: Option<Drop>,
    pub transform: Option<Transform>,
}
