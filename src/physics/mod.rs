pub(crate) mod calc_gravity;
pub(crate) mod directional_forces;
pub(crate) mod velocity;

use crate::dev_tools::is_debug_enabled;
use crate::physics::calc_gravity::{Attractee, Attractor};
use crate::physics::directional_forces::draw_directional_forces;
use crate::{AppSystems, PausableSystems};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            calc_gravity::apply_gravity,
            directional_forces::apply_directional_force,
            velocity::apply_velocity,
        )
            .chain()
            .in_set(PausableSystems)
            .in_set(AppSystems::Physics),
    );

    app.add_systems(
        FixedPostUpdate,
        (
            draw_directional_forces
                .run_if(is_debug_enabled)
                .before(directional_forces::clear_forces),
            directional_forces::clear_forces.in_set(AppSystems::Physics),
        ),
    );

    app.add_systems(Update, (draw_attractee).in_set(AppSystems::Update));
    app.add_systems(
        PostUpdate,
        velocity::draw_velocities.run_if(is_debug_enabled),
    );
}

fn draw_attractee(
    mut gizmos: Gizmos,
    query: Query<&Transform, (With<Attractee>, Without<Attractor>, Without<Sprite>)>,
) {
    query.iter().for_each(|i_trans| {
        let isometry = Isometry2d::new(i_trans.translation.xy(), Rot2::default());
        let color = Color::WHITE;
        gizmos.circle_2d(isometry, 5.0, color);
    });
}
