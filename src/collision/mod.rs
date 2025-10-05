use crate::dev_tools::is_debug_enabled;
use crate::physics::calc_gravity::{Attractee, Attractor};
use crate::sun_system::SolarSystemAssets;
use crate::{AppSystems, GameplaySystem};
use bevy::color::palettes::basic::BLUE;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (check_for_collisions, draw_hitboxes.run_if(is_debug_enabled))
            .in_set(AppSystems::Update)
            .in_set(GameplaySystem),
    );
    app.add_observer(handle_fatal_collision_event);
}

#[derive(Component, Copy, Clone, Debug, PartialEq, Default)]
pub struct HitBox {
    pub radius: f32,
}

#[derive(Event)]
pub struct FatalCollisionEvent {
    pub destroyed: Entity,
    pub other: Entity,
}

pub fn is_colliding(
    obj1_transform: &Transform,
    obj1_hitbox: &HitBox,
    obj2_transform: &Transform,
    obj2_hitbox: &HitBox,
) -> bool {
    let distance = obj1_transform
        .translation
        .distance(obj2_transform.translation);
    if distance < (obj1_hitbox.radius + obj2_hitbox.radius) {
        return true;
    }
    //no collision
    false
}

fn check_for_collisions(
    mut commands: Commands,
    hitboxes: Query<(Entity, &Transform, &HitBox, Has<Attractor>, Has<Attractee>)>,
    solarRes: Res<SolarSystemAssets>,
) {
    for (entity, entity_transform, hitbox1, isAttractor, isAttractee) in hitboxes.iter() {
        for (entity_check, check_transform, hitbox2, isAttractor2, isAttractee2) in hitboxes.iter()
        {
            if (entity == entity_check) {
                continue;
            }
            let distance = entity_transform
                .translation
                .distance(check_transform.translation);
            if distance < (hitbox1.radius + hitbox2.radius) {
                info!("Adding crash image");

                if(isAttractor){//attractor involved delete attracted
                    //delete attracted ( mark for cleanup system)
                    commands.trigger(FatalCollisionEvent {
                        destroyed: entity_check,
                        other: entity,
                    });
                } else if (isAttractor2) {
                } else {
                    info!("lala!");
                    commands.trigger(FatalCollisionEvent {
                        destroyed: entity,
                        other: entity_check,
                    });
                }
                //info!("Collision found");
                // if entity is not attractor delete
                // attracted and attracted ?
            }
        }
    }
}

fn handle_fatal_collision_event(event: On<FatalCollisionEvent>, mut commands: Commands) {
    commands
        .get_entity(event.destroyed)
        .expect("Wanted to despawn entity after fatal collision but entity does not exist!")
        .despawn();
}

/**
collectors have hp, output, level
lower hp decreases output
fornow collectors die when colliding with each other
**/

fn draw_hitboxes(mut gizmos: Gizmos, query: Query<(&Transform, &HitBox)>) {
    query.iter().for_each(|(i_trans, i_hitbox)| {
        let isometry = Isometry2d::new(i_trans.translation.xy(), Rot2::default());
        let color = BLUE;
        gizmos.circle_2d(isometry, i_hitbox.radius, color);
    });
}
