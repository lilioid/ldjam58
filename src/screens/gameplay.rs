//! The screen state for the main gameplay.

use crate::sun_system::init_sun_system;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(super::Screen::Gameplay), setup_scene);
    app.add_systems(OnEnter(super::Screen::Gameplay), init_sun_system);
}

fn setup_scene(mut commands: Commands) {
    commands.spawn((
        Name::new("Camera"),
        Camera2d,
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)).with_scale(Vec3::splat(0.25)),
        ));

    commands.spawn((
        Name::new("Sun"),
    ));
}
