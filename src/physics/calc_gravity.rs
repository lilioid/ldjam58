use bevy::ecs::query::QuerySingleError;
use crate::physics::apply_directional_force::{GravityForce, Mass};
use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct Attractor;

pub(super) fn calc_gravity(
    attractor: Query<(&Mass, &Transform), (With<Attractor>, Without<GravityForce>)>,
    mut attractee: Query<(&Mass, &Transform, &mut GravityForce), Without<Attractor>>,
) {
    let attractor = match attractor.single() {
        Ok(ok) => ok,
        Err(QuerySingleError::NoEntities(_)) => { return },
        Err(QuerySingleError::MultipleEntities(e)) => panic!("Found multiple attractors in world but n-body physics is not supported: {e}"),
    };

    attractee.iter_mut().for_each(|(i_mass, i_transform, mut i_gravity_force)| {
        let pos1 = attractor.1.translation;
        let pos2 = i_transform.translation;

        let distance = pos1.distance(pos2);
        let direction = (pos2 - pos1).normalize_or_zero();

        let f = calc_gravity_force(attractor.0.0, i_mass.0, distance);
        let directional_force = direction * f;

        debug!("Applying gravity force between attractor {:?} and attractee {:?} -> {f}", pos1, pos2);
        i_gravity_force.0 = directional_force.xy();
    });
}

fn calc_gravity_force(m1: f32, m2: f32, r: f32) -> f32 {
    let G: f32 = 6.674 * 10.0f32.powi(-11);
    G * ((m1 * m2) / r.powi(2))
}
