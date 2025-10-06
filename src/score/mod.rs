use bevy::ecs::relationship::Relationship;
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

#[derive(Component)]
pub struct EnergyRateLabel;

fn update_score(
    mut score: ResMut<Score>,
    mut satellite_query: Query<(Entity, &Transform, &mut CollectorStats), With<Satellite>>,
    sun_query: Query<&Transform, (With<Sun>, Without<Satellite>)>,
    mut label_query: Query<(&ChildOf, &mut Text2d), With<EnergyRateLabel>>,
    time: Res<Time>,
) {
    let sun_transform = sun_query.single();
    let sun_position = sun_transform.unwrap().translation;

    score.energy_rate = 0.01;

    for (entity, satellite_transform, mut collector_stats) in satellite_query.iter_mut() {
        let distance = satellite_transform.translation.distance(sun_position);
        if distance > 0.0 {
            let mut individual_rate = 2.0 / distance;
            collector_stats.energy_rate = individual_rate;
            individual_rate *= 2.0;
            score.energy_rate += individual_rate;

            // Update the label for this satellite
            for (parent, mut text) in label_query.iter_mut() {
                if parent.get() == entity {
                    **text = format!("+{:.2}", individual_rate);
                    break;
                }
            }
        }
    }

    score.energy_stored += score.energy_rate * time.delta_secs();
}
