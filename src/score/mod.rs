use bevy::prelude::*;
use crate::collision::FatalCollisionEvent;
use crate::GameplaySystem;
use crate::launching::CollectorStats;
use crate::screens::Screen;
use crate::sun_system::{Satellite, Sun};

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, update_score.in_set(GameplaySystem));
    app.insert_resource(Score{energy_rate:1.0,energy_stored:1.0});
}

#[derive(Resource, Default)]
pub struct Score {
    pub energy_rate: f32,
    pub energy_stored: f32,

}

fn update_score(
    mut score: ResMut<Score>,
    mut satellite_query: Query<(&Transform, &mut CollectorStats), With<Satellite>>,
    sun_query: Query<(&Transform), (With<Sun>, Without<Satellite>)>,
    time: Res<Time>,
) {

    let sun_transform = sun_query.single().unwrap();
    let sun_position = sun_transform.translation;

    score.energy_rate = 0.0;

    for (satellite_transform, mut collector_stats) in satellite_query.iter_mut() {
        let distance = satellite_transform.translation.distance(sun_position);
        if distance > 0.0 {
            let individual_rate = 1.0 / distance;
            collector_stats.energy_rate = individual_rate;
            score.energy_rate += individual_rate;
        }
    }

    score.energy_stored += score.energy_rate * time.delta_secs();
}