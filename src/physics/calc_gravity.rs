use bevy::ecs::query::QuerySingleError;
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
    let pos1 = attractor_transform.translation.xy();
    let pos2 = attractee_transform.translation.xy();

    let distance = pos1.distance(pos2);
    let direction_angle = pos1 - pos2;

    let f = calc_gravity_force_magnitude(attractor_mass.0, attractee_mass.0, distance);
    (Vec2::X * f).rotate(direction_angle)
}

fn calc_gravity_force_magnitude(m1: f32, m2: f32, r: f32) -> f32 {
    let G: f32 = 6.674 * 10.0f32.powi(-11);
    const MUL: f32 = 10000.0;
    MUL * G * ((m1 * m2) / r.powi(2))
}
