pub(crate) mod calc_gravity;
pub(crate) mod directional_forces;
pub(crate) mod velocity;

use crate::dev_tools::is_debug_enabled;
use crate::physics::directional_forces::draw_directional_forces;
use crate::physics::velocity::draw_velocities;
use crate::{AppSystems, GameplaySystem, PausableSystems};
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
            .in_set(GameplaySystem)
            .in_set(AppSystems::Physics),
    );

    app.add_systems(
        FixedPostUpdate,
        (
            draw_directional_forces
                .run_if(is_debug_enabled)
                .before(directional_forces::clear_forces),
            draw_velocities.run_if(is_debug_enabled),
            directional_forces::clear_forces.in_set(AppSystems::Physics),
        )
            .in_set(GameplaySystem),
    );
}
