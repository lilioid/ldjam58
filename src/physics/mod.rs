pub(crate) mod apply_directional_force;
pub(crate) mod calc_gravity;
pub(crate) mod velocity;

use bevy::color::palettes::basic::{GRAY, YELLOW};
use crate::physics::apply_directional_force::{GravityForce, Mass};
use crate::physics::velocity::Velocity;
use crate::{AppSystems, PausableSystems};
use bevy::prelude::*;
use crate::physics::calc_gravity::{Attractee, Attractor};
use crate::sun_system::init_sun_system;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            calc_gravity::calc_gravity.in_set(AppSystems::CalcPhysics),
            apply_directional_force::apply_directional_force.in_set(AppSystems::ApplyPhysics),
            velocity::apply_velocity
                .in_set(AppSystems::ApplyPhysics)
                .in_set(AppSystems::ApplyPhysics)
                .after(apply_directional_force::apply_directional_force),
        )
            .in_set(PausableSystems),
    );

    // app.add_systems(
    //     Update,
    //     (draw_attractee).in_set(AppSystems::Update)
    // );
}



// fn draw_attractor(mut gizmos: Gizmos, query: Query<(&Transform), (With<Attractor>, Without<Attractee>)>) {
//     query.iter().for_each(|(i_trans)| {
//         let isometry = Isometry2d::new(i_trans.translation.xy(), Rot2::default());
//         let color = Color::srgb(1.0, 0.5, 0.0);
//         gizmos.circle_2d(isometry, 10.0, color);
//     });
// }
//
// fn draw_attractee(mut gizmos: Gizmos, query: Query<&Transform, (With<Attractee>, Without<Attractor>)>) {
//     query.iter().for_each(|(i_trans)| {
//         let isometry = Isometry2d::new(i_trans.translation.xy(), Rot2::default());
//         let color = Color::WHITE;
//         gizmos.circle_2d(isometry, 5.0, color);
//     });
// }
