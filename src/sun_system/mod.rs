pub mod navigation_instruments;
pub mod thruster;

use crate::AppSystems;
use crate::asset_tracking::LoadResource;
use crate::physics::calc_gravity::{Attractee, Attractor};
use crate::physics::directional_forces::{GravityForce, Mass};
use crate::physics::velocity::Velocity;
use crate::screens::Screen;
use crate::sun_system::navigation_instruments::NavigationInstruments;
use crate::sun_system::thruster::{Thruster, ThrusterDirection};
use bevy::input::common_conditions::{input_just_pressed, input_just_released};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::collision::HitBox;

#[derive(Component)]
struct TiledGrid {
    cols: i32,
    rows: i32,
    tile_world_size: f32,
}

#[derive(Component)]
struct GridIndex {
    col: i32,
    row: i32,
}

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<SolarSystemAssets>();
    app.add_systems(
        FixedUpdate,
        (thruster::apply_thrust_force)
            .in_set(AppSystems::Physics)
            .run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(
        Update,
        thruster::toggle_thruster
            .run_if(
                input_just_pressed(thruster::THRUSTER_KEY)
                    .or(input_just_released(thruster::THRUSTER_KEY)),
            )
            .in_set(AppSystems::RecordInput),
    );
    app.add_systems(
        Update,
        navigation_instruments::draw_nav_projections
            .run_if(in_state(Screen::Gameplay))
            .in_set(AppSystems::Update),
    );
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct SolarSystemAssets {
    #[dependency]
    sun: Handle<Image>,
    #[dependency]
    pub(crate) collector: Handle<Image>,
    #[dependency]
    grid: Handle<Image>,
    #[dependency]
    pub(crate) bg: Handle<Image>,
}

#[derive(Component)]
pub struct Satellite;

#[derive(Component)]
pub struct Sun;


impl FromWorld for SolarSystemAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            sun: assets.load("sun.png"),
            grid: assets.load("retro_grid.png"),
            collector: assets.load("satellite.png"),
            bg: assets.load("retro_grid_bg.png"),
        }
    }
}

pub fn init_sun_system(mut commands: Commands, solar_system_assets: Res<SolarSystemAssets>) {
    info!("Adding sun");
    commands.spawn((
        Attractor,
        HitBox {
            radius: 20.0
        },
        Mass(100_000_000_000_000.0),
        Name::new("Sun"),
        Transform::from_translation(Vec3::ZERO).with_scale(Vec3::splat(0.02)),
        Sprite::from(solar_system_assets.sun.clone()),
        Sun
    ));
}

pub fn setup_tiled_grid(
    mut commands: Commands,
    solar_system_assets: Res<SolarSystemAssets>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    info!("Adding tiled grid");
    let Ok(win) = windows.single() else {
        return;
    };
    info!("after check grid");

    // World size per tile (match your grid cell size)
    let tile_world_size: f32 = 24.0;
    let margin_tiles: i32 = 2;

    let cols = ((win.width() / tile_world_size).ceil() as i32) + margin_tiles * 2 + 1;
    let rows = ((win.height() / tile_world_size).ceil() as i32) + margin_tiles * 2 + 1;

    let parent = commands
        .spawn((
            Name::new("RetroGrid"),
            Visibility::default(),
            TiledGrid {
                cols,
                rows,
                tile_world_size,
            },
            // Keep behind everything
            Transform::from_xyz(0.0, 0.0, -1.0),
            GlobalTransform::default(),
        ))
        .id();

    for row in 0..rows * rows {
        for col in 0..cols * cols {
            commands.entity(parent).with_children(|p| {
                p.spawn((
                    Transform::from_translation(Vec3::new(
                        (col * 18 - (win.width() / 2.0f32) as i32) as f32,
                        (row * 18 - (win.height() / 2.0f32) as i32) as f32,
                        0.0,
                    ))
                    .with_scale(Vec3::splat(0.015)),
                    Sprite::from(solar_system_assets.grid.clone()),
                ));
            });
        }
    }
}

pub fn setup_grid_image(mut commands: Commands, solar_system_assets: Res<SolarSystemAssets>) {
    commands.spawn((
        Name::new("GridImage"),
        Sprite::from(solar_system_assets.bg.clone()),
        Transform::from_translation(Vec3::new(0.0, 0.0, -5.0)).with_scale(Vec3::splat(0.15))
    ));
}