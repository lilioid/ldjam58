use crate::collision::{HitBox, is_colliding};
use crate::physics::calc_gravity::{Attractee, Attractor, calc_gravity_force};
use crate::physics::directional_forces::{Mass, calc_velocity_change};
use crate::physics::velocity::{Velocity, calc_position_change};
use bevy::color::palettes::basic::GRAY;
use bevy::prelude::*;
use crate::achievements::{FullOrbitAchieved, FullOrbitAwarded};
use std::f32::consts::PI;

const PROJECTION_DELTA: f32 = 0.5;
const PROJECTION_MAX_COUNT: usize = 250;

#[derive(Component, Debug, Default, Copy, Clone)]
#[require(Transform, Velocity, Mass, HitBox)]
pub struct NavigationInstruments;

pub fn draw_nav_projections(
    mut gizmos: Gizmos,
    attractor: Query<(&Transform, &Mass, &HitBox), With<Attractor>>,
    query: Query<(Entity, &Transform, &Mass, &Velocity, &HitBox, Option<&FullOrbitAwarded>), (With<NavigationInstruments>, With<Attractee>)>,
    mut commands: Commands,
) {
    let Some((attractor_trans, attractor_mass, attractor_hitbox)) = attractor.iter().next() else { return; };

    query.iter().for_each(|(entity, i_trans, i_mass, i_velocity, i_hitbox, awarded)| {
        draw_orbit_projection(
            &mut gizmos,
            attractor_trans,
            attractor_mass,
            attractor_hitbox,
            i_trans,
            i_mass,
            i_velocity,
            i_hitbox,
            &mut commands,
            entity,
            awarded.is_some(),
        )
    });
}

fn draw_orbit_projection(
    gizmos: &mut Gizmos,
    attractor_trans: &Transform,
    attractor_mass: &Mass,
    attractor_hitbox: &HitBox,
    transform: &Transform,
    mass: &Mass,
    velocity: &Velocity,
    hitbox: &HitBox,
    commands: &mut Commands,
    entity: Entity,
    already_awarded: bool,
) {
    let mut degrees_covered = 0.0;

    let mut projected_trans = *transform;
    let mut projected_velocity = *velocity;

    for _ in 0..PROJECTION_MAX_COUNT {
        let last_trans = projected_trans.translation.xy();

        let grav_force =
            calc_gravity_force(attractor_mass, attractor_trans, mass, &projected_trans);
        projected_velocity.0 += calc_velocity_change(grav_force, mass, PROJECTION_DELTA);
        projected_trans.translation +=
            calc_position_change(&projected_velocity, PROJECTION_DELTA).extend(0.0);

        // don't draw any more gizmos if they start colliding with the sun
        if is_colliding(
            attractor_trans,
            attractor_hitbox,
            &projected_trans,
            hitbox,
        ) {
            break
        }

        // draw the projection gizmo
        gizmos.cross_2d(
            Isometry2d::from_translation(projected_trans.translation.xy()),
            1.0,
            GRAY,
        );

        // don't draw any more gizmos if we have covered 360°
        degrees_covered += last_trans.angle_to(projected_trans.translation.xy()) * 180.0 / PI;
        if degrees_covered >= 355.0 {
            if !already_awarded {
                // Trigger achievement once per satellite
                commands.entity(entity).insert(FullOrbitAwarded);
                commands.trigger(FullOrbitAchieved { entity });
            }
            break;
        }
    }
}
