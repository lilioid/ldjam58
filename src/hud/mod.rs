use bevy::prelude::*;
use crate::score::Score;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_hud)
            .add_systems(Update, update_hud);
    }
}

#[derive(Component)]
struct EnergyRateText;

#[derive(Component)]
struct EnergyStorageText;

fn setup_hud(
    mut commands: Commands,
) {

    commands.spawn((
        Text::new("Energy Rate: 0"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(15.0),
            ..default()
        },

        EnergyRateText
    ));

    commands.spawn((
        Text::new("Total: 0"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(30.0),
            left: Val::Px(15.0),
            ..default()
        },

        EnergyStorageText
    ));

}

fn update_hud(
    player_data: Res<Score>,
    mut energy_rate_query: Query<(&mut Text, &EnergyRateText), (With<EnergyRateText>, Without<EnergyStorageText>)>,
    mut energy_storage_query: Query<(&mut Text, &EnergyStorageText), (With<EnergyStorageText>, Without<EnergyRateText>)>,
) {
    if player_data.is_changed() {
        for (mut text, _) in energy_rate_query.iter_mut() {
            text.0 = format!("Energy Rate: {}", player_data.energy_rate)
        }

        for (mut text, _) in energy_storage_query.iter_mut() {
            text.0 = format!("Total: {}", player_data.energy_stored)
        }
    }
}