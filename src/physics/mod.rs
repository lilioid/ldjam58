pub(crate) mod calc_gravity;
pub(crate) mod directional_forces;
pub(crate) mod velocity;

use crate::dev_tools::is_debug_enabled;
use crate::physics::calc_gravity::{Attractee, Attractor};
use crate::physics::directional_forces::{
    GravityForce, Mass, ThrustForce, draw_directional_forces,
};
use crate::physics::velocity::Velocity;
use crate::{AppSystems, PausableSystems};
use bevy::color::palettes::basic::{GRAY, YELLOW};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            calc_gravity::calc_gravity,
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

    app.add_systems(Startup, debug_init_system);
    app.add_systems(Update, (draw_attractee).in_set(AppSystems::Update));
    app.add_systems(
        PostUpdate,
        (velocity::draw_velocities.run_if(is_debug_enabled)),
    );
}

fn debug_init_system(mut commands: Commands) {
    debug!("Adding orbiting satellite");
    /*
    commands.spawn((
        Attractee,
        GravityForce::default(),
        ThrustForce::default(),
        Velocity(Vec2::new(0.0, 0.2)),
        Mass(1.0),
        Transform::from_translation(Vec3::new(50.0, 0.0, 0.0)),
    ));

    commands.spawn((
        Attractee,
        GravityForce::default(),
        ThrustForce::default(),
        Velocity(Vec2::new(0.0, -0.1)),
        Mass(1.0),
        Transform::from_translation(Vec3::new(120.0, 30.0, 0.0)),
    ));
     */
}

fn draw_attractee(
    mut gizmos: Gizmos,
    query: Query<&Transform, (With<Attractee>, Without<Attractor>, Without<Sprite>)>,
) {
    query.iter().for_each(|(i_trans)| {
        let isometry = Isometry2d::new(i_trans.translation.xy(), Rot2::default());
        let color = Color::WHITE;
        gizmos.circle_2d(isometry, 5.0, color);
    });
}
