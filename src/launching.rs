use std::fmt::Debug;
use bevy::color::palettes::basic::GREEN;
use crate::GameplaySystem;
use crate::collision::HitBox;
use crate::physics::calc_gravity::Attractee;
use crate::physics::directional_forces::{GravityForce, Mass};
use crate::physics::velocity::Velocity;
use crate::score::Score;
use crate::sun_system::navigation_instruments::NavigationInstruments;
use crate::sun_system::thruster::{Thruster, ThrusterDirection};
use crate::sun_system::{Level, Satellite, SolarSystemAssets};
use bevy::input::common_conditions::{input_just_pressed, input_just_released};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;


#[derive(Component)]
pub struct LaunchPad;



#[derive(Resource)]
pub struct LaunchState {
    pub launched_at_time: Option<f64>,
}

#[derive(Component)]
pub struct CollectorStats {
    pub energy_rate: f32,
    pub total_collected: f32,
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            start_new_launch.run_if(input_just_released(MouseButton::Left)),
            record_launch_time.run_if(input_just_pressed(MouseButton::Left)),
            deactivate_old_sats.run_if(input_just_released(MouseButton::Left)),
        )
            .in_set(GameplaySystem),
    );
    app.insert_resource(LaunchState {
        launched_at_time: None,
    });
}

pub fn make_launchpad() -> impl Bundle {
    (
        Name::new("LaunchPad"),
        Transform::default(),
        LaunchPad,
    )
}

fn start_new_launch(
    mut commands: Commands,
    launch_pad_query: Query<&Transform, With<LaunchPad>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    solar_system_assets: Res<SolarSystemAssets>,
    mut launch_state: ResMut<LaunchState>,
    time: Res<Time>,
    mut score: ResMut<Score>,
) {
    info!("Pay energy");
    if (score.energy_stored >= 0.2) {
        score.energy_stored -= 0.2f32;
    } else {
        return;
    }
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
    let mut force_multiplier = if let Some(launch_start_time) = launch_state.launched_at_time {
        let held_duration = time.elapsed_secs_f64() - launch_start_time;
        held_duration.min(1.0) //cap at 1 secs
    } else {
        0.1
    };

    force_multiplier = force_multiplier * 10.0;

    commands.spawn((
        Name::new("Collector"),
        Level { level: 1.0 },
        Attractee,
        GravityForce::default(),
        Velocity(launch_direction.xy() * Vec2::splat(force_multiplier as f32)),
        Mass(1.0),
        Transform::from_translation(launch_position + launch_direction)
            .with_scale(Vec3::splat(0.015)),
        Sprite::from(solar_system_assets.collector.clone()),
        Thruster::new(ThrusterDirection::Retrograde, 2.0),
        HitBox { radius: 5.0 },
        NavigationInstruments,
        Satellite,
        CollectorStats {
            energy_rate: 0.0,
            total_collected: 0.0,
        },
        children![(
            Transform::from_translation(Vec3::new(0.0, 0.0, 1.0))
            .with_scale(Vec3::splat(100.0)),
        Text::from("0.0 GW"),
        TextColor(Color::from(GREEN)),
        Name::new("EnergyRateText"),
        )],
        Pickable::default(),

    ))
        .observe(on_hover_collector_over)

    ;

    launch_state.launched_at_time = None;
}

fn on_hover_collector_over(
    ev: On<Pointer<Over>>,
    mut commands: Commands,
    query: Query<Entity, With<NavigationInstruments>>,
) {
    //add the navigation instruments to the satellite
    commands.entity(ev.entity).insert(NavigationInstruments);

    //remove it from all other satellites
    for entity in query.iter() {
        if entity != ev.entity {
            commands.entity(entity).remove::<NavigationInstruments>();
        }
    }
}


fn record_launch_time(time: Res<Time>, mut launch_state: ResMut<LaunchState>, score: Res<Score>) {
    if (score.energy_stored < 0.2) {
        return;
    }
    if launch_state.launched_at_time.is_none() {
        launch_state.launched_at_time = Some(time.elapsed_secs_f64());
    }
}

fn deactivate_old_sats(
    mut commands: Commands,
    thruster_query: Query<Entity, (With<Thruster>, With<NavigationInstruments>)>,
) {
    for entity in thruster_query.iter() {
        let mut ec = commands.get_entity(entity).unwrap();
        ec.remove::<Thruster>();
        ec.remove::<NavigationInstruments>();
    }
}


