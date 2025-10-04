use bevy::color::palettes::basic::GRAY;
use crate::physics::calc_gravity::{calc_gravity_force, Attractee, Attractor};
use crate::physics::directional_forces::{calc_velocity_change, Mass};
use crate::physics::velocity::{calc_position_change, Velocity};
use bevy::prelude::*;

const PROJECTION_DELTA: f32 = 0.25;
const PROJECTION_COUNT: usize = 100;

#[derive(Component, Debug, Default, Copy, Clone)]
#[require(Transform, Velocity, Mass)]
pub struct NavigationInstruments;

pub fn draw_nav_projections(
    mut gizmos: Gizmos,
    attractor: Query<(&Transform, &Mass), With<Attractor>>,
    query: Query<(&Transform, &Mass, &Velocity), (With<NavigationInstruments>, With<Attractee>)>,
) {
    let (attractor_trans, attractor_mass) = attractor
        .single()
        .expect("Cannot draw orbital projections if there is no attractor in the world");

    query.iter().for_each(|(i_trans, i_mass, i_velocity)| {
        draw_orbit_projection(&mut gizmos, attractor_trans, attractor_mass, i_trans, i_mass, i_velocity)
    });
}

fn draw_orbit_projection(
    gizmos: &mut Gizmos,
    attractor_trans: &Transform,
    attractor_mass: &Mass,
    transform: &Transform,
    mass: &Mass,
    velocity: &Velocity,
) {
    let mut projected_trans = *transform;
    let mut projected_velocity = *velocity;
    
    for _ in 0..PROJECTION_COUNT {        
        let grav_force = calc_gravity_force(attractor_mass, attractor_trans, mass, &projected_trans);
        projected_velocity.0 += calc_velocity_change(grav_force, mass, PROJECTION_DELTA);
        projected_trans.translation += calc_position_change(&projected_velocity, PROJECTION_DELTA).extend(0.0);
        
        gizmos.cross_2d(Isometry2d::from_translation(projected_trans.translation.xy()), 1.0, GRAY);
    }
}
