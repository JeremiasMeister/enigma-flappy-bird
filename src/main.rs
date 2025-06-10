use std::cmp::PartialEq;
use std::sync::Arc;
use enigma_3d::{AppState, camera, collision_world, event, EventLoop, light, material, object, postprocessing, texture, ui};
use enigma_3d::audio::AudioClip;
use enigma_3d::ui::Vec2;
use rand::Rng;
use std::fs;
use std::io::{Write, Read};
use std::path::Path;

// resources -> we load not via string but via bytes to include them in the built game
const BIRD: &'static [u8] = include_bytes!("res/bird.glb");
const PIPE: &'static [u8] = include_bytes!("res/pipe.glb");
const COIN: &'static [u8] = include_bytes!("res/coin.glb");
const BACKGROUND: &'static [u8] = include_bytes!("res/background.glb");
const BIRD_TEXTURE: &'static [u8] = include_bytes!("res/bird_texture.png");
const BACKGROUND_TEXTURE: &'static [u8] = include_bytes!("res/background_texture.png");
const HEART_TEXTURE: &'static [u8] = include_bytes!("res/heart.png");
const FONT_PRESS_START: &'static [u8] = include_bytes!("res/PrStart.ttf");

const BACKGROUND_MUSIC: &'static [u8] = include_bytes!("res/background-music.ogg");
const HIT_SOUND: &'static [u8] = include_bytes!("res/hit-sound.ogg");
const COLLECT_SOUND: &'static [u8] = include_bytes!("res/collect-sound.ogg");
const COLLECT_SOUND_TEN: &'static [u8] = include_bytes!("res/collect-sound-2.ogg");
const WUSH_SOUND: &'static [u8] = include_bytes!("res/wush.ogg");
const GAME_OVER_SOUND: &'static [u8] = include_bytes!("res/game-over.ogg");
const HIGHSCORE_FILE: &str = "enigma-3d_flappy_bird_highscore.txt";


#[derive(PartialEq)]
enum CollisionState {
    Coin,
    Pipe,
    None,
}

fn main() {
    let mut event_loop = EventLoop::new("Enigma 3D - Flappy Bird", 1080, 720);
    let mut app_state = AppState::new();

    let highscore = load_highscore();

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

    setup_scene(&mut app_state, &mut event_loop);

    app_state.inject_update_function(Arc::new(player_update));
    app_state.inject_update_function(Arc::new(update_pipes));
    app_state.inject_update_function(Arc::new(check_collision));
    app_state.inject_update_function(Arc::new(update_ui_timers));

    app_state.inject_event(event::EventCharacteristic::KeyPress(event::VirtualKeyCode::Space), Arc::new(player_jump), None);
    app_state.inject_event(event::EventCharacteristic::KeyPress(event::VirtualKeyCode::Escape), Arc::new(toggle_pause), None);

    app_state.add_post_process(Box::new(postprocessing::edge::Edge::new(&event_loop.display.clone(), 0.001, [0.0, 0.0, 0.0])));

    app_state.inject_gui(Arc::new(ui_function));

    // add audio
    let background_music = AudioClip::from_resource(BACKGROUND_MUSIC, "music");
    let hit = AudioClip::from_resource(HIT_SOUND,"hit");
    let collect = AudioClip::from_resource(COLLECT_SOUND, "collect");
    let collect_ten = AudioClip::from_resource(COLLECT_SOUND_TEN, "collect-ten");
    let wush = AudioClip::from_resource(WUSH_SOUND, "wush");
    let game_over = AudioClip::from_resource(GAME_OVER_SOUND, "game-over");
    app_state.add_audio(background_music);
    app_state.add_audio(hit);
    app_state.add_audio(collect);
    app_state.add_audio(collect_ten);
    app_state.add_audio(wush);
    app_state.add_audio(game_over);
    app_state.play_audio_loop("music");

    event_loop.run(app_state.convert_to_arc_mutex());
}

fn ui_function(context: &ui::Context, app_state: &mut AppState) {
    // set font //TODO: this should be on the appstate in future versions
    context.set_fonts({
        let mut fonts = ui::FontDefinitions::default();
        fonts.font_data.insert(
            "press_start".to_owned(),
            ui::FontData::from_static(FONT_PRESS_START),
        );
        fonts.families
            .entry(ui::FontFamily::Proportional)
            .or_default()
            .insert(0, "press_start".to_owned());

        fonts.families
            .entry(ui::FontFamily::Monospace)
            .or_default()
            .insert(0, "press_start".to_owned());

        fonts
    });

    let heart_texture_handle =
        if let Some(handle) = app_state.get_state_data_value::<ui::TextureHandle>("HEART_TEXTURE_HANDLE") {
            handle.clone() // Get a clone of the persistent handle
        } else {
            // This block now only runs ONCE at the very beginning
            let image = image::load_from_memory(HEART_TEXTURE).expect("Failed to load heart texture.");
            let image_buffer = image.to_rgba8();
            let size = [image.width() as _, image.height() as _];
            let pixels = image_buffer.as_flat_samples();
            let color_image = ui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

            let texture_handle = context.load_texture("heart_texture", color_image, Default::default());

            // Store a clone of the handle in the AppState
            app_state.add_state_data("HEART_TEXTURE_HANDLE", Box::new(texture_handle.clone()));
            texture_handle
        };

    let score = app_state.get_state_data_value::<i32>("SCORE")
        .map(|s| *s)
        .unwrap_or(0);

    let lives = app_state.get_state_data_value::<i32>("LIVES")
        .map(|l| *l)
        .unwrap_or(0);

    let pause = app_state.get_state_data_value::<bool>("PAUSE")
        .map(|l| *l)
        .unwrap_or(false);

    let highscore = app_state.get_state_data_value::<i32>("HIGHSCORE")
        .map(|s| *s)
        .unwrap_or(0);

    let top_bar_frame = ui::Frame {
        inner_margin: ui::Margin::symmetric(10.0, 10.0),
        fill: ui::Color32::from_rgba_unmultiplied(0, 0, 0, 45),
        ..Default::default()
    };

    ui::TopBottomPanel::top("top_score_panel")
        .frame(top_bar_frame)
        .show(context, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    ui::RichText::new("SCORE")
                        .color(ui::Color32::WHITE)
                        .size(40.0)
                );
                ui.label(
                    ui::RichText::new(format!("{}", score))
                        .color(ui::Color32::WHITE)
                        .size(40.0)
                        .strong(),
                );
                ui.add_space(30.0);
                ui.label(
                    ui::RichText::new("HIGHSCORE")
                        .color(ui::Color32::WHITE)
                        .size(40.0)
                );
                ui.label(
                    ui::RichText::new(format!("{}", highscore))
                        .color(ui::Color32::WHITE)
                        .size(40.0)
                        .strong(),
                );
                ui.with_layout(ui::Layout::right_to_left(ui::Align::Center), |ui| {
                    for _ in 0..lives {
                        ui.add_space(5.0);
                        ui.image((heart_texture_handle.id(), Vec2::new(35.0,35.0)));
                    }
                });
            });
        });

    let well_done_timer = app_state.get_state_data_value::<i32>("WELL_DONE_TIMER")
        .map(|t| *t)
        .unwrap_or(0);

    let try_again_timer = app_state.get_state_data_value::<i32>("TRY_AGAIN_TIMER")
        .map(|t| *t)
        .unwrap_or(0);

    if well_done_timer > 0 && score > 0 {
        // Use ui::Area for a frameless, background-less container
        ui::Area::new(ui::Id::new("well_done_area"))
            .anchor(ui::Align2::CENTER_CENTER, [0.0, 0.0]) // Still centered
            .show(context, |ui| {
                // The text itself remains the same
                ui.label(
                    ui::RichText::new("Well Done!")
                        .color(ui::Color32::from_rgb(255, 215, 0)) // Gold color
                        .size(50.0)
                        .strong()
                );
            });
    } else if try_again_timer > 0 && score == 0 {
        ui::Area::new(ui::Id::new("try_again_area"))
            .anchor(ui::Align2::CENTER_CENTER, [0.0, 0.0]) // Still centered
            .show(context, |ui| {
                // The text itself remains the same
                ui.label(
                    ui::RichText::new("Oh no! Try Again!")
                        .color(ui::Color32::from_rgb(255, 128, 0))
                        .size(50.0)
                        .strong()
                );
            });
    }

    if pause {
        ui::Area::new(ui::Id::new("pause_area"))
            .anchor(ui::Align2::CENTER_CENTER, [0.0, 0.0]) // Still centered
            .show(context, |ui| {
                // The text itself remains the same
                ui.label(
                    ui::RichText::new("Pause")
                        .color(ui::Color32::from_rgb(0, 0, 0)) // Gold color
                        .size(50.0)
                        .strong()
                );
            });
    }
}

fn setup_scene(app_state: &mut AppState, event_loop:  &mut EventLoop){
    //create a camera
    let camera = camera::Camera::new(Some([0.0, 0.0, 5.0]), Some([0.0, 0.0, 0.0]), Some(90.0), Some(16. / 9.), Some(0.01), Some(1024.));
    app_state.set_camera(camera);

    //create lights
    let light1 = light::Light::new([1.0, 1.0, 5.0], [1.0, 1.0, 1.0], 100.0, None, false);
    let ambient_light = light::Light::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], 0.50, None, false);
    app_state.add_light(ambient_light, light::LightEmissionType::Ambient);
    app_state.add_light(light1, light::LightEmissionType::Source);

    //create background
    let mut background = object::Object::load_from_gltf_resource(BACKGROUND, None);
    background.set_name("BACKGROUND".to_string());
    background.transform.set_position([0.0, 0.0, -8.0]);
    let mut background_mat = material::Material::unlit(event_loop.get_display_clone(), false);
    background_mat.set_texture_from_resource(BACKGROUND_TEXTURE, material::TextureType::Albedo);
    background.add_material(background_mat.uuid);
    app_state.add_object(background);
    app_state.add_material(background_mat);

    //create skybox
    let skybox_texture = texture::Texture::from_resource(event_loop.get_display_reference(), BACKGROUND_TEXTURE);
    app_state.set_skybox_from_texture(skybox_texture, event_loop);

    //create the player
    let mut player = object::Object::load_from_gltf_resource(BIRD, None);
    player.set_name("PLAYER".to_string());
    let mut player_mat = material::Material::unlit( event_loop.get_display_clone(), false);
    player_mat.set_texture_from_resource(BIRD_TEXTURE, material::TextureType::Albedo);
    player.add_material(player_mat.uuid);
    player.transform.set_position([0.0, 0.0, 0.0]);
    player.transform.set_scale([2.0, 2.0, 2.0]);
    player.transform.set_rotation([0.0, 0.0, 0.0]);
    app_state.add_object(player);
    app_state.add_material(player_mat);

    //create the pipes
    spawn_pipes(app_state, event_loop, 0.0);
    spawn_pipes(app_state, event_loop, 5.0);
    spawn_pipes(app_state, event_loop, 10.0);
    spawn_pipes(app_state, event_loop, 15.0);
    spawn_pipes(app_state, event_loop, 20.0);
    spawn_pipes(app_state, event_loop, 25.0);
    spawn_pipes(app_state, event_loop, 30.0);
    spawn_pipes(app_state, event_loop, 35.0);
}

fn load_highscore() -> i32 {
    if Path::new(HIGHSCORE_FILE).exists() {
        if let Ok(mut file) = fs::File::open(HIGHSCORE_FILE) {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                return contents.trim().parse::<i32>().unwrap_or(0);
            }
        }
    }
    0 // Default highscore
}

fn save_highscore(score: i32) {
    if let Ok(mut file) = fs::File::create(HIGHSCORE_FILE) {
        let _ = file.write_all(score.to_string().as_bytes());
    }
}

fn player_update(app_state: &mut AppState){
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

fn toggle_pause(app_state: &mut AppState){
    match app_state.get_state_data_value_mut::<bool>("PAUSE") {
        Some(p) => {
            *p = !*p;
            app_state.toggle_pause_audio("music");
        },
        None => {}
    }
}

fn player_jump(app_state: &mut AppState){
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

fn update_pipes(app_state: &mut AppState){
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

fn spawn_pipes(app_state: &mut AppState, event_loop: &mut EventLoop, x_offset: f32){
    let y_offset = rand::rng().random_range(-2.0..2.0);
    let pipe_spacing = 7.0;

    let mut pipe1_mat = material::Material::lit_pbr(event_loop.get_display_clone(), false);
    pipe1_mat.set_color([0.0, 1.0, 0.0]);

    let mut coin_mat = material::Material::lit_pbr(event_loop.get_display_clone(), false);
    coin_mat.set_color([1.0, 0.8, 0.0]);

    let mut pipe1 = object::Object::load_from_gltf_resource(PIPE, None);
    pipe1.set_name(String::from("PIPE1"));
    pipe1.add_material(pipe1_mat.uuid);
    pipe1.transform.set_position([5.0 + x_offset, pipe_spacing + y_offset, 0.0]);
    pipe1.transform.set_scale([1.0, 1.0, 0.5]);

    let mut pipe2 = pipe1.clone();
    pipe2.set_name(String::from("PIPE2"));
    pipe2.transform.set_position([5.0 + x_offset, -pipe_spacing + y_offset, 0.0]);
    pipe2.transform.set_scale([1.0, 1.0, 0.5]);

    let mut coin = object::Object::load_from_gltf_resource(COIN, None);
    coin.add_material(coin_mat.uuid);
    coin.set_name(String::from("COIN"));
    coin.transform.set_scale([0.5, 0.5, 0.5]);
    coin.transform.set_position([5.0 + x_offset, 0.0 + y_offset, 0.0]);


    app_state.add_object(coin);
    app_state.add_object(pipe1);
    app_state.add_object(pipe2);
    app_state.add_material(pipe1_mat);
    app_state.add_material(coin_mat);
}

fn update_ui_timers(app_state: &mut AppState) {
    match app_state.get_state_data_value_mut::<bool>("PAUSE") {
        Some(p) => {
            if *p {
                return;
            }
        },
        None => {}
    }
    if let Some(timer) = app_state.get_state_data_value_mut::<i32>("WELL_DONE_TIMER") {
        if *timer > 0 {
            *timer -= 1;
        }
    }
    if let Some(timer) = app_state.get_state_data_value_mut::<i32>("TRY_AGAIN_TIMER") {
        if *timer > 0 {
            *timer -= 1;
        }
    }
    if let Some(timer) = app_state.get_state_data_value_mut::<i32>("SAFE_TIMER") {
        if *timer > 0 {
            *timer -= 1;
        }
    }
}

fn check_collision(app_state: &mut AppState){
    match app_state.get_state_data_value_mut::<bool>("PAUSE") {
        Some(p) if *p => return,
        _ => {}
    }
    let is_safe = app_state.get_state_data_value::<i32>("SAFE_TIMER").map_or(false, |t| *t > 0);
    let player_option = app_state.get_object_mut("PLAYER");
    // define temp var to store collision
    let mut colliding = CollisionState::None;

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
                        colliding = CollisionState::Coin;
                        continue;
                    }
                }
                if !is_safe && object.name.contains("PIPE") {
                    let object_bounds = object.get_bounding_box();
                    if collision_world::is_colliding(&player_bounds, &object_bounds){
                        colliding = CollisionState::Pipe;
                        break;
                    }
                }
            }
        },
        None => {}
    }

    // setting player positions
    if colliding == CollisionState::Pipe {
        if let Some(player) = app_state.get_object_mut("PLAYER") {
            player.transform.set_position([0.0, 0.0, 0.0]);
            player.transform.set_rotation([0.0, 0.0, 0.0]);
        }
    }


    // handle lives
    let mut live_tracker = app_state.get_state_data_value::<i32>("LIVES").map_or(0, |l| *l);
    if colliding == CollisionState::Pipe {
        if let Some(live) = app_state.get_state_data_value_mut::<i32>("LIVES") {
            *live -= 1;
            live_tracker = *live;
        }
    }

    // now lets set the score
    let mut current_score = app_state.get_state_data_value::<i32>("SCORE").map_or(0, |s| *s);
    if let Some(s) = app_state.get_state_data_value_mut::<i32>("SCORE") {
        if colliding == CollisionState::Coin {
            *s += 1;
            current_score = *s;
        } else if colliding == CollisionState::Pipe {
            if live_tracker <= 0 {
                *s = 0;
                current_score = 0;
            }
        }
    }

    // let's set the highscore
    if colliding == CollisionState::Coin {
        if let Some(hs) = app_state.get_state_data_value_mut::<i32>("HIGHSCORE") {
            if current_score > *hs {
                *hs = current_score;
                save_highscore(*hs);
            }
        }
    }

    // finally set lives
    if colliding == CollisionState::Pipe && live_tracker <= 0 {
        if let Some(l) = app_state.get_state_data_value_mut::<i32>("LIVES") {
            *l = 3;
        }
    }

    // handling audio
    if colliding == CollisionState::Pipe {
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
    } else if colliding == CollisionState::Coin {
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