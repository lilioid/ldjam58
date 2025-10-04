use bevy::prelude::*;
use crate::screens::Screen;
use crate::sun_system::{Satellite, Sun};

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, update_score.run_if(in_state(Screen::Gameplay)));
    app.insert_resource(Score::default());
}

#[derive(Resource, Default)]
pub struct Score {
    pub energy_rate: f32,
    pub energy_stored: f32,

}

fn update_score(
    mut score: ResMut<Score>,
    satellite_query: Query<(&Transform), With<Satellite>>,
    sun_query: Query<(&Transform), (With<Sun>, Without<Satellite>)>,
    time: Res<Time>,
) {

    let sun_transform = sun_query.single().unwrap();
    let sun_position = sun_transform.translation;

    score.energy_rate = 0.0;

    for satellite_transform in satellite_query.iter() {
        let distance = satellite_transform.translation.distance(sun_position);
        if distance > 0.0 {
            score.energy_rate += 1.0 / distance;
        }
    }

    score.energy_stored += score.energy_rate * time.delta_secs();
}