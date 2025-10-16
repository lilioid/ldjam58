use crate::GameplaySystem;
use crate::collision::FatalCollisionEvent;
use crate::launching::{LaunchState, SatellitePriceFactor};
use crate::score::Score;
use crate::screens::{gameover, Screen};
use crate::sun_system::{SolarSystemAssets, Sun};
use crate::sun_system::asteroids::AsteroidSwarmSpawned;

// Generated at compile-time by build.rs
include!(concat!(env!("OUT_DIR"), "/build_info.rs"));
use bevy::prelude::*;
use bevy::ui_render::stack_z_offsets::BORDER;
use crate::sound::Music;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Gameplay), setup_hud)
            .add_systems(
                Update,
                (
                    update_hud,
                    update_crash_indicators,
                    update_launch_pad_ui,
                    update_zoom_level,
                    update_explanation_text,
                    update_debris_warning,
                    update_countdown,
                    handle_music_button,
                    update_music_button_visual,
                )
                .in_set(GameplaySystem),
            );
        app.add_observer(handle_fatal_collision_event_for_hud);
        app.add_observer(handle_asteroid_swarm_spawned);
        app.insert_resource(HudState {
            just_destroyed: None,
            already_pressed_space: false,
            already_pressed_lmb: false,
            is_mobile: false,
        });
    }
}

#[derive(Component)]
struct EnergyRateText;

#[derive(Component)]
struct EnergyStorageText;

#[derive(Component)]
struct CrashIndicator {
    timer: Timer,
    blink_count: u32,
    blink_state: bool,
}

#[derive(Component)]
struct LaunchBarText;

#[derive(Component)]
struct ZoomLevelText;

#[derive(Component)]
struct ExplanationText;

#[derive(Component)]
struct ExplanationContainer;

#[derive(Resource)]
struct HudState {
    just_destroyed: Option<Entity>,
    already_pressed_space: bool,
    already_pressed_lmb: bool,
    is_mobile: bool,
}

#[derive(Component)]
struct DebrisWarning {
    timer: Timer,
}

#[derive(Component)]
struct CountdownText;

// Music button components
#[derive(Component)]
struct MusicButton;
#[derive(Component)]
struct MusicButtonText;

fn setup_hud(mut commands: Commands, solar_system_assets: Res<SolarSystemAssets>) {
    // TOP LEFT: Energy Rate and Total Energy Storage
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(15.0),
            left: Val::Px(15.0),
            width: Val::Px(330.0),
            height: Val::Px(125.0),
            border: UiRect::all(Val::Px(BORDER)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
        Outline {
            width: Val::Px(2.0),
            offset: Default::default(),
            color: Color::xyz(0.4811, 0.3064, 0.0253),
        },
        children![
            (
                Text::new("ENERGY RATE\n0"),
                Node {
                    position_type: PositionType::Relative,
                    top: Val::Px(5.0),
                    left: Val::Px(15.0),
                    border: UiRect::all(Val::Px(4.0)),
                    ..default()
                },
                TextFont {
                    font: solar_system_assets.font.clone(),
                    ..default()
                },
                TextColor(Color::xyz(0.4811, 0.3064, 0.0253)),
                EnergyRateText
            ),
            (
                Text::new("TOTAL:\n0"),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(65.0),
                    left: Val::Px(15.0),
                    ..default()
                },
                TextFont {
                    font: solar_system_assets.font.clone(),
                    ..default()
                },
                TextColor(Color::xyz(0.4811, 0.3064, 0.0253)),
                EnergyStorageText
            )
        ],
    ));

    // BOTTOM LEFT — Music toggle button (to the right of the zoom indicator)
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(15.0),
            left: Val::Px(110.0),
            width: Val::Px(80.0),
            height: Val::Px(50.0),
            border: UiRect::all(Val::Px(BORDER)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
        Outline {
            width: Val::Px(2.0),
            offset: Default::default(),
            color: Color::xyz(0.4811, 0.3064, 0.0253),
        },
        Button,
        MusicButton,
        children![
            (
                Text::new("[)] )))"),
                TextFont {
                    font: solar_system_assets.font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::xyz(0.4811, 0.3064, 0.0253)),
                MusicButtonText,
            )
        ],
    ));

    let text_center = Justify::Center;

    // TOP RIGHT: Countdown to game end
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(15.0),
            right: Val::Px(15.0),
            width: Val::Px(150.0),
            height: Val::Px(60.0),
            border: UiRect::all(Val::Px(BORDER)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
        Outline {
            width: Val::Px(2.0),
            offset: Default::default(),
            color: Color::xyz(0.4811, 0.3064, 0.0253),
        },
        children![
            (
                Text::new("TIME\n10:00"),
                Node {
                    position_type: PositionType::Relative,
                    top: Val::Px(5.0),
                    left: Val::Px(15.0),
                    ..default()
                },
                TextFont {
                    font: solar_system_assets.font.clone(),
                    ..default()
                },
                TextColor(Color::xyz(0.4811, 0.3064, 0.0253)),
                CountdownText
            )
        ],
    ));

    // BOTTOM RIGHT: Launch Pad UI
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(15.0),
            right: Val::Px(15.0),
            width: Val::Px(45.0),
            height: Val::Px(550.0),
            border: UiRect::all(Val::Px(BORDER)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
        Outline {
            width: Val::Px(2.0),
            offset: Default::default(),
            color: Color::xyz(0.4811, 0.3064, 0.0253),
        },
        children![
            (
                Text::new("PRESS\nLMB"),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(15.0),
                    left: Val::Px(5.0),
                    ..default()
                },
                TextLayout::new_with_justify(text_center),
                TextColor(Color::xyz(0.4811, 0.3064, 0.0253)),
                TextFont {
                    font: solar_system_assets.font.clone(),
                    font_size: 12.0,
                    ..default()
                },
            ),
            (
                LaunchBarText,
                Text::new(get_vertical_ascii_bar(0.0)),


                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(45.0),
                    right: Val::Px(15.0),
                    ..default()
                },
                TextFont {
                    font: solar_system_assets.font.clone(),
                    font_size: 25.0,
                    ..default()
                },
                TextColor(Color::xyz(0.4811, 0.3064, 0.0253)),
            ),
        ],
    ));

    //BOTTOM LEFT: ZOOM LEVEL INDICATOR
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(15.0),
            left: Val::Px(15.0),
            width: Val::Px(80.0),
            height: Val::Px(50.0),
            border: UiRect::all(Val::Px(BORDER)),

            ..default()
        },
        BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
        Outline {
            width: Val::Px(2.0),
            offset: Default::default(),
            color: Color::xyz(0.4811, 0.3064, 0.0253),
        },
        children![
            (
                Text::new("1.0x"),
                ZoomLevelText,
                Node {
                    position_type: PositionType::Relative,
                    top: Val::Px(12.0),
                    left: Val::Px(15.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                TextFont {
                    font: solar_system_assets.font.clone(),
                    ..default()
                },
                TextColor(Color::xyz(0.4811, 0.3064, 0.0253)),
            )
        ],
    ));


    //MIDDLE OF SCREEN: Explanation text
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Pickable::IGNORE,
        children![
        (
            Node {
                width: Val::Px(300.0),
                height: Val::Px(45.0),
                border: UiRect::all(Val::Px(BORDER)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            Pickable::IGNORE,
            BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
            Outline {
                width: Val::Px(2.0),
                offset: Default::default(),
                color: Color::xyz(0.4811, 0.3064, 0.0253),
            },
            ExplanationContainer,
            children![
                (
                    Text::new("HOLD/RELEASE LMB TO LAUNCH"),
                    TextFont {
                        font: solar_system_assets.font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::xyz(0.4811, 0.3064, 0.0253)),
                    ExplanationText,
                    Pickable::IGNORE,
                )
            ],
        )
    ],
    ));

    //MIDDLE OF SCREEN: DEBRIS WARNING
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Pickable::IGNORE,
        Visibility::Hidden,
        DebrisWarning {
            timer: Timer::from_seconds(3.0, TimerMode::Once),
        },
        children![
            (
                Node {
                    width: Val::Px(300.0),
                    height: Val::Px(60.0),
                    border: UiRect::all(Val::Px(BORDER)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                Pickable::IGNORE,
                BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
                Outline {
                    width: Val::Px(3.0),
                    offset: Default::default(),
                    color: Color::xyz(0.4811, 0.3064, 0.0253),
                },
                children![
                    (
                        Text::new("DEBRIS WARNING ! !"),
                        TextFont {
                            font: solar_system_assets.font.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::xyz(0.4811, 0.3064, 0.0253)),
                        Pickable::IGNORE,
                    )
                ],
            )
        ],
    ));
}

fn update_hud(
    player_data: Res<Score>,
    time: Res<Time>,
    price: Res<SatellitePriceFactor>,
    mut energy_rate_query: Query<
        (&mut Text, &EnergyRateText),
        (With<EnergyRateText>, Without<EnergyStorageText>),
    >,
    mut energy_storage_query: Query<
        (&mut Text, &mut TextColor, &EnergyStorageText),
        (With<EnergyStorageText>, Without<EnergyRateText>),
    >,
) {
    let percent_rate = player_data.energy_rate /400.;
    let percent_stored = player_data.energy_stored /100000.;
    if player_data.is_changed() {
        for (mut text, _) in energy_rate_query.iter_mut() {
            text.0 = format!(
                "ENERGY RATE\n{} {:.3}YW",
                get_ascii_bar(percent_rate.clamp(0.0, 1.0)),
                player_data.energy_rate
            )
        }

        for (mut text, mut _color, _) in energy_storage_query.iter_mut() {
            text.0 = format!(
                "TOTAL:\n{} {:.0}YWh",
                get_ascii_bar(percent_stored.clamp(0.0, 1.0)),
                player_data.energy_stored
            )
        }
    }

    // Flash the energy storage text red when there isn't enough energy to launch a satellite
    let lvl: f32 = if player_data.energy_stored > 20_000.0 {
        3.0
    } else if player_data.energy_stored > 10_000.0 {
        2.0
    } else {
        1.0
    };
    let required_energy = price.factor * lvl;
    let insufficient = player_data.energy_stored < required_energy;
    let blink_on = (time.elapsed_secs() * 6.0).sin() > 0.0; // ~1 Hz
    for (_text, mut color, _) in energy_storage_query.iter_mut() {
        if insufficient && blink_on {
            color.0 = Color::srgb(1.0, 0.1, 0.1);
        } else {
            color.0 = Color::xyz(0.4811, 0.3064, 0.0253);
        }
    }
}

fn get_ascii_bar(percentage: f32) -> String {
    let total_bars = 15;
    let filled_bars = (percentage * total_bars as f32).round() as usize;
    let empty_bars = total_bars - filled_bars;

    let filled_part = "█".repeat(filled_bars);
    let empty_part = "░".repeat(empty_bars);

    format!("{}{}", filled_part, empty_part)
}

fn handle_fatal_collision_event_for_hud(
    event: On<FatalCollisionEvent>,
    mut commands: Commands,
    entity_query: Query<(&Transform, Entity)>,
    solar_system_assets: Res<SolarSystemAssets>,
    sun_query: Query<(), With<Sun>>,
    mut just_destroyed: ResMut<HudState>,
) {
    let (entity_transform, _) = entity_query
        .get(event.destroyed)
        .expect("Wanted to get transform of destroyed entity but entity does not exist!");

    // Skip HUD X marker when the other collider is the sun
    if sun_query.get(event.other).is_ok() {
        return;
    }

    if just_destroyed.just_destroyed == Some(event.other) {
        //already showing crash indicator for the other entity; skipping to avoid overlapping indicators
        return;
    }


    commands.spawn((
        Name::new("crash"),
        Transform::from_translation(entity_transform.translation).with_scale(Vec3::splat(0.01)),
        Sprite::from(solar_system_assets.crash.clone()),
        CrashIndicator {
            timer: Timer::from_seconds(0.15, TimerMode::Repeating),
            blink_count: 0,
            blink_state: true,
        },
        Visibility::Visible,
    ));

    just_destroyed.just_destroyed = Some(event.destroyed);
}

fn update_crash_indicators(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut CrashIndicator, &mut Visibility)>,
    mut hud_state: ResMut<HudState>,
) {
    for (entity, mut crash_indicator, mut visibility) in query.iter_mut() {
        crash_indicator.timer.tick(time.delta());

        if crash_indicator.timer.just_finished() {
            if crash_indicator.blink_count < 4 {
                crash_indicator.blink_state = !crash_indicator.blink_state;
                *visibility = if crash_indicator.blink_state {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                crash_indicator.blink_count += 1;
            } else if crash_indicator.blink_count == 4 {
                *visibility = Visibility::Visible;
                crash_indicator.timer = Timer::from_seconds(1.0, TimerMode::Once);
                crash_indicator.blink_count += 1;
            } else {
                commands.entity(entity).despawn();
                hud_state.just_destroyed = None;
            }
        }
    }
}

fn update_launch_pad_ui(
    mut launch_bar_query: Query<&mut Text, With<LaunchBarText>>,
    time: Res<Time>,
    launch_state: Res<LaunchState>,
) {
    let mut launch_bar_text = launch_bar_query.single_mut().unwrap();

    if let Some(launch_start_time) = launch_state.launched_at_time {
        let held_duration = time.elapsed_secs_f64() - launch_start_time;
        let clamped_duration = held_duration.min(1.0);

        let vertical_bar = get_vertical_ascii_bar(clamped_duration as f32);
        launch_bar_text.0 = vertical_bar;
    } else {
        launch_bar_text.0 = get_vertical_ascii_bar(0.0);
    }
}

fn get_vertical_ascii_bar(percentage: f32) -> String {
    let total_bars = 15;
    let filled_bars = (percentage * total_bars as f32).round() as usize;

    let mut result = String::from("╦\n");

    for i in 0..total_bars {
        if i >= (total_bars - filled_bars) {
            result.push('█');
        } else {
            result.push('│');
        }
        result.push('\n');
    }

    result.push('╩');
    result
}

fn update_zoom_level (
    camera_query: Query<(&Camera, &Transform)>,
    mut zoom_level_query: Query<&mut Text, With<ZoomLevelText>>,
) {
    let (_, transform) = camera_query.single().unwrap();
    let mut zoom_level_text = zoom_level_query.single_mut().unwrap();

    let mut zoom_level = 1.0 / transform.scale.x;
    zoom_level = zoom_level / 4.0;
    zoom_level_text.0 = format!("{:.1}x", zoom_level);
}

fn update_explanation_text(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut er_touch: MessageReader<bevy::input::touch::TouchInput>,
    mut explanation_text_query: Query<&mut Text, With<ExplanationText>>,
    mut explanation_container_query: Query<&mut Visibility, With<ExplanationContainer>>,
    mut hud_state: ResMut<HudState>,
) {
    let mut explanation_text = explanation_text_query.single_mut().unwrap();
    let mut container_visibility = explanation_container_query.single_mut().unwrap();

    // Mobile detection based on presence of touch events (runtime, no web_sys needed)
    let mut saw_touch = false;
    for _ in er_touch.read() { saw_touch = true; break; }
    if saw_touch { hud_state.is_mobile = true; }

    if hud_state.is_mobile {
        // Mobile placeholder explanations
        if !hud_state.already_pressed_space {
            // First message: how to launch
            explanation_text.0 = "TAP EARTH AND RELEASE TO LAUNCH".to_string();
            if saw_touch {
                hud_state.already_pressed_space = true;
                explanation_text.0 = "TAP THE SUN TO ACTIVATE THRUSTER".to_string();
            }
        } else if !hud_state.already_pressed_lmb {
            // Second interaction hides the hint box
            if saw_touch {
                hud_state.already_pressed_lmb = true;
                *container_visibility = Visibility::Hidden;
            }
        }
        return;
    }else {
        // Desktop (mouse/keyboard) behavior stays unchanged
        if !hud_state.already_pressed_space {
            if mouse_input.pressed(MouseButton::Left) {
                hud_state.already_pressed_space = true;
                explanation_text.0 = "PRESS SPACE TO SLOW DOWN".to_string();
            }
        } else if !hud_state.already_pressed_lmb {
            if keyboard_input.pressed(KeyCode::Space) {
                hud_state.already_pressed_lmb = true;
                *container_visibility = Visibility::Hidden;
            }
        }
    }
}

fn update_debris_warning(
    mut query: Query<(&mut DebrisWarning, &mut Visibility)>,
    time: Res<Time>,
) {
    let Ok((mut warning, mut visibility)) = query.single_mut() else {
        return;
    };

    // Update timer and hide when finished
    if *visibility == Visibility::Visible {
        warning.timer.tick(time.delta());
        if warning.timer.is_finished() {
            *visibility = Visibility::Hidden;
        }
    }
}

fn handle_asteroid_swarm_spawned(
    _trigger: On<AsteroidSwarmSpawned>,
    mut query: Query<(&mut DebrisWarning, &mut Visibility)>,
) {
    let Ok((mut warning, mut visibility)) = query.single_mut() else {
        return;
    };

    warning.timer.reset();
    *visibility = Visibility::Visible;
}


fn update_countdown(
    time: Res<Time>,
    game_end: Option<Res<gameover::GameEnd>>,
    mut query: Query<&mut Text, With<crate::hud::CountdownText>>,
) {
    let Ok(mut text) = query.single_mut() else { return; };
    let Some(game_end) = game_end else { return; };

    if !game_end.enabled {
        return;
    }

    let remaining = (game_end.game_end_time - time.elapsed_secs()).max(0.0);
    let mins = (remaining / 60.0).floor() as i32;
    let secs = (remaining % 60.0).floor() as i32;
    text.0 = format!("{}\n{:02}:{:02}s", BUILD_LABEL, mins, secs);
}

// --- Music toggle HUD systems ---
fn update_music_button_visual(
    music_exists: Query<(), With<Music>>,
    mut text_q: Query<&mut Text, With<MusicButtonText>>,
) {
    let Ok(mut text) = text_q.single_mut() else { return; };
    let speaker_on = "[)] )))";
    let speaker_off = "[)] x";

    if music_exists.iter().next().is_some() {
        text.0 = speaker_on.to_string();
    } else {
        text.0 = speaker_off.to_string();
    }
}

fn handle_music_button(
    mut commands: Commands,
    assets: Res<SolarSystemAssets>,
    mut q: Query<&Interaction, (With<MusicButton>, Changed<Interaction>)>,
    music_q: Query<Entity, With<Music>>,
) {
    for interaction in &mut q {
        if *interaction == Interaction::Pressed {
            if let Ok(e) = music_q.single() {
                commands.entity(e).despawn();
            } else {
                commands.spawn((
                    AudioPlayer::new(assets.music_loop.clone()),
                    PlaybackSettings::LOOP,
                    Music,
                ));
            }
        }
    }
}
