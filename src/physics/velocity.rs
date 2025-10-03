use bevy::prelude::*;

#[derive(Component, Debug, PartialEq)]
pub struct Velocity(pub Vec2);

pub(super) fn apply_velocity(query: Query<&Velocity>) {
    query.iter().for_each(|components| {
        debug!("velocity of entity is {components:?}");
    });
}
