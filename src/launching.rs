use bevy::input::common_conditions::{input_just_pressed, input_just_released};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::physics::calc_gravity::Attractee;
use crate::physics::directional_forces::{GravityForce, Mass};
use crate::physics::velocity::Velocity;
use crate::sun_system::SolarSystemAssets;
use crate::sun_system::thruster::{Thruster, ThrusterDirection};

struct LaunchingPlugin;

#[derive(Component)]
struct LaunchPad;

#[derive(Resource)]
struct LaunchState {
    launched_at_time: Option<f64>,
}


pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, (
                     start_new_launch.run_if(input_just_released(MouseButton::Left)),
                     record_launch_time.run_if(input_just_pressed(MouseButton::Left)),
    ));
    app.add_systems(Startup, init_launching_system);
    app.insert_resource(LaunchState { launched_at_time: None });
}

fn init_launching_system(mut commands: Commands) {
    commands.spawn((
        Name::new("LaunchPad"),
        Transform::from_translation(Vec3::new(150.0, 0.0, 0.0)).with_scale(Vec3::splat(0.1)),
        LaunchPad

    ));
}

fn start_new_launch (
    mut commands: Commands,
    launch_pad_query: Query<&Transform, With<LaunchPad>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    solar_system_assets: Res<SolarSystemAssets>,
    mut launch_state: ResMut<LaunchState>,
    time: Res<Time>,
) {

    let launch_pad_transform = launch_pad_query.single().unwrap();
    let launch_position = launch_pad_transform.translation;


    let (camera, camera_transform) = camera_query.single().unwrap();

    let launch_direction = if let Some(cursor_pos) = window.cursor_position() {
        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            (world_pos.extend(0.0) - launch_position).normalize()
        } else {
            return;
        }
    } else {
        return;
    };

    info!("Launching new satellite towards {:?}", launch_direction);

    //force is dependent on how long the mouse was held down
    let force_multiplier = if let Some(launch_start_time) = launch_state.launched_at_time {
        let held_duration = time.elapsed_secs_f64() - launch_start_time;
        held_duration.min(1.0) //cap at 1 seconds
    } else {
        0.1
    };

    commands.spawn((
        Attractee,
        GravityForce::default(),
        Velocity(launch_direction.xy() * Vec2::splat(force_multiplier as f32)),
        Mass(1.0),
        Transform::from_translation(launch_position + launch_direction).with_scale(Vec3::splat(0.025)),
        Sprite::from(solar_system_assets.collector.clone()),
        Thruster::new(ThrusterDirection::Retrograde, 2.0),
    ));

    launch_state.launched_at_time = None;
}

fn record_launch_time(
    time: Res<Time>,
    mut launch_state: ResMut<LaunchState>,
) {
    if launch_state.launched_at_time.is_none() {
        launch_state.launched_at_time = Some(time.elapsed_secs_f64());
    }
}