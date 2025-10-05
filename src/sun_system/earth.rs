use crate::asset_tracking::LoadResource;
use crate::screens::Screen;
use bevy::prelude::*;
use crate::GameplaySystem;
use crate::launching::{make_launchpad, LaunchPad};
use crate::sun_system::Sun;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<EarthAssets>();
    app.add_systems(OnEnter(Screen::Gameplay), init_earth);
    app.add_systems(Update, move_earth_around_sun.in_set(GameplaySystem));
    app.add_systems(Update, draw_arrow.in_set(GameplaySystem));
}

#[derive(Resource, Asset, Reflect, Debug, Clone)]
#[reflect(Resource)]
pub struct EarthAssets {
    #[dependency]
    earth: Handle<Image>,
}

impl FromWorld for EarthAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            earth: assets.load("planet.png"),
        }
    }
}

/// A marker component for the players home planet
#[derive(Component)]
#[require(Transform)]
pub struct Earth;

fn init_earth(mut commands: Commands, assets: Res<EarthAssets>) {
    info!("Init earth");

    commands.spawn((
        Name::new("Earth"),
        Earth,
        Transform::from_translation(Vec3::new(100.0, 0.0, 0.0)).with_scale(Vec3::splat(0.004)),
        Sprite::from(assets.earth.clone()),
        children![ 
            make_launchpad(),
        ]
    ));
}

fn move_earth_around_sun(
    mut earth_query: Query<&mut Transform, With<Earth>>,
    sun_query: Query<&Transform, (With<Sun>, Without<Earth>)>,
    mut launch_pad_query: Query<&mut Transform, (With<LaunchPad>, Without<Earth>, Without<Sun>)>,
    time: Res<Time>
) {
    let sun_transform = sun_query.single();
    let sun_position = sun_transform.unwrap().translation;

    for mut earth_transform in earth_query.iter_mut() {
        let angle_speed = 0.1;
        let radius = earth_transform.translation.distance(sun_position);
        let angle = time.elapsed_secs() * angle_speed;

        let new_x = sun_position.x + radius * angle.cos();
        let new_y = sun_position.y + radius * angle.sin();
        earth_transform.translation = Vec3::new(new_x, new_y, earth_transform.translation.z);
        // FUCK, I DONT KNOW WHY BUT TRANSFORM PROPAGATION IS BROKEN :(
        let mut launch_pad_transform = launch_pad_query.single_mut().unwrap();
        launch_pad_transform.translation = earth_transform.translation;
    }
}

fn draw_arrow(
    mut gizmos: Gizmos,
    earth_query: Query<&Transform, With<Earth>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
) {
    let earth_transform = earth_query.single().unwrap();
    let (camera, camera_transform) = camera_query.single().unwrap();
    let window = windows.iter().next().unwrap();

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    let Ok(mouse_world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    let earth_pos = earth_transform.translation.truncate();
    let direction = (mouse_world_pos - earth_pos).normalize_or_zero();

    let max_length = 100.0;
    let actual_distance = earth_pos.distance(mouse_world_pos);
    let arrow_length = actual_distance.min(max_length);

    let arrow_end = earth_pos + direction * arrow_length;

    gizmos.arrow_2d(earth_pos, arrow_end, Color::WHITE);
}