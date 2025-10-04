use bevy::prelude::*;
use crate::physics::velocity::Velocity;

#[derive(Component, Debug, PartialEq, PartialOrd)]
pub struct Mass(pub f32);

#[derive(Component, Debug, PartialEq, Default)]
pub struct GravityForce(pub Vec2);

#[derive(Component, Debug, PartialEq, Default)]
pub struct ThrustForce(pub Vec2);

pub(super) fn apply_directional_force(mut query: Query<(Option<&mut GravityForce>, Option<&mut ThrustForce>, &mut Velocity, &Mass)>, time: Res<Time<Fixed>>) {
    query.iter_mut().for_each(|(mut gravity, mut thrust, mut velocity, mass)| {
        let mut accumulated_forces = Vec2::ZERO;
        
        if let Some(mut gravity) = gravity {
            accumulated_forces += gravity.0;
        }
        if let Some(mut thrust) = thrust {
            accumulated_forces += thrust.0;
        }
        
        let acceleration: Vec2 = accumulated_forces / mass.0;
        velocity.0 += acceleration * time.delta_secs();
    })
}

pub(super) fn clear_forces(mut gravity: Query<(&mut GravityForce)>, mut thrust: Query<(&mut ThrustForce)>) {
    gravity.iter_mut().for_each(|(mut i_gravity)| {
        i_gravity.0 = Vec2::ZERO;
    });
    thrust.iter_mut().for_each(|(mut i_thrust)| {
        i_thrust.0 = Vec2::ZERO;
    })
}

pub(super) fn draw_directional_forces(mut gizmos: Gizmos, gravity: Query<(&GravityForce, &Transform)>, thrust: Query<(&ThrustForce, &Transform)>) {
    gravity.iter().for_each(|(i_gravity, i_trans)| {
        draw_force_arrow(&mut gizmos, i_gravity.0, i_trans.translation.xy());
    });
    thrust.iter().for_each(|(i_thrust, i_trans)| {
        draw_force_arrow(&mut gizmos, i_thrust.0, i_trans.translation.xy());
    })
}

fn draw_force_arrow(gizmos: &mut Gizmos, force: Vec2, at: Vec2) {
    if force == Vec2::ZERO {
        return;
    }
    
    let color = Color::srgb_u8(255, 0, 150);
    gizmos.arrow_2d(at, at + (force * 250.0), color);
}
