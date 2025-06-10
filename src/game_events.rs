use enigma_3d::AppState;

pub fn player_jump(app_state: &mut AppState){
    match app_state.get_state_data_value_mut::<bool>("PAUSE") {
        Some(p) => {
            if *p {
                return;
            }
        },
        None => {}
    }
    app_state.play_audio_once("wush");
    let player_option = app_state.get_object_mut("PLAYER");
    match player_option {
        Some(player) => {
            if player.transform.get_position().y < 5.0 {
                player.transform.move_dir_array([0.0, 1.0, 0.0]);
                player.transform.set_rotation([0.0, 0.0, 35.0])
            }
        },
        None => {}
    }
}

pub fn toggle_pause(app_state: &mut AppState){
    match app_state.get_state_data_value_mut::<bool>("PAUSE") {
        Some(p) => {
            *p = !*p;
            app_state.toggle_pause_audio("music");
        },
        None => {}
    }
}