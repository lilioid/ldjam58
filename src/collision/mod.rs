use bevy::color::palettes::basic::BLUE;
use bevy::prelude::*;
use crate::physics::calc_gravity::{Attractee, Attractor};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, (check_for_collisions));
}

#[derive(Component)]
pub struct HitBox {
    pub radius: f32,
}

fn check_for_collisions(hitboxes: Query<(Entity, &Transform, &HitBox)>) {
    for (entity, entity_transform, hitbox) in hitboxes.iter() {
        for (entity_check, check_transform, hitbox) in hitboxes.iter() {
            if (entity == entity_check){
                continue;
            }
            let distance = entity_transform.translation.distance(check_transform.translation);
            if distance < hitbox.radius {
                info!("Collision found");
            }
        }
    }
}

fn draw_hitboxes(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &HitBox)>,
) {
    query.iter().for_each(|((i_trans, i_hitbox))| {
        let isometry = Isometry2d::new(i_trans.translation.xy(), Rot2::default());
        let color = BLUE;
        gizmos.circle_2d(isometry, i_hitbox.radius, color);
    });
}
