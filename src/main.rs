
mod game_resources;
mod game_ui;
mod game_events;
mod game_update;
mod game_utils;

use std::sync::Arc;
use enigma_3d::{AppState, event, EventLoop, postprocessing};
use enigma_3d::audio::AudioClip;

fn main() {
    let mut event_loop = EventLoop::new("Enigma 3D - Flappy Bird", 1080, 720);
    let mut app_state = AppState::new();

    let highscore = game_utils::load_highscore();

    // init score, well done timer and lives
    app_state.add_state_data("SCORE", Box::new(0i32));
    app_state.add_state_data("HIGHSCORE", Box::new(highscore));
    app_state.add_state_data("WELL_DONE_TIMER", Box::new(0i32));
    app_state.add_state_data("TRY_AGAIN_TIMER", Box::new(0i32));
    app_state.add_state_data("SAFE_TIMER", Box::new(0i32));
    app_state.add_state_data("LIVES", Box::new(3i32));
    app_state.add_state_data("PAUSE", Box::new(false));

    app_state.set_fps(60);
    app_state.set_max_buffers(3);

    game_utils::setup_scene(&mut app_state, &mut event_loop);

    app_state.inject_update_function(Arc::new(game_update::player_update));
    app_state.inject_update_function(Arc::new(game_update::update_pipes));
    app_state.inject_update_function(Arc::new(game_update::check_collision));
    app_state.inject_update_function(Arc::new(game_ui::update_ui_timers));

    app_state.inject_event(event::EventCharacteristic::KeyPress(event::VirtualKeyCode::Space), Arc::new(game_events::player_jump), None);
    app_state.inject_event(event::EventCharacteristic::KeyPress(event::VirtualKeyCode::Escape), Arc::new(game_events::toggle_pause), None);

    app_state.add_post_process(Box::new(postprocessing::edge::Edge::new(&event_loop.display.clone(), 0.001, [0.0, 0.0, 0.0])));

    app_state.inject_gui(Arc::new(game_ui::ui_header));
    app_state.inject_gui(Arc::new(game_ui::ui_pause));
    app_state.inject_gui(Arc::new(game_ui::ui_popups));

    // add audio
    let background_music = AudioClip::from_resource(game_resources::BACKGROUND_MUSIC, "music");
    let hit = AudioClip::from_resource(game_resources::HIT_SOUND,"hit");
    let collect = AudioClip::from_resource(game_resources::COLLECT_SOUND, "collect");
    let collect_ten = AudioClip::from_resource(game_resources::COLLECT_SOUND_TEN, "collect-ten");
    let wush = AudioClip::from_resource(game_resources::WUSH_SOUND, "wush");
    let game_over = AudioClip::from_resource(game_resources::GAME_OVER_SOUND, "game-over");
    app_state.add_audio(background_music);
    app_state.add_audio(hit);
    app_state.add_audio(collect);
    app_state.add_audio(collect_ten);
    app_state.add_audio(wush);
    app_state.add_audio(game_over);
    app_state.play_audio_loop("music");

    event_loop.run(app_state.convert_to_arc_mutex());
}