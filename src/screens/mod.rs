//! The game's main screen states and transitions between them.

mod loading;
mod gameplay;
mod gameover;
//mod splash;
//mod title;

use bevy::prelude::*;
use crate::collision::FatalCollisionEvent;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Screen>();
    app.add_plugins((
        gameplay::plugin,
        loading::plugin,
        gameover::plugin,
        //splash::plugin,
        //title::plugin,
    ));
}

/// The game's main screen states.
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum Screen {
    #[default]
    Loading,
    Title,
    Gameplay,
    Gameover
}


