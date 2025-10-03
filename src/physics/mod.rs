mod apply_directional_force;
mod calc_gravity;
mod velocity;

use crate::physics::apply_directional_force::{GravityForce, Mass};
use crate::{AppSystems, PausableSystems};
use bevy::prelude::*;
use crate::physics::velocity::Velocity;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            calc_gravity::calc_gravity.in_set(AppSystems::CalcPhysics),
            apply_directional_force::apply_directional_force.in_set(AppSystems::ApplyPhysics),
            velocity::apply_velocity
                .in_set(AppSystems::ApplyPhysics)
                .after(apply_directional_force::apply_directional_force),
        )
            .in_set(PausableSystems),
    );

    app.add_systems(Startup, debug_init_system);
    app.add_systems(
        Update,
        debug_force_system
            .in_set(AppSystems::CalcPhysics)
            .in_set(PausableSystems),
    );
}

fn debug_init_system(mut commands: Commands) {
    debug!("Adding basic entity with forces");
    commands.spawn((
        GravityForce(Vec2::new(1.0, 0.0)),
        Velocity(Vec2::new(0.0, 0.0)),
        Mass(1.0),
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
    ));
}

fn debug_force_system(mut commands: Commands) {}
