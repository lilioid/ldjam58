use crate::asset_tracking::LoadResource;
use crate::dev_tools::is_debug_enabled;
use crate::physics::velocity::Velocity;
use crate::{AppSystems, GameplaySystem, RandomSource};
use bevy::color::palettes::basic::GREEN;
use bevy::prelude::*;
use rand::Rng;
use std::f32::consts::PI;
use std::ops::Range;
use std::time::Duration;

pub fn plugin(app: &mut App) {
    app.load_resource::<AsteroidAssets>();
    app.init_resource::<AsteroidConfig>();
    app.init_resource::<AsteroidTracker>();
    app.add_systems(
        Update,
        (asteroid_spawning_system)
            .in_set(GameplaySystem)
            .in_set(AppSystems::Update),
    );
    app.add_systems(PostUpdate, (draw_swarm_debug, draw_asteroid_debug).run_if(is_debug_enabled));
}

#[derive(Resource, Asset, Reflect, Debug, Clone)]
#[reflect(Resource)]
struct AsteroidAssets {
    asteroid: Handle<Image>,
}

impl FromWorld for AsteroidAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            asteroid: assets.load("asteroid.png"),
        }
    }
}

#[derive(Resource, Debug, PartialEq)]
pub struct AsteroidConfig {
    /// chance (evaluated per frame) that an asteroid swarm will spawn, expressed as 1 / $this
    pub spawn_chance: usize,
    /// Minimum time between asteroid swarms in seconds
    pub min_time_between: usize,
    /// Minimum time that the game should be running before the first swarm appears
    pub min_initial_wait: usize,
    /// A range of how many asteroids should be spawned
    asteroid_gen_range: Range<usize>,
    /// Radius in which asteroids are clustered
    cluster_radius: f32,
    /// Minimum distance between asteroids in a cluster
    min_distance: f32,
    /// Maximum attempts to place an asteroid in a cluster without violating the min_distance constraint
    max_attempts: usize,
}

impl Default for AsteroidConfig {
    fn default() -> Self {
        Self {
            spawn_chance: 100,
            min_time_between: 5,
            min_initial_wait: 1,
            asteroid_gen_range: 2..6,
            cluster_radius: 30.0,
            min_distance: 10.0,
            max_attempts: 10,
        }
    }
}

/// Helper for tracking state between asteroid system executions
#[derive(Resource, Debug, Eq, PartialEq)]
struct AsteroidTracker {
    start_timer: Timer,
    spawn_backoff_timer: Timer,
}

impl FromWorld for AsteroidTracker {
    fn from_world(world: &mut World) -> Self {
        let cfg = world.resource::<AsteroidConfig>();
        Self {
            start_timer: Timer::new(
                Duration::from_secs(cfg.min_initial_wait as u64),
                TimerMode::Once,
            ),
            spawn_backoff_timer: Timer::new(
                Duration::from_secs(cfg.min_time_between as u64),
                TimerMode::Once,
            ),
        }
    }
}

/// Marker component to mark an asteroid swarm entity.
/// It should have asteroids as children.
#[derive(Component, Debug, Eq, PartialEq, Hash)]
#[require(Transform)]
pub struct AsteroidSwarm;

/// Marker component to mark asteroids
#[derive(Component, Debug, Eq, PartialEq, Hash)]
#[require(Transform, Sprite)]
pub struct Asteroid;

fn asteroid_spawning_system(
    mut commands: Commands,
    assets: Res<AsteroidAssets>,
    cfg: Res<AsteroidConfig>,
    mut randomness: ResMut<RandomSource>,
    mut tracker: ResMut<AsteroidTracker>,
    time: Res<Time>,
) {
    tracker.start_timer.tick(time.delta());
    tracker.spawn_backoff_timer.tick(time.delta());

    // don't execute the remaining system if gameplay has not been running for the configured amount of time
    if !tracker.start_timer.is_finished() {
        return;
    }
    if tracker.start_timer.just_finished() {
        info!("Grace period has expired and asteroids can spawn now");
        tracker.spawn_backoff_timer.finish();
    }

    // don't try to spawn anything if we've just done so (the backoff timer is running)
    if !tracker.spawn_backoff_timer.is_finished() {
        return;
    }

    // if the backoff has been reached, spawn something if randomness lets us
    if randomness.random_ratio(1, cfg.spawn_chance as u32) {
        tracker.spawn_backoff_timer.reset();
        spawn_asteroids(&mut commands, &cfg, &assets, &mut randomness);
    }
}

fn spawn_asteroids(
    commands: &mut Commands,
    cfg: &AsteroidConfig,
    assets: &AsteroidAssets,
    random: &mut RandomSource,
) {
    let num_asteroids = random.random_range(cfg.asteroid_gen_range.clone());
    let direction = random.random_range(-45..45) as f32 * PI / 180.0;
    let speed = random.random_range(10..20) as f32;
    info!("Spawning asteroid swarm with {num_asteroids} asteroids");

    let swarm = commands
        .spawn((
            AsteroidSwarm,
            Transform::from_translation(Vec3::new(0.0, -75.0, 0.0))
                .with_rotation(Quat::from_axis_angle(Vec3::Z, direction)),
            InheritedVisibility::default(),
            Velocity(Vec2::from_angle(direction + 0.5 * PI) * speed),
        ))
        .id();


    let mut positions = Vec::new();

    for _ in 0..num_asteroids {
        let mut position = None;

        for _ in 0..cfg.max_attempts {
            let x = random.random_range(-cfg.cluster_radius as i32..cfg.cluster_radius as i32) as f32;
            let y = random.random_range(-cfg.cluster_radius as i32..cfg.cluster_radius as i32) as f32;
            let candidate = Vec2::new(x, y);

            if positions.iter().all(|&pos: &Vec2| pos.distance(candidate) >= cfg.min_distance) {
                position = Some(candidate);
                break;
            }
        }

        if let Some(pos) = position {
            positions.push(pos);

            commands.spawn((
                Asteroid,
                ChildOf(swarm),
                Transform::from_translation(Vec3::new(pos.x, pos.y, 0.0))
                    .with_scale(Vec3::splat(0.01))
                    .with_rotation(Quat::from_axis_angle(Vec3::X, PI)),
                Sprite::from(assets.asteroid.clone()),
            ));
        }
    }
}

fn draw_swarm_debug(mut gizmos: Gizmos, query: Query<&GlobalTransform, With<AsteroidSwarm>>) {
    query.iter().for_each(|(i_trans)| {
        let isometry = Isometry2d::from_translation(i_trans.translation().xy());
        gizmos.circle_2d(isometry, 4.0, GREEN);
    });
}

fn draw_asteroid_debug(mut gizmos: Gizmos, query: Query<&GlobalTransform, With<Asteroid>>) {
    query.iter().for_each(|(i_trans)| {
        let isometry = Isometry2d::from_translation(i_trans.translation().xy());
        gizmos.circle_2d(isometry, 2.0, GREEN);
    });
}
