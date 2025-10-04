pub(crate) mod apply_directional_force;
pub(crate) mod calc_gravity;
pub(crate) mod velocity;

use crate::physics::apply_directional_force::{GravityForce, Mass};
use crate::physics::calc_gravity::{Attractee, Attractor};
use crate::physics::velocity::Velocity;
use crate::sun_system::init_sun_system;
use crate::{AppSystems, PausableSystems};
use bevy::color::palettes::basic::{GRAY, YELLOW};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            calc_gravity::calc_gravity,
            apply_directional_force::apply_directional_force,
            velocity::apply_velocity,
        )
            .chain()
            .in_set(PausableSystems)
            .in_set(AppSystems::Physics),
    );

    app.add_systems(Startup, debug_init_system);
    app.add_systems(
        Update,
        (draw_attractor, draw_attractee).in_set(AppSystems::Update)(draw_attractor, draw_attractee)
            .in_set(AppSystems::Update),
    );
}

fn debug_init_system(mut commands: Commands) {
    debug!("Adding sun");
    commands.spawn((
        Attractor,
        Mass(100000000000.0),
        Transform::from_translation(Vec3::splat(0.0)),
    ));

    debug!("Adding orbiting satellite");
    commands.spawn((
        Attractee,
        GravityForce(Vec2::new(1.0, 0.0)),
        Velocity(Vec2::new(0.0, 0.0)),
        Mass(1.0),
        Transform::from_translation(Vec3::new(50.0, 0.0, 0.0)),
    ));
}

fn draw_attractor(
    mut gizmos: Gizmos,
    query: Query<(&Transform), (With<Attractor>, Without<Attractee>)>,
) {
    query.iter().for_each(|(i_trans)| {
        let isometry = Isometry2d::new(i_trans.translation.xy(), Rot2::default());
        let color = Color::srgb(1.0, 0.5, 0.0);
        gizmos.circle_2d(isometry, 10.0, color);
    });
}

fn draw_attractee(
    mut gizmos: Gizmos,
    query: Query<&Transform, (With<Attractee>, Without<Attractor>)>,
) {
    fn draw_attractee(
        mut gizmos: Gizmos,
        query: Query<&Transform, (With<Attractee>, Without<Attractor>)>,
    ) {
        query.iter().for_each(|(i_trans)| {
            let isometry = Isometry2d::new(i_trans.translation.xy(), Rot2::default());
            let color = Color::WHITE;
            gizmos.circle_2d(isometry, 5.0, color);
        });
    }
}
