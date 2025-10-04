use bevy::prelude::*;
use crate::physics::velocity::Velocity;

#[derive(Component, Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct Mass(pub f32);

#[derive(Component, Debug, Copy, Clone, PartialEq, Default)]
pub struct GravityForce(pub Vec2);

#[derive(Component, Debug, Copy, Clone, PartialEq, Default)]
pub struct ThrustForce(pub Vec2);

pub(super) fn apply_directional_force(mut query: Query<(Option<&GravityForce>, Option<&ThrustForce>, &mut Velocity, &Mass)>, time: Res<Time>) {
    query.iter_mut().for_each(|(gravity, thrust, mut velocity, mass)| {
        let mut accumulated_forces = Vec2::ZERO;
        
        if let Some(gravity) = gravity {
            accumulated_forces += gravity.0;
        }
        if let Some(thrust) = thrust {
            accumulated_forces += thrust.0;
        }

        velocity.0 += calc_velocity_change(accumulated_forces, mass, time.delta_secs());
    })
}

pub fn calc_velocity_change(forces: Vec2, mass: &Mass, time_delta: f32) -> Vec2 {
    let acceleration = forces / mass.0 as f32;
    acceleration * time_delta as f32
}

pub(super) fn clear_forces(mut gravity: Query<&mut GravityForce>, mut thrust: Query<&mut ThrustForce>) {
    gravity.iter_mut().for_each(|mut i_gravity| {
        i_gravity.0 = Vec2::ZERO;
    });
    thrust.iter_mut().for_each(|mut i_thrust| {
        i_thrust.0 = Vec2::ZERO;
    })
}

pub(super) fn draw_directional_forces(mut gizmos: Gizmos, gravity: Query<(&GravityForce, &Transform)>, thrust: Query<(&ThrustForce, &Transform)>, time: Res<Time<Fixed>>) {
    gravity.iter().for_each(|(i_gravity, i_trans)| {
        draw_force_arrow(&mut gizmos, i_gravity.0, i_trans.translation.xy(), &time);
    });
    thrust.iter().for_each(|(i_thrust, i_trans)| {
        draw_force_arrow(&mut gizmos, i_thrust.0, i_trans.translation.xy(), &time);
    })
}

fn draw_force_arrow(gizmos: &mut Gizmos, force: Vec2, at: Vec2, time: &Time<Fixed>) {
    if force == Vec2::ZERO {
        return;
    }
    
    let color = Color::srgb_u8(255, 0, 150);
    gizmos.arrow_2d(
        at,
        at + (force * time.timestep().as_secs_f32() * 250.0),
        color
    );
}
