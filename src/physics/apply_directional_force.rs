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
            gravity.0 = Vec2::ZERO;
        }
        if let Some(mut thrust) = thrust {
            accumulated_forces += thrust.0;
            thrust.0 = Vec2::ZERO;
        }
        
        let acceleration: Vec2 = accumulated_forces / mass.0;
        velocity.0 += acceleration * time.delta_secs();
    })
}
