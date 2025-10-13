use crate::dev_tools::is_debug_enabled;
use crate::physics::calc_gravity::{Attractee, Attractor};
use crate::sun_system::{Level, Satellite, SolarSystemAssets};
use crate::{AppSystems, GameplaySystem};
use bevy::color::palettes::basic::BLUE;
use bevy::prelude::*;
use std::collections::HashSet;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (check_for_collisions, draw_hitboxes.run_if(is_debug_enabled))
            .in_set(AppSystems::Update)
            .in_set(GameplaySystem),
    );
    app.add_observer(handle_fatal_collision_event);
    app.add_observer(handle_demote_collision_event);
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

#[derive(Event)]
pub struct GeneralCollisionEvent {
    pub destroyed: Entity,
    pub other: Entity,
}

#[derive(Event)]
pub struct DemoteCollisionEvent {
    pub demoted: Entity,
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
    hitboxes: Query<(Entity, &Transform, &HitBox, Has<Attractor>, Has<Attractee>, &Level)>,
) {
    // Track entities we already decided to destroy this system run to avoid duplicate events
    let mut destroyed_in_this_system: HashSet<Entity> = HashSet::new();
    for (entity, entity_transform, hitbox1, is_attractor, is_attractee, level1) in hitboxes.iter() {
        for (entity_check, check_transform, hitbox2, is_attractor2, is_attractee2, level2) in hitboxes.iter()
        {
            if (entity == entity_check) {
                // no need to check collisions with self
                continue;
            }
            // Skip pairs where either entity is already scheduled to be destroyed in this pass
            if destroyed_in_this_system.contains(&entity) || destroyed_in_this_system.contains(&entity_check) {
                continue;
            }
            let distance = entity_transform
                .translation
                .distance(check_transform.translation);
            if distance < (hitbox1.radius + hitbox2.radius) {
                info!("crash");

                if(is_attractor){
                    info!("crash sun case");
                    // first sun, sun has level 0
                    if !destroyed_in_this_system.contains(&entity_check) {
                        commands.trigger(FatalCollisionEvent {
                            destroyed: entity_check,
                            other: entity,
                        });
                        destroyed_in_this_system.insert(entity_check);
                    }
                } else if (is_attractor2) {
                } else {
                    info!("crash Satellites");

                    // satellite 1
                    if(level1.level == 1.) {
                        if !destroyed_in_this_system.contains(&entity) {
                            commands.trigger(FatalCollisionEvent {
                                destroyed: entity,
                                other: entity_check,
                            });
                            destroyed_in_this_system.insert(entity);
                        }
                    }else {
                        commands.trigger(DemoteCollisionEvent {
                            demoted: entity,
                            other: entity_check,
                        });
                    }
                    // satellite 2
                    if level2.level == 1. {
                        if !destroyed_in_this_system.contains(&entity_check) {
                            commands.trigger(FatalCollisionEvent {
                                destroyed: entity_check,
                                other: entity,
                            });
                            destroyed_in_this_system.insert(entity_check);
                        }
                    }else {
                        commands.trigger(DemoteCollisionEvent {
                            demoted: entity_check,
                            other: entity,
                        });

                    }
                }
            }
        }
    }
}

fn handle_demote_collision_event(event: On<DemoteCollisionEvent>, mut commands: Commands, mut collector_query: Query<(Entity, &mut Level, &mut Sprite),  (With<Attractee>)>, assets: Res<SolarSystemAssets>) {
    let demoted_entity= commands
        .get_entity(event.demoted)
        .expect("Wanted to demote entity after collision but entity does not exist!") ;
    for (entity, mut level, mut sprite) in collector_query.iter_mut() {
        if demoted_entity.id() == entity {
            if (level.level > 1.) {
                level.level -= 1.;
            }
            if level.level == 1. {
                //frm 2 to 1
                *sprite = Sprite::from(assets.collector.clone());
            } else if(level.level == 2.){
                //reduce from lv3 to 2
               *sprite =  Sprite::from(assets.collector2.clone());
            } else{
                *sprite =  Sprite::from(assets.collector3.clone());
            }
        }
    }
    // todo Get sprite and replace it with the correct level sprite
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
