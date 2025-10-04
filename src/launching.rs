use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::physics::calc_gravity::Attractee;
use crate::physics::directional_forces::{GravityForce, Mass};
use crate::physics::velocity::Velocity;
use crate::sun_system::SolarSystemAssets;

struct LaunchingPlugin;

#[derive(Component)]
struct LaunchPad;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update,
                    (spawn_new_satellite.run_if(input_just_pressed(KeyCode::Backspace)),
                     start_new_launch.run_if(input_just_pressed(MouseButton::Left))
                    )

    );
    app.add_systems(Startup, init_launching_system);
}

fn init_launching_system(mut commands: Commands) {
    commands.spawn((
        Name::new("LaunchPad"),
        Transform::from_translation(Vec3::new(150.0, 0.0, 0.0)).with_scale(Vec3::splat(0.1)),
        LaunchPad

    ));
}

fn spawn_new_satellite(mut commands: Commands, solar_system_assets: Res<SolarSystemAssets>) {
    info!("Spawning new satellite");
    commands.spawn((
        Attractee,
        GravityForce::default(),
        Velocity(Vec2::new(0.0, 0.1)),
        Mass(1.0),
        Transform::from_translation(Vec3::new(50.0, 0.0, 0.0)).with_scale(Vec3::splat(0.025)),
        Sprite::from(solar_system_assets.collector.clone())
    ));
}

fn start_new_launch (
    mut commands: Commands,
    launch_pad_query: Query<&Transform, With<LaunchPad>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    solar_system_assets: Res<SolarSystemAssets>
) {

    if !mouse_button_input.pressed(MouseButton::Left) {
        return;
    }

    let launch_pad_transform = launch_pad_query.single().unwrap();
    let launch_position = launch_pad_transform.translation;


    let (camera, camera_transform) = camera_query.single().unwrap();

    let launch_direction = if let Some(cursor_pos) = window.cursor_position() {
        // Convert screen coordinates to world coordinates
        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            (world_pos.extend(0.0) - launch_position).normalize()
        } else {
            return;
        }
    } else {
        return;
    };

    info!("Launching new satellite towards {:?}", launch_direction);

    commands.spawn((
        Attractee,
        GravityForce::default(),
        Velocity(launch_direction.xy()),
        Mass(1.0),
        Transform::from_translation(launch_position + launch_direction).with_scale(Vec3::splat(0.025)),
        Sprite::from(solar_system_assets.collector.clone())
    ));






}