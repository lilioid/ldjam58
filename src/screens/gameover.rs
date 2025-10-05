//! The screen state for the main gameplay.

use bevy::input::common_conditions::input_just_pressed;
use bevy::input::mouse::MouseWheel;
use crate::sun_system::{init_sun_system, setup_grid_image, Satellite, Sun};
use bevy::prelude::*;
use bevy::time::common_conditions::paused;
use crate::collision::FatalCollisionEvent;
use crate::GameplaySystem;
use crate::score::Score;
use crate::screens::Screen;


#[derive(Resource, Default)]
pub struct GameEnd{
    pub game_end_time: f32,
    pub ktype: f32
}

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(GameEnd{game_end_time:600.0, ktype: 0.0});
    app.add_systems(Update, enter_gameover_screen.run_if(in_state(Screen::Gameplay).and(is_gameover)));
    app.add_systems(OnEnter(Screen::Gameover), show_game_over);
}




fn spawn_GameOver_screen(mut commands: Commands) {
    commands.spawn(DespawnOnExit(Screen::Loading));
}

fn enter_gameover_screen(mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Gameover);
}


fn is_gameover( score: Res<Score>,
                      time: Res<Time>,
                      game_end: Res<GameEnd>) -> bool {
    // 400 Yottawatt are 4 x 10^26, Kardashev type two,2.0 energy threshold
    // Start at 10 petawatt Kardashev scale 1.0
   if( time.elapsed_secs() - game_end.game_end_time > 0. || score.energy_stored > 400.){
       return true;
   }
    return false;
}


fn show_game_over(mut commands: Commands, mut score: ResMut<Score>,
                  mut game_end: ResMut<GameEnd>) {
    game_end.ktype = ((score.energy_stored.log10() - 6.0) / 10.0).abs();
    info!("show Game Over {}", game_end.ktype);
    //show game over screen / stats / achievements?
}