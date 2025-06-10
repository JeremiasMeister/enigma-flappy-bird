use enigma_3d::{AppState, collision_world};
use crate::game_utils;

pub fn player_update(app_state: &mut AppState){
    match app_state.get_state_data_value_mut::<bool>("PAUSE") {
        Some(p) => {
            if *p {
                return;
            }
        },
        None => {}
    }
    let timer_value = *app_state.get_state_data_value::<i32>("SAFE_TIMER").unwrap_or(&0);
    let is_safe = timer_value > 0;
    let player_option = app_state.get_object_mut("PLAYER");
    match player_option {
        Some(player) => {
            if is_safe {
                if (timer_value / 5) % 2 == 0 {
                    player.transform.set_scale([2.0, 2.0, 2.0]);
                } else {
                    player.transform.set_scale([0.0, 0.0, 0.0]); // "Hide" the player
                }
            }
            if player.transform.get_position().y > -5.0 {
                player.transform.move_dir_array([0.0, -0.05, 0.0]);
                player.transform.rotate([0.0, 0.0, -0.7])
            }
        },
        None => {}
    }
}

pub fn update_pipes(app_state: &mut AppState){
    match app_state.get_state_data_value_mut::<bool>("PAUSE") {
        Some(p) => {
            if *p {
                return;
            }
        },
        None => {}
    }
    for object in app_state.get_objects_mut(){
        if object.name.contains("PIPE") || object.name.contains("COIN") {
            object.transform.move_dir_array([-0.05, 0.0, 0.0]);
            if object.transform.get_position().x < -20.0 {
                object.transform.move_dir_array([40.0, 0.0, 0.0]);
                if object.name.contains("COIN") {
                    object.transform.set_scale([0.5, 0.5, 0.5]);
                    if object.transform.get_position().y > 5.0 {
                        object.transform.move_dir_array([0.0, -10.0, 0.0]);
                    }
                }
            }
        }

        if object.name.contains("COIN") {
            object.transform.rotate([0.0, 5.0, 0.0]);
        }
    }
}

pub fn check_collision(app_state: &mut AppState){
    match app_state.get_state_data_value_mut::<bool>("PAUSE") {
        Some(p) if *p => return,
        _ => {}
    }
    let is_safe = app_state.get_state_data_value::<i32>("SAFE_TIMER").map_or(false, |t| *t > 0);
    let player_option = app_state.get_object_mut("PLAYER");
    // define temp var to store collision
    let mut colliding = game_utils::CollisionState::None;

    // set collision state
    match player_option {
        Some(player) => {
            let player_bounds = player.get_bounding_box();
            for object in app_state.get_objects_mut(){
                if object.name.contains("COIN") {
                    let object_bounds = object.get_bounding_box();
                    if collision_world::is_colliding(&player_bounds, &object_bounds){
                        object.transform.set_scale([0.0, 0.0, 0.0]);
                        object.transform.move_dir_array([0.0, 10.0, 0.0]);
                        colliding = game_utils::CollisionState::Coin;
                        continue;
                    }
                }
                if !is_safe && object.name.contains("PIPE") {
                    let object_bounds = object.get_bounding_box();
                    if collision_world::is_colliding(&player_bounds, &object_bounds){
                        colliding = game_utils::CollisionState::Pipe;
                        break;
                    }
                }
            }
        },
        None => {}
    }

    // setting player positions
    if colliding == game_utils::CollisionState::Pipe {
        if let Some(player) = app_state.get_object_mut("PLAYER") {
            player.transform.set_position([0.0, 0.0, 0.0]);
            player.transform.set_rotation([0.0, 0.0, 0.0]);
        }
    }


    // handle lives
    let mut live_tracker = app_state.get_state_data_value::<i32>("LIVES").map_or(0, |l| *l);
    if colliding == game_utils::CollisionState::Pipe {
        if let Some(live) = app_state.get_state_data_value_mut::<i32>("LIVES") {
            *live -= 1;
            live_tracker = *live;
        }
    }

    // now lets set the score
    let mut current_score = app_state.get_state_data_value::<i32>("SCORE").map_or(0, |s| *s);
    if let Some(s) = app_state.get_state_data_value_mut::<i32>("SCORE") {
        if colliding == game_utils::CollisionState::Coin {
            *s += 1;
            current_score = *s;
        } else if colliding == game_utils::CollisionState::Pipe {
            if live_tracker <= 0 {
                *s = 0;
                current_score = 0;
            }
        }
    }

    // let's set the highscore
    if colliding == game_utils::CollisionState::Coin {
        if let Some(hs) = app_state.get_state_data_value_mut::<i32>("HIGHSCORE") {
            if current_score > *hs {
                *hs = current_score;
                game_utils::save_highscore(*hs);
            }
        }
    }

    // finally set lives
    if colliding == game_utils::CollisionState::Pipe && live_tracker <= 0 {
        if let Some(l) = app_state.get_state_data_value_mut::<i32>("LIVES") {
            *l = 3;
        }
    }

    // handling audio
    if colliding == game_utils::CollisionState::Pipe {
        if live_tracker <= 0 {
            app_state.play_audio_once("game-over");
            if let Some(timer) = app_state.get_state_data_value_mut::<i32>("TRY_AGAIN_TIMER") {
                *timer = 120; // 2 seconds
            }
        } else {
            app_state.play_audio_once("hit");
            // Set the safe timer since a life was lost but it's not game over
            if let Some(timer) = app_state.get_state_data_value_mut::<i32>("SAFE_TIMER") {
                *timer = 120; // 2 seconds of immunity
            }
        }
    } else if colliding == game_utils::CollisionState::Coin {
        if current_score > 0 && current_score % 10 == 0 {
            app_state.play_audio_once("collect-ten");
            if let Some(timer) = app_state.get_state_data_value_mut::<i32>("WELL_DONE_TIMER") {
                *timer = 120; // 2 seconds
            }
        } else {
            app_state.play_audio_once("collect");
        }
    }
}