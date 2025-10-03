use bevy::prelude::*;
use crate::physics::velocity::Velocity;

#[derive(Component, Debug, PartialEq, PartialOrd)]
pub struct Mass(pub f32);

trait Force {
    fn get_force(&self) -> &Vec2;
}

#[derive(Component, Debug, PartialEq)]
pub struct GravityForce(pub Vec2);

pub(super) fn apply_directional_force(mut query: Query<(&mut GravityForce, &mut Velocity, &Mass)>, time: Res<Time<Fixed>>) {
    query.iter_mut().for_each(|(mut gravity, mut velocity, mass)| {
        debug!("Applying directional force to entity with components {gravity:?} {velocity:?} {mass:?}");
        let acceleration: Vec2 = gravity.0 / mass.0;
        velocity.0 += acceleration * time.delta_secs();
        gravity.0 = Vec2::default();
    })
}
