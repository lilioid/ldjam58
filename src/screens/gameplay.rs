//! The screen state for the main gameplay.

use bevy::input::common_conditions::input_just_pressed;
use bevy::input::mouse::MouseWheel;
use bevy::input::touch::{TouchInput, TouchPhase};
use crate::sun_system::{init_sun_system};
use bevy::prelude::*;
use crate::GameplaySystem;
use crate::screens::Screen;
use std::collections::HashMap;

#[derive(Component)]
struct CameraZoom {
    level: usize,
}

#[derive(Resource, Default)]
struct PinchZoomState {
    prev_dist: Option<f32>,
    active: HashMap<u64, Vec2>,
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<PinchZoomState>();
    app.add_systems(OnEnter(Screen::Gameplay), setup_scene);
    app.add_systems(OnEnter(Screen::Gameplay), init_sun_system);
    app.add_systems(Update, camera_zoom.in_set(GameplaySystem));
    app.add_systems(Update, camera_pinch_zoom.in_set(GameplaySystem));
    app.add_systems(Update, change_time_speed::<2>.run_if(input_just_pressed(KeyCode::ArrowUp)));
    app.add_systems(Update, change_time_speed::<-2>.run_if(input_just_pressed(KeyCode::ArrowDown)));
}

fn setup_scene(mut commands: Commands) {
    commands.spawn((
        Name::new("Camera"),
        Camera2d,
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        CameraZoom { level: 2 },
    ));
}


fn change_time_speed<const DELTA: i8>(mut time: ResMut<Time<Virtual>>) {
    let time_speed = (time.relative_speed() + DELTA as f32)
        .round()
        .clamp(1., 20.);

    info!("Time speed changed to {}", time_speed);
    // set the speed of the virtual time to speed it up or slow it down
    time.set_relative_speed(time_speed);
}

fn camera_zoom(
    mut scroll_evr: MessageReader<MouseWheel>,
    mut query: Query<(&mut Transform, &mut CameraZoom), With<Camera>>,
) {
    //stepped zoom with predefined levels
    let zoom_levels = [0.1, 0.15, 0.25, 0.5, 0.75];

    if let Ok((mut transform, mut camera_zoom)) = query.single_mut() {
        for ev in scroll_evr.read() {
            if ev.y > 0.0 && camera_zoom.level > 0 {
                camera_zoom.level -= 1;
            } else if ev.y < 0.0 && camera_zoom.level < zoom_levels.len() - 1 {
                camera_zoom.level += 1;
            }
        }

        let zoom_level = zoom_levels[camera_zoom.level];
        transform.scale = Vec3::splat(zoom_level);
    }
}

fn camera_pinch_zoom(
    mut pinch: ResMut<PinchZoomState>,
    mut er_touch: EventReader<TouchInput>,
    mut query: Query<(&mut Transform, &mut CameraZoom), With<Camera>>,
) {
    // Update active touches map
    for ev in er_touch.read() {
        match ev.phase {
            TouchPhase::Started | TouchPhase::Moved => {
                pinch.active.insert(ev.id, ev.position);
            }
            TouchPhase::Ended | TouchPhase::Canceled => {
                pinch.active.remove(&ev.id);
            }
        }
    }

    // Only apply when two or more touches are active (use the first two)
    if pinch.active.len() < 2 { 
        pinch.prev_dist = None; 
        return; 
    }

    let mut iter = pinch.active.values();
    let p1 = *iter.next().unwrap();
    let p2 = *iter.next().unwrap();
    let dist = p1.distance(p2);

    if let Some(prev) = pinch.prev_dist {
        let delta = dist - prev;
        // threshold in logical pixels to avoid jitter
        let threshold = 5.0;
        if delta.abs() > threshold {
            if let Ok((mut transform, mut camera_zoom)) = query.single_mut() {
                // same zoom levels as mouse
                let zoom_levels = [0.1, 0.15, 0.25, 0.5, 0.75];
                if delta > 0.0 && camera_zoom.level < zoom_levels.len() - 1 {
                    camera_zoom.level += 1;
                } else if delta < 0.0 && camera_zoom.level > 0 {
                    camera_zoom.level -= 1;
                }
                let zoom_level = zoom_levels[camera_zoom.level];
                transform.scale = Vec3::splat(zoom_level);
            }
        }
    }

    pinch.prev_dist = Some(dist);
}