//! The screen state for the main gameplay.

use bevy::{prelude::*};
use bevy::color::palettes::basic::YELLOW;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(super::Screen::Gameplay), setup_scene);
}

fn setup_scene(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn(Camera2d);

    commands.spawn((
        Name::new("Sun"),
        Mesh2d(meshes.add(Circle::default())),
        MeshMaterial2d(materials.add(Color::from(YELLOW))),
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)).with_scale(Vec3::splat(128.0)),
        ));
}