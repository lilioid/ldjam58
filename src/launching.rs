use std::fmt::Debug;
use bevy::color::palettes::basic::GREEN;
use bevy::color::palettes::css::WHITE;
use crate::GameplaySystem;
use crate::collision::HitBox;
use crate::physics::calc_gravity::Attractee;
use crate::physics::directional_forces::{GravityForce, Mass};
use crate::physics::velocity::Velocity;
use crate::score::{EnergyRateLabel, Score};
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
    pub total_collected: f32
}

#[derive(Component)]
pub struct Fuel {
    pub amount: f32,
}

#[derive(Component)]
pub struct FuelLabel;


pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            start_new_launch.run_if(input_just_released(MouseButton::Left)),
            record_launch_time.run_if(input_just_pressed(MouseButton::Left)),
            deactivate_old_sats.run_if(input_just_released(MouseButton::Left)),
            update_fuel_label,
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
    let sprite ;
    let lvl ;
    if (score.energy_stored > 2. && score.energy_stored <5.){
        lvl=2.;
        sprite = solar_system_assets.collector2.clone();
    }else if (score.energy_stored >5.){
        lvl=3.;
        sprite = solar_system_assets.collector3.clone();
    }else{
        lvl=1.;
        sprite= solar_system_assets.collector.clone()
    }

let collector_id = commands.spawn((
        Fuel { amount: 1.5 },
        Level { level: lvl },
        Attractee,
        GravityForce::default(),
        Velocity(launch_direction.xy() * Vec2::splat(force_multiplier as f32)),
        Mass(1.0),
        Transform::from_translation(launch_position + launch_direction)
            .with_scale(Vec3::splat(0.015)),
        Sprite::from(sprite),
        TextColor(Color::from(GREEN)),
        Thruster::new(ThrusterDirection::Retrograde, 2.0),
        HitBox { radius: 5.0 },
        NavigationInstruments,
        Satellite,
        CollectorStats {
            energy_rate: 0.0,
            total_collected: 0.0,
        },
        Pickable::default(),
    ))
        .observe(on_hover_collector_over)
        .id();

    commands.spawn((
        Text2d::new("0"),
        Transform::default().with_translation(Vec3::new(0.0, -600.0, 0.0)).with_scale(Vec3::splat(10.0)),
        TextFont {
            font_size: 32.0,
            ..default()
        },
        TextColor(Color::from(GREEN)),
        ChildOf(collector_id),
        EnergyRateLabel,
    ));

    commands.spawn((
        Text2d::new("0"),
        Transform::default().with_translation(Vec3::new(0.0, -1000.0, 0.0)).with_scale(Vec3::splat(10.0)),
        TextFont {
            font_size: 32.0,
            ..default()
        },
        TextColor(Color::from(WHITE)),
        ChildOf(collector_id),
        FuelLabel,
        Visibility::Visible,
    ));

    launch_state.launched_at_time = None;
}

fn on_hover_collector_over(
    ev: On<Pointer<Over>>,
    mut commands: Commands,
    query: Query<Entity, (With<NavigationInstruments>, With<Thruster>)>,
) {

    commands.entity(ev.entity).insert(NavigationInstruments);
    commands.entity(ev.entity).insert(Thruster::new(ThrusterDirection::Retrograde, 2.0));

    //remove it from all other satellites
    for entity in query.iter() {
        if entity != ev.entity {
            commands.entity(entity).remove::<NavigationInstruments>();
            commands.entity(entity).remove::<Thruster>();
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

fn update_fuel_label(
    collector_query: Query<(&Fuel, &Children), With<CollectorStats>>,
    mut label_query: Query<(&mut Text2d, &mut Visibility), With<FuelLabel>>,
) {
    for (fuel, children) in collector_query.iter() {
        for child in children.iter() {
            if let Ok((mut text, mut visibility)) = label_query.get_mut(child) {
                if fuel.amount <= 0.0 {
                    *visibility = Visibility::Hidden;
                } else {
                    *visibility = Visibility::Inherited;
                    **text = format!("{:.1}", fuel.amount);
                }
            }
        }
    }
}




