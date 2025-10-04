use bevy::ecs::query::QuerySingleError;
use bevy::log::tracing;
use crate::physics::directional_forces::{GravityForce, Mass};
use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct Attractor;

#[derive(Component, Debug, Copy, Clone)]
pub struct Attractee;

pub(super) fn apply_gravity(
    attractor: Query<(&Mass, &Transform), ((With<Attractor>, Without<Attractee>), Without<GravityForce>)>,
    mut attractee: Query<(&Mass, &Transform, &mut GravityForce), (Without<Attractor>, With<Attractee>)>,
) {
    let attractor = match attractor.single() {
        Ok(ok) => ok,
        Err(QuerySingleError::NoEntities(_)) => { return },
        Err(QuerySingleError::MultipleEntities(e)) => panic!("Found multiple attractors in world but n-body physics is not supported: {e}"),
    };

    attractee.iter_mut().for_each(|(i_mass, i_transform, mut i_gravity_force)| {
        i_gravity_force.0 = calc_gravity_force(attractor.0, attractor.1, i_mass, i_transform);
    });
}

pub fn calc_gravity_force(attractor_mass: &Mass, attractor_transform: &Transform, attractee_mass: &Mass, attractee_transform: &Transform) -> Vec2 {
    let pos_attractor = attractor_transform.translation.xy();
    let pos_attractee = attractee_transform.translation.xy();

    let distance = pos_attractor.distance(pos_attractee);
    let f = calc_gravity_force_magnitude(attractor_mass.0, attractee_mass.0, distance);

    (pos_attractor - pos_attractee).clamp_length(f, f)
}

fn calc_gravity_force_magnitude(m1: f32, m2: f32, r: f32) -> f32 {
    let G: f32 = 6.674 * (10.0f32.powi(-11));
    G * ((m1 * m2) / r.powi(2))
}
