use bevy::color::palettes::basic::BLUE;
use bevy::prelude::*;
use crate::dev_tools::is_debug_enabled;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, (check_for_collisions));
    app.add_systems(Update, (draw_hitboxes).run_if(is_debug_enabled) );
}

#[derive(Component, Copy, Clone, Debug, PartialEq, Default)]
pub struct HitBox {
    pub radius: f32,
}

pub fn is_colliding(obj1_transform: &Transform, obj1_hitbox: &HitBox,obj2_transform: &Transform, obj2_hitbox: &HitBox) -> bool {
    let distance = obj1_transform.translation.distance(obj2_transform.translation);
    if distance < (obj1_hitbox.radius+obj2_hitbox.radius) {
        return true;
    }
    //no collision
    false
}

fn check_for_collisions(hitboxes: Query<(Entity, &Transform, &HitBox)>) {
    for (entity, entity_transform, hitbox1) in hitboxes.iter() {
        for (entity_check, check_transform, hitbox2) in hitboxes.iter() {
            if (entity == entity_check){
                continue;
            }
            let distance = entity_transform.translation.distance(check_transform.translation);
            if distance < (hitbox1.radius+hitbox2.radius) {
                //info!("Collision found");
                //
            }
        }
    }
}

fn draw_hitboxes(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &HitBox)>,
) {
    query.iter().for_each(|(i_trans, i_hitbox)| {
        let isometry = Isometry2d::new(i_trans.translation.xy(), Rot2::default());
        let color = BLUE;
        gizmos.circle_2d(isometry, i_hitbox.radius, color);
    });
}
