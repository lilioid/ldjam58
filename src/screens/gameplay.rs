//! The screen state for the main gameplay.

use bevy::{prelude::*};
use crate::sun_system::init_sun_system;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(super::Screen::Gameplay), setup_scene);
    app.add_systems(OnEnter(super::Screen::Gameplay), init_sun_system);
}

fn setup_scene(mut commands: Commands) {
    commands.spawn(Camera2d);
}