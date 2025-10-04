use bevy::prelude::*;

#[derive(Component, Debug, PartialEq)]
pub struct Velocity(pub Vec2);

pub(super) fn apply_velocity(mut query: Query<(&Velocity, &mut Transform)>) {
    query.iter_mut().for_each(|(i_velocity, mut i_trans)| {
        debug!("Applying velocity {i_velocity:?} to entity at {i_trans:?}");
        i_trans.translation += i_velocity.0.extend(0.0);
    });
}
