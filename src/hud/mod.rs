use crate::score::Score;
use crate::sun_system::SolarSystemAssets;
use bevy::prelude::*;
use bevy::ui_render::stack_z_offsets::BORDER;
use crate::GameplaySystem;
use crate::screens::Screen;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Gameplay), setup_hud)
            .add_systems(Update, update_hud);
    }
}

#[derive(Component)]
struct EnergyRateText;

#[derive(Component)]
struct EnergyStorageText;

fn setup_hud(mut commands: Commands, solar_system_assets: Res<SolarSystemAssets>) {
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
            text.0 = format!("ENERGY RATE\n{} {:.5}GW", get_ascii_bar(player_data.energy_rate.clamp(0.0, 1.0)),player_data.energy_rate)
        }

        for (mut text, _) in energy_storage_query.iter_mut() {
            text.0 = format!("TOTAL:\n{} {:.2}GWh", get_ascii_bar((player_data.energy_stored / 10.0).clamp(0.0, 1.0)),player_data.energy_stored)
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
