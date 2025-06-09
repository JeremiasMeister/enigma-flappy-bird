use std::cmp::PartialEq;
use std::sync::Arc;
use enigma_3d::{AppState, camera, collision_world, event, EventLoop, light, material, object, postprocessing, texture, ui};

// resources -> we load not via string but via bytes to include them in the built game
const BIRD: &'static [u8] = include_bytes!("res/bird.glb");
const PIPE: &'static [u8] = include_bytes!("res/pipe.glb");
const COIN: &'static [u8] = include_bytes!("res/coin.glb");
const BACKGROUND: &'static [u8] = include_bytes!("res/background.glb");
const BIRD_TEXTURE: &'static [u8] = include_bytes!("res/bird_texture.png");
const BACKGROUND_TEXTURE: &'static [u8] = include_bytes!("res/background_texture.png");
const FONT_PRESS_START: &'static [u8] = include_bytes!("res/PrStart.ttf");


#[derive(PartialEq)]
enum CollisionState {
    Coin,
    Pipe,
    None,
}

fn main() {
    let mut event_loop = EventLoop::new("Enigma 3D - Flappy Bird", 1080, 720);
    let mut app_state = AppState::new();

    // init score
    app_state.add_state_data("SCORE", Box::new(0i32));

    app_state.set_fps(60);
    app_state.set_max_buffers(3);

    setup_scene(&mut app_state, &mut event_loop);

    app_state.inject_update_function(Arc::new(player_update));
    app_state.inject_update_function(Arc::new(update_pipes));
    app_state.inject_update_function(Arc::new(check_collision));

    app_state.inject_event(event::EventCharacteristic::KeyPress(event::VirtualKeyCode::Space), Arc::new(player_jump), None);

    app_state.add_post_process(Box::new(postprocessing::edge::Edge::new(&event_loop.display.clone(), 0.001, [0.0, 0.0, 0.0])));

    app_state.inject_gui(Arc::new(ui_function));

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

    let score = app_state.get_state_data_value::<i32>("SCORE")
        .map(|s| *s)
        .unwrap_or(0);

    let top_bar_frame = ui::Frame {
        inner_margin: ui::Margin::symmetric(10.0, 10.0),
        fill: ui::Color32::from_rgba_unmultiplied(0, 0, 0, 45),
        //stroke: ui::Stroke::new(0.0, ui::Color32::from_gray(45)),
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
            });
        });
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
    spawn_pipes(app_state, event_loop, 0.0, 0.0);
    spawn_pipes(app_state, event_loop, 5.0, 1.0);
    spawn_pipes(app_state, event_loop, 10.0, 0.0);
    spawn_pipes(app_state, event_loop, 15.0, -1.0);
    spawn_pipes(app_state, event_loop, 20.0, 0.0);
    spawn_pipes(app_state, event_loop, 25.0, 1.0);
    spawn_pipes(app_state, event_loop, 30.0, 0.0);
    spawn_pipes(app_state, event_loop, 35.0, -1.0);
}

fn player_update(app_state: &mut AppState){
    let player_option = app_state.get_object_mut("PLAYER");
    match player_option {
        Some(player) => {
            if player.transform.get_position().y > -5.0 {
                player.transform.move_dir_array([0.0, -0.05, 0.0]);
                player.transform.rotate([0.0, 0.0, -0.7])
            }
        },
        None => {}
    }
}

fn player_jump(app_state: &mut AppState){
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

fn spawn_pipes(app_state: &mut AppState, event_loop: &mut EventLoop, x_offset: f32, y_offset: f32){
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

fn check_collision(app_state: &mut AppState){
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
                if object.name.contains("PIPE") {
                    let object_bounds = object.get_bounding_box();
                    if collision_world::is_colliding(&player_bounds, &object_bounds){
                        colliding = CollisionState::Pipe;
                        continue;
                    }
                }
            }
        },
        None => {}
    }

    // setting player positions
    if colliding == CollisionState::Pipe {
        match app_state.get_object_mut("PLAYER") {
            Some(player) => {
                player.transform.set_position([0.0, 0.0, 0.0]);
                player.transform.set_scale([2.0, 2.0, 2.0]);
                player.transform.set_rotation([0.0, 0.0, 0.0]);
            },
            None => {}
        }
    }

    // now lets set the score
    match app_state.get_state_data_value_mut::<i32>("SCORE"){
        Some(s) => {
            if colliding == CollisionState::Coin {
                *s = *s + 1;
            } else if colliding == CollisionState::Pipe {
                *s = 0;
            }
        },
        None => {}
    }
}