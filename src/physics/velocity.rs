use bevy::prelude::*;

#[derive(Component, Debug, Copy, Clone, PartialEq, Default)]
pub struct Velocity(pub Vec2);

pub(super) fn apply_velocity(mut query: Query<(&Velocity, &mut Transform)>, time: Res<Time>) {
    query.iter_mut().for_each(|(i_velocity, mut i_trans)| {
        i_trans.translation += calc_position_change(i_velocity, time.delta_secs()).extend(0.0);
    });
}

pub fn calc_position_change(velocity: &Velocity, time_delta: f32) -> Vec2 {
    velocity.0 * time_delta
}

pub(super) fn draw_velocities(mut gizmos: Gizmos, query: Query<(&Velocity, &Transform)>, time: Res<Time<Fixed>>) {
    query.iter().for_each(|(i_velocity, i_trans)| {
        if i_velocity.0 == Vec2::ZERO {
            return;
        }

        let color = Color::srgb_u8(0, 100, 255);
        gizmos.arrow_2d(
            i_trans.translation.xy(),
            i_trans.translation.xy() + (i_velocity.0 * time.timestep().as_secs_f32() * 100.0),
            color
        );
    });
}
