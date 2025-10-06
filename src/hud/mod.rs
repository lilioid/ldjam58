use crate::GameplaySystem;
use crate::collision::FatalCollisionEvent;
use crate::launching::{LaunchPad, LaunchState};
use crate::score::Score;
use crate::screens::Screen;
use crate::sun_system::SolarSystemAssets;
use bevy::prelude::*;
use bevy::ui_render::stack_z_offsets::BORDER;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Gameplay), setup_hud)
            .add_systems(
                Update,
                (update_hud, update_crash_indicators, update_launch_pad_ui).in_set(GameplaySystem),
            );
        app.add_observer(handle_fatal_collision_event_for_hud);
        app.insert_resource(HudState {
            just_destroyed: None,
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

#[derive(Resource)]
struct HudState {
    just_destroyed: Option<Entity>,
}

fn setup_hud(mut commands: Commands, solar_system_assets: Res<SolarSystemAssets>) {
    // TOP LEFT: Energy Rate and Total Energy Storage
    let container = commands.spawn((
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
                Text::new("TOTAL\n0"),
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

    let text_center = Justify::Center;

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
}

fn update_hud(
    player_data: Res<Score>,
    mut energy_rate_query: Query<
        (&mut Text, &EnergyRateText),
        (With<EnergyRateText>, Without<EnergyStorageText>),
    >,
    mut energy_storage_query: Query<
        (&mut Text, &EnergyStorageText),
        (With<EnergyStorageText>, Without<EnergyRateText>),
    >,
) {
    if player_data.is_changed() {
        for (mut text, _) in energy_rate_query.iter_mut() {
            text.0 = format!(
                "ENERGY RATE\n{} {:.5}GW",
                get_ascii_bar(player_data.energy_rate.clamp(0.0, 1.0)),
                player_data.energy_rate
            )
        }

        for (mut text, _) in energy_storage_query.iter_mut() {
            text.0 = format!(
                "TOTAL:\n{} {:.2}GWh",
                get_ascii_bar((player_data.energy_stored / 10.0).clamp(0.0, 1.0)),
                player_data.energy_stored
            )
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
    mut just_destroyed: ResMut<HudState>,
) {
    let (entity_transform, _) = entity_query
        .get(event.destroyed)
        .expect("Wanted to get transform of destroyed entity but entity does not exist!");

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
            result.push('║');
        } else {
            result.push('│');
        }
        result.push('\n');
    }

    result.push('╩');
    result
}
