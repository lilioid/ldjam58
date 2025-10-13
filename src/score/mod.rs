use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;
use crate::GameplaySystem;
use crate::launching::CollectorStats;
use crate::sun_system::{Level, Satellite, Sun};
use std::collections::VecDeque;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, update_score.in_set(GameplaySystem));
    app.insert_resource(Score::default());
}

#[derive(Resource)]
pub struct Score {
    pub energy_rate: f32,
    pub energy_stored: f32,
    rate_history: VecDeque<(f32, f32)>, // (timestamp, rate)
    history_duration: f32,
}

impl Default for Score {
    fn default() -> Self {
        Self {
            energy_rate: 1.0,
            energy_stored: 1.0,
            rate_history: VecDeque::new(),
            history_duration: 60.0,
        }
    }
}

#[derive(Component)]
pub struct EnergyRateLabel;

fn update_score(
    mut score: ResMut<Score>,
    mut satellite_query: Query<(Entity, &Transform, &mut CollectorStats, &Level), With<Satellite>>,
    sun_query: Query<&Transform, (With<Sun>, Without<Satellite>) >,
    mut label_query: Query<(&ChildOf, &mut Text2d), With<EnergyRateLabel>>,

    time: Res<Time>,
) {
    let Some(sun_transform) = sun_query.iter().next() else { return; };
    let sun_position = sun_transform.translation;

    let current_time = time.elapsed_secs();
    let mut instant_rate = 0.01;

    for (entity, satellite_transform, mut collector_stats, level) in satellite_query.iter_mut() {
        let distance = satellite_transform.translation.distance(sun_position);
        if distance > 0.0 {
            let mut individual_rate = 2.0 / distance;
            collector_stats.energy_rate = individual_rate;
            individual_rate = individual_rate*level.level*200.;
            instant_rate += individual_rate;

            for (parent, mut text) in label_query.iter_mut() {
                if parent.get() == entity {
                    **text = format!("+{:.2}", individual_rate);
                    break;
                }
            }
        }
    }

    score.rate_history.push_back((current_time, instant_rate));

    // remove old entries
    while let Some(&(timestamp, _)) = score.rate_history.front() {
        if current_time - timestamp > score.history_duration {
            score.rate_history.pop_front();
        } else {
            break;
        }
    }

    // calculate average
    if !score.rate_history.is_empty() {
        let sum: f32 = score.rate_history.iter().map(|(_, rate)| rate).sum();
        score.energy_rate = sum / score.rate_history.len() as f32;
    } else {
        score.energy_rate = instant_rate;
    }

    score.energy_stored += score.energy_rate * time.delta_secs();
}
