// Support configuring Bevy lints within code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

mod asset_tracking;
mod collision;
#[cfg(feature = "dev")]
mod dev_tools;
mod hud;
mod launching;
mod physics;
mod score;
mod screens;
mod sun_system;
mod sound;

use std::ops::{Deref, DerefMut};
use crate::screens::Screen;
use bevy::log::LogPlugin;
use bevy::window::WindowResolution;
use bevy::{asset::AssetMetaCheck, prelude::*};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn main() -> AppExit {
    App::new().add_plugins(AppPlugin).run()
}

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Configure bevys default plugins
        app.add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics on web build on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "Type two".to_string(),
                        fit_canvas_to_parent: true,
                        //mode: WindowMode::Fullscreen(MonitorSelection::Primary, VideoModeSelection::Current), Laggy
                        resolution: WindowResolution::new(1024, 576),
                        ..default()
                    }
                    .into(),
                    ..default()
                })
                .set(LogPlugin {
                    filter: "info,ldjam58=debug".to_string(),
                    ..default()
                }),
        );
        app.insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)));

        // add our own plugins
        app.add_plugins((
            asset_tracking::plugin,
            physics::plugin,
            #[cfg(feature = "dev")]
            dev_tools::plugin,
            screens::plugin,
            sun_system::plugin,
            launching::plugin,
            collision::plugin,
            score::plugin,
            hud::HudPlugin,
            sound::SoundPlugin,
        ));

        // Tell bevy that our AppSystems should always be executed in the below order
        app.configure_sets(
            Update,
            (
                AppSystems::RecordInput,
                AppSystems::Physics,
                AppSystems::Update,
            )
                .chain(),
        );

        // Tell all of our used bevy schedules that they should only run Gameplay systems if we're in the gameplay screen
        app.configure_sets(PreUpdate, GameplaySystem.run_if(in_state(Screen::Gameplay)));
        app.configure_sets(Update, GameplaySystem.run_if(in_state(Screen::Gameplay)));
        app.configure_sets(
            PostUpdate,
            GameplaySystem.run_if(in_state(Screen::Gameplay)),
        );
        app.configure_sets(
            FixedUpdate,
            GameplaySystem.run_if(in_state(Screen::Gameplay)),
        );

        // Set up the `Pause` state.
        app.init_state::<Pause>();
        app.configure_sets(Update, PausableSystems.run_if(in_state(Pause(false))));

        // Set up a randomness source
        let rng = ChaCha8Rng::try_from_os_rng().unwrap_or(ChaCha8Rng::seed_from_u64(42));
        app.insert_resource(RandomSource(rng));
    }
}

/// High-level groupings/tags of systems for the app in the `Update` schedule.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum AppSystems {
    /// Record player input.
    RecordInput,
    /// Calculate physical forces based on entity components
    Physics,
    /// Do everything else (consider splitting this into further variants).
    Update,
}

/// Whether or not the game is paused.
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
struct Pause(pub bool);

/// A system set for systems that shouldn't run while the game is paused.
#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
struct PausableSystems;

/// A system set which marks systems that should only run during gameplay i.e. not during the loading screen
#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
struct GameplaySystem;

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

#[derive(Resource)]
struct RandomSource(ChaCha8Rng);

impl Deref for RandomSource {
    type Target = ChaCha8Rng;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RandomSource {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
