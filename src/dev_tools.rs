//! Development tools for the game. This plugin is only enabled in dev builds.

use crate::Pause;
use crate::screens::Screen;
use bevy::{
    dev_tools::states::log_transitions, input::common_conditions::input_just_pressed, prelude::*, 
};
//use crate::screens::Screen;

const TOGGLE_KEY: KeyCode = KeyCode::Space;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, log_transitions::<Screen>);

    // Toggle the debug overlay for UI.
    app.add_systems(
        Update,
        toggle_debug_ui
            .run_if(in_state(Screen::Gameplay))
            .run_if(input_just_pressed(TOGGLE_KEY)),
    );
}

pub fn is_debug_enabled(options: Res<UiDebugOptions>) -> bool {
    return options.enabled
}

fn toggle_debug_ui(mut options: ResMut<UiDebugOptions>) {
    options.toggle();
}

fn toggle_pause(current_pause: Res<State<Pause>>, mut next_pause: ResMut<NextState<Pause>>) {
    match current_pause.0 {
        true => {
            debug!("Pausing game");
            next_pause.set(Pause(false))
        }
        false => {
            debug!("Unpausing game");
            next_pause.set(Pause(true))
        }
    }
}
