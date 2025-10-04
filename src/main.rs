// Support configuring Bevy lints within code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

mod asset_tracking;
#[cfg(feature = "dev")]
mod dev_tools;
mod physics;
mod screens;
mod sun_system;
mod launching;
mod collision;

use bevy::log::{Level, LogPlugin};
use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy::window::WindowResolution;

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
                        title: "Bevy New 2D".to_string(),
                        fit_canvas_to_parent: true,
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
        ));

        // Order new `AppSystems` variants by adding them here:
        app.configure_sets(
            Update,
            (
                AppSystems::RecordInput,
                AppSystems::Physics,
                AppSystems::Update,
            )
                .chain(),
        );

        // Set up the `Pause` state.
        app.init_state::<Pause>();
        app.configure_sets(Update, PausableSystems.run_if(in_state(Pause(false))));
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
#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct PausableSystems;

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
