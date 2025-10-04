use bevy::prelude::*;

#[derive(Component, Debug, Copy, Clone, PartialEq, Default)]
pub struct Velocity(pub Vec2);

pub(super) fn apply_velocity(mut query: Query<(&Velocity, &mut Transform)>) {
    query.iter_mut().for_each(|(i_velocity, mut i_trans)| {
        i_trans.translation += i_velocity.0.extend(0.0);
    });
}

pub(super) fn draw_velocities(mut gizmos: Gizmos, query: Query<(&Velocity, &Transform)>) {
    query.iter().for_each(|(i_velocity, i_trans)| {
        if i_velocity.0 == Vec2::ZERO {
            return;
        }

        let color = Color::srgb_u8(0, 100, 255);
        gizmos.arrow_2d(i_trans.translation.xy(), i_trans.translation.xy() + (i_velocity.0 * 200.0), color);
    });
}
