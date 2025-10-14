//! The screen state for the main gameplay.

use bevy::prelude::*;
use crate::score::Score;
use crate::screens::Screen;
use crate::sun_system::SolarSystemAssets;


#[derive(Resource, Default)]
pub struct GameEnd{
    pub game_end_time: f32,
    pub ktype: f32,
    pub enabled: bool,
}

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(GameEnd{game_end_time:600.0, ktype: 0.0, enabled: false});
    app.add_systems(Update, enter_gameover_screen.run_if(in_state(Screen::Gameplay).and(is_gameover)));
    app.add_systems(OnEnter(Screen::Gameover), show_game_over);
    app.add_systems(OnEnter(Screen::Gameplay), reset_game_end_timer);
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
   if (game_end.enabled && (time.elapsed_secs() - game_end.game_end_time > 0.)) || score.energy_rate >= 400. {
       return true;
   }
    return false;
}


#[derive(Component)]
struct GameOverPopup;

fn show_game_over(mut commands: Commands, mut score: ResMut<Score>,
                  mut game_end: ResMut<GameEnd>,
                  solar_system_assets: Res<SolarSystemAssets>) {
    if score.energy_rate >= 400. { score.energy_rate=400.;}
    //let toYotta: f64=(score.energy_rate/100.) as f64* 1e24_f64; // multiplied by yotta

    game_end.ktype = (1.0 + (score.energy_rate - 1.0) / 399.0).clamp(1.0, 2.0);//((toYotta.log10() - 6.0) / 10.0).abs();//((score.energy_rate.log10() + 9.0) / 10.0).max(0.0);
    info!("show Game Over {}", game_end.ktype);
    // Stop countdown immediately on game over (e.g., win by energy)
    game_end.enabled = false;

    let text_center = Justify::Center;
    let mut better_earth = "";
    if game_end.ktype > 1.46{
        better_earth="You generate more Energy than 2 Earths!";
    }
    let mut game_end_string = "GAME OVER";
    if score.energy_rate >= 400. {
        game_end_string = "YOU WON!";
    }
    // Game-Over Popup
    commands.spawn((
        GameOverPopup,
        Pickable::IGNORE,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        children![
            (
                Node {
                    width: Val::Px(400.0),
                    height: Val::Px(380.0),
                    border: UiRect::all(Val::Px(2.0)),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(20.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
                Outline {
                    width: Val::Px(3.0),
                    offset: Default::default(),
                    color: Color::xyz(0.4811, 0.3064, 0.0253),
                },
                children![
                    // Title
                    (
                        Text::new(game_end_string),
                        Node {
                            margin: UiRect::bottom(Val::Px(20.0)),
                            ..default()
                        },
                        TextFont {
                            font: solar_system_assets.font.clone(),
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::xyz(0.4811, 0.3064, 0.0253)),
                        TextLayout::new_with_justify(text_center),
                    ),
                    // Energy bar decoration
                    (
                        Text::new("═══════════════════════════"),
                        Node {
                            margin: UiRect::bottom(Val::Px(15.0)),
                            ..default()
                        },
                        TextFont {
                            font: solar_system_assets.font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::xyz(0.4811, 0.3064, 0.0253)),
                    ),
                    // Energy Rate
                    (
                        Text::new(format!("ENERGY RATE\n{:.3} YT", score.energy_rate)),
                        Node {
                            margin: UiRect::bottom(Val::Px(20.0)),
                            ..default()
                        },
                        TextFont {
                            font: solar_system_assets.font.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::xyz(0.4811, 0.3064, 0.0253)),
                        TextLayout::new_with_justify(text_center),
                    ),
                    // Total Energy
                    (
                        Text::new(format!("TOTAL ENERGY STORED\n{:.2} YTh", score.energy_stored)),
                        Node {
                            margin: UiRect::bottom(Val::Px(20.0)),
                            ..default()
                        },
                        TextFont {
                            font: solar_system_assets.font.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::xyz(0.4811, 0.3064, 0.0253)),
                        TextLayout::new_with_justify(text_center),
                    ),
                    // Kardashev Scale
                    (
                        Text::new(format!("Kardashev \nTYPE {:.3}\n {} ", game_end.ktype,better_earth)),
                        Node {
                            margin: UiRect::bottom(Val::Px(25.0)),
                            ..default()
                        },
                        TextFont {
                            font: solar_system_assets.font.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::xyz(0.4811, 0.3064, 0.0253)),
                        TextLayout::new_with_justify(text_center),
                    ),
                    // Total Energy
                    (
                        Text::new(format!("A Type 2 Civilization harnesses all power of a star")),

                        TextFont {
                            font: solar_system_assets.font.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::xyz(0.4811, 0.3064, 0.0253)),
                        TextLayout::new_with_justify(text_center),
                    ),
                    // Bottom bar decoration
                    (
                        Text::new("═══════════════════════════"),
                        Node {
                            margin: UiRect::bottom(Val::Px(10.0)),
                            ..default()
                        },
                        TextFont {
                            font: solar_system_assets.font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::xyz(0.4811, 0.3064, 0.0253)),
                    ),

                ],
            )
        ],
    ));
}

fn reset_game_end_timer(mut game_end: ResMut<GameEnd>, time: Res<Time>) {
    // Enable and start countdown whenever we enter Gameplay
    game_end.enabled = true;
    game_end.game_end_time = time.elapsed_secs() + game_end.game_end_time;
    game_end.ktype = 0.0;
}