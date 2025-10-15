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
use crate::sun_system::{Level, Satellite, SolarSystemAssets, Sun};
use crate::sun_system::earth::Earth;
use bevy::input::common_conditions::{input_just_pressed, input_just_released};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::input::touch::{TouchInput, TouchPhase};

#[derive(Component)]
pub struct LaunchPad;



#[derive(Resource)]
pub struct LaunchState {
    pub launched_at_time: Option<f64>,
    pub active_touch: Option<u64>,
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

#[derive(Resource)]
pub struct LaunchArmed(pub bool);

impl Default for LaunchArmed {
    fn default() -> Self { Self(true) }
}

#[derive(Resource, Default)]
pub struct ThrusterTouch {
    pub active_touch_id: Option<u64>,
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<LaunchArmed>();
    app.init_resource::<ThrusterTouch>();
    app.add_systems(
        Update,
        (
            start_new_launch.run_if(input_just_released(MouseButton::Left)),
            record_launch_time.run_if(input_just_pressed(MouseButton::Left)),
            deactivate_old_sats.run_if(input_just_pressed(MouseButton::Left)),
            update_fuel_label,
            arm_launch_on_earth_tap,
            start_launch_from_touch_end,
            select_satellite_on_touch,
            sun_thruster_touch,
        )
            .in_set(GameplaySystem),
    );
    app.insert_resource(LaunchState { launched_at_time: None, active_touch: None });
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
    window_q: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    solar_system_assets: Res<SolarSystemAssets>,
    mut launch_state: ResMut<LaunchState>,
    time: Res<Time>,
    mut score: ResMut<Score>,
    current_marked: Query<Entity, With<NavigationInstruments>>,
) {

    let Some(launch_pad_transform) = launch_pad_query.iter().next() else { return; };
    let launch_position = launch_pad_transform.translation;

    let Some((camera, camera_transform)) = camera_query.iter().next() else { return; };

    let Some(window) = window_q.iter().next() else { return; };
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
    if score.energy_stored > 10000. && score.energy_stored <20000. {
        lvl=2.;
        sprite = solar_system_assets.collector2.clone();
    }else if score.energy_stored >20000. {
        lvl=3.;
        sprite = solar_system_assets.collector3.clone();
    }else{
        lvl=1.;
        sprite= solar_system_assets.collector.clone();        
    }
    info!("Pay energy");
    if score.energy_stored >= 0.2 {
        score.energy_stored -= 0.2f32*lvl;
    } else {
        return;
    }
    // Ensure only the newly launched satellite will be selected
    for e in current_marked.iter() {
        commands.entity(e).remove::<NavigationInstruments>();
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
        HitBox { radius: 4.0 },
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
            font_size: 27.0,
            ..default()
        },
        TextColor(Color::from(GREEN)),
        ChildOf(collector_id),
        EnergyRateLabel,
        Pickable::IGNORE,
    ));

    commands.spawn((
        Text2d::new("0"),
        Transform::default().with_translation(Vec3::new(0.0, -1000.0, 0.0)).with_scale(Vec3::splat(10.0)),
        TextFont {
            font_size: 27.0,
            ..default()
        },
        TextColor(Color::from(WHITE)),
        ChildOf(collector_id),
        FuelLabel,
        Visibility::Visible,
        Pickable::IGNORE,
    ));

    launch_state.launched_at_time = None;
}

fn screen_to_world(
    camera_query: &Query<(&Camera, &GlobalTransform)>,
    window_q: &Query<&Window, With<PrimaryWindow>>,
    screen_pos: Vec2,
) -> Option<Vec2> {
    let (camera, cam_gt) = camera_query.iter().next()?;
    let _ = window_q.iter().next()?; // ensure window exists
    camera.viewport_to_world_2d(cam_gt, screen_pos).ok()
}

fn record_touch_start(
    mut er_touch: MessageReader<TouchInput>,
    time: Res<Time>,
    mut st: ResMut<LaunchState>,
    score: Res<Score>,
) {
    if score.energy_stored < 0.2 { return; }
    if st.active_touch.is_some() { return; }
    for t in er_touch.read() {
        if t.phase == TouchPhase::Started {
            st.launched_at_time = Some(time.elapsed_secs_f64());
            st.active_touch = Some(t.id);
            break;
        }
    }
}

fn start_launch_from_touch_end(
    mut er_touch: MessageReader<TouchInput>,
    mut commands: Commands,
    launch_pad_query: Query<&Transform, With<LaunchPad>>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    solar_system_assets: Res<SolarSystemAssets>,
    mut st: ResMut<LaunchState>,
    mut launch_armed: ResMut<LaunchArmed>,
    mut score: ResMut<Score>,
    current_marked: Query<Entity, With<NavigationInstruments>>,
) {
    let Some(launch_pad_transform) = launch_pad_query.iter().next() else { return; };
    let launch_position = launch_pad_transform.translation;

    if !launch_armed.0 { return; }

    // any touch end counts as single-tap launch when armed
    let mut screen_pos = None;
    for t in er_touch.read() {
        if t.phase == TouchPhase::Ended {
            screen_pos = Some(t.position);
            break;
        }
    }
    let Some(screen_pos) = screen_pos else { return; };

    let Some(world_pos) = screen_to_world(&camera_query, &window_q, screen_pos) else { return; };
    let launch_direction = (world_pos.extend(0.0) - launch_position).normalize_or_zero();
    if launch_direction == Vec3::ZERO { return; }

    info!("Launching new satellite (single-tap) towards {:?}", launch_direction);

    // fixed force for single-tap launch
    let force_multiplier: f64 = 1.0 * 10.0;

    let sprite;
    let lvl;
    if score.energy_stored > 10000. && score.energy_stored <20000. {
        lvl=2.;
        sprite = solar_system_assets.collector2.clone();
    }else if score.energy_stored >20000. {
        lvl=3.;
        sprite = solar_system_assets.collector3.clone();
    }else{
        lvl=1.;
        sprite= solar_system_assets.collector.clone();        
    }
    info!("Pay energy");
    if score.energy_stored >= 0.2 {
        score.energy_stored -= 0.2f32*lvl;
    } else {
        return;
    }
    // Ensure only the newly launched satellite will be selected
    for e in current_marked.iter() {
        commands.entity(e).remove::<NavigationInstruments>();
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
        HitBox { radius: 4.0 },
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
            font_size: 27.0,
            ..default()
        },
        TextColor(Color::from(GREEN)),
        ChildOf(collector_id),
        EnergyRateLabel,
        Pickable::IGNORE,
    ));

    commands.spawn((
        Text2d::new("0"),
        Transform::default().with_translation(Vec3::new(0.0, -1000.0, 0.0)).with_scale(Vec3::splat(10.0)),
        TextFont {
            font_size: 27.0,
            ..default()
        },
        TextColor(Color::from(WHITE)),
        ChildOf(collector_id),
        FuelLabel,
        Visibility::Visible,
        Pickable::IGNORE,
    ));

    // disarm after launch
    launch_armed.0 = false;

    st.launched_at_time = None;
    st.active_touch = None;
}

fn on_hover_collector_over(
    ev: On<Pointer<Over>>,
    mut commands: Commands,
    query: Query<Entity, With<NavigationInstruments>>,
) {
    // Hover only indicates potential selection; do not modify thrusters here
    println!("hover over collector {:?}", ev.entity);
    commands.entity(ev.entity).insert(NavigationInstruments);

    // Remove selection marker from others
    for entity in query.iter() {
        if entity != ev.entity {
            commands.entity(entity).remove::<NavigationInstruments>();
        }
    }
}



fn record_launch_time(time: Res<Time>, mut launch_state: ResMut<LaunchState>, score: Res<Score>) {
    if score.energy_stored < 0.2 {
        return;
    }
    if launch_state.launched_at_time.is_none() {
        launch_state.launched_at_time = Some(time.elapsed_secs_f64());
    }
}

fn deactivate_old_sats(
    mut commands: Commands,
    mut thruster_query: Query<(Entity, &mut Thruster), With<NavigationInstruments>>,
) {
    for (entity, mut thr) in thruster_query.iter_mut() {
        // turn off, don't remove the thruster component
        thr.active = false;
        // remove navigation instruments marker
        commands.entity(entity).remove::<NavigationInstruments>();
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





fn select_satellite_on_touch(
    mut er_touch: MessageReader<TouchInput>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut commands: Commands,
    sats: Query<(Entity, &GlobalTransform, &HitBox), With<Satellite>>,
    current_marked: Query<Entity, With<NavigationInstruments>>,
) {
    let Some((camera, cam_gt)) = camera_query.iter().next() else { return; };
    let _ = window_q.iter().next() else { return; };

    for t in er_touch.read() {
        if t.phase != TouchPhase::Ended { continue; }
        let Ok(world_pos) = camera.viewport_to_world_2d(cam_gt, t.position) else { continue; };

        let mut best: Option<(Entity, f32)> = None;
        for (e, gt, hb) in sats.iter() {
            let sat_pos = gt.translation().truncate();
            let dist = sat_pos.distance(world_pos);
            let radius = hb.radius;
            if dist <= radius {
                if let Some((_, best_dist)) = best {
                    if dist < best_dist { best = Some((e, dist)); }
                } else {
                    best = Some((e, dist));
                }
            }
        }

        if let Some((target, _)) = best {
            // Single tap: select only (NavigationInstruments)
            commands.entity(target).insert(NavigationInstruments);
            // remove selection from others
            for e in current_marked.iter() {
                if e != target {
                    commands.entity(e).remove::<NavigationInstruments>();
                }
            }
        }
    }
}

fn arm_launch_on_earth_tap(
    mut er_touch: MessageReader<TouchInput>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    earth_q: Query<(&GlobalTransform, Option<&HitBox>), With<Earth>>,
    mut launch_armed: ResMut<LaunchArmed>,
) {
    let Some((camera, cam_gt)) = camera_query.iter().next() else { return; };
    let _ = window_q.iter().next() else { return; };

    for t in er_touch.read() {
        if t.phase != TouchPhase::Ended { continue; }
        let Ok(world_pos) = camera.viewport_to_world_2d(cam_gt, t.position) else { continue; };
        for (gt, hb_opt) in earth_q.iter() {
            let earth_pos = gt.translation().truncate();
            let dist = earth_pos.distance(world_pos);
            let radius = hb_opt.map(|hb| hb.radius).unwrap_or(20.0);
            if dist <= radius {
                launch_armed.0 = true;
                return;
            }
        }
    }
}

fn sun_thruster_touch(
    mut er_touch: MessageReader<TouchInput>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    sun_q: Query<(&GlobalTransform, &HitBox), With<Sun>>,
    mut selected_thruster: Query<(&mut Thruster, Entity), (With<NavigationInstruments>, With<Thruster>)>,
    mut thr_touch: ResMut<ThrusterTouch>,
) {
    let Some((camera, cam_gt)) = camera_query.iter().next() else { return; };
    let _ = window_q.iter().next() else { return; };

    for t in er_touch.read() {
        match t.phase {
            TouchPhase::Started => {
                if thr_touch.active_touch_id.is_some() { continue; }
                let Ok(world_pos) = camera.viewport_to_world_2d(cam_gt, t.position) else { continue; };
                // check hit on sun
                for (gt, hb) in sun_q.iter() {
                    let sun_pos = gt.translation().truncate();
                    let dist = sun_pos.distance(world_pos);
                    let radius = hb.radius;
                    if dist <= radius {
                        if let Ok((mut thr, _)) = selected_thruster.single_mut() {
                            thr.active = true;
                            thr_touch.active_touch_id = Some(t.id);
                        }
                        break;
                    }
                }
            }
            TouchPhase::Ended | TouchPhase::Canceled => {
                if thr_touch.active_touch_id == Some(t.id) {
                    if let Ok((mut thr, _)) = selected_thruster.single_mut() {
                        thr.active = false;
                    }
                    thr_touch.active_touch_id = None;
                }
            }
            _ => {}
        }
    }
}

