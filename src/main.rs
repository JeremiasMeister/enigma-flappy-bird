extern crate enigma;
use std::sync::Arc;
use winit;

//globals
static mut SCORE: i32 = 0;

// resources -> we load not via string but via bytes to include them in the built game
const BIRD: &'static [u8] = include_bytes!("res/bird.glb");
const PIPE: &'static [u8] = include_bytes!("res/pipe.glb");
const COIN: &'static [u8] = include_bytes!("res/coin.glb");
const BACKGROUND: &'static [u8] = include_bytes!("res/background.glb");
const BIRD_TEXTURE: &'static [u8] = include_bytes!("res/bird_texture.png");
const BACKGROUND_TEXTURE: &'static [u8] = include_bytes!("res/background_texture.png");

fn main() {
    let mut event_loop = enigma::EventLoop::new("Enigma 3D - Flappy Bird", 1080, 720);
    let mut app_state = enigma::AppState::new();

    app_state.set_fps(60);
    app_state.set_max_buffers(3);

    setup_scene(&mut app_state, &mut event_loop);

    app_state.inject_update_function(Arc::new(player_update));
    app_state.inject_update_function(Arc::new(update_pipes));
    app_state.inject_update_function(Arc::new(check_collision));

    app_state.inject_event(enigma::event::EventCharacteristic::KeyPress(winit::event::VirtualKeyCode::Space), Arc::new(player_jump));

    app_state.add_post_process(Box::new(enigma::postprocessing::edge::Edge::new(&event_loop.display.clone(), 0.95, [0.0, 0.0, 0.0])));

    app_state.inject_gui(Arc::new(ui_function));

    event_loop.run(app_state.convert_to_arc_mutex());
}

fn ui_style(context: &egui::Context) -> egui::Style {
    let mut style = (*context.style()).clone();

    // Adjust the text style to make the font larger
    let text_style = egui::TextStyle::Body; // Choose an appropriate text style
    let header_style = egui::TextStyle::Heading; // Choose an appropriate text style
    if let Some(font) = style.text_styles.get_mut(&text_style) {
        font.size = 60.0; // Set your desired font size, making it larger for a bolder look
    }
    if let Some(font) = style.text_styles.get_mut(&header_style) {
        font.size = 60.0; // Set your desired font size, making it larger for a bolder look
    }

    // Customize other visual styles as before
    style.visuals.window_fill = egui::Color32::TRANSPARENT;
    style.visuals.override_text_color = Some(egui::Color32::WHITE);
    style.visuals.window_shadow.extrusion = 0.0;
    style.visuals.window_shadow.color = egui::Color32::TRANSPARENT;
    style.visuals.window_stroke = egui::Stroke::new(0.0, egui::Color32::TRANSPARENT);

    style
}

fn ui_function(context: &egui::Context, _app_state: &mut enigma::AppState) {
    context.set_style(ui_style(context));
    egui::Window::new("Score")
        .frame(egui::Frame::none())
        .default_width(200.0)
        .default_height(200.0)
        .show(context, |ui| unsafe {
            ui.label(format!("{}", SCORE));
        });
}

fn setup_scene(app_state: &mut enigma::AppState, event_loop:  &mut enigma::EventLoop){
    //create a camera
    let camera = enigma::camera::Camera::new(Some([0.0, 0.0, 5.0]), Some([0.0, 0.0, 0.0]), Some(90.0), Some(16. / 9.), Some(0.01), Some(1024.));
    app_state.set_camera(camera);

    //create lights
    let light1 = enigma::light::Light {
        position: [1.0, 1.0, 5.0],
        color: [1.0, 1.0, 1.0],
        intensity: 100.0,
    };
    let ambient_light = enigma::light::Light {
        position: [0.0, 0.0, 0.0],
        color: [1.0, 1.0, 1.0],
        intensity: 0.50,
    };
    app_state.add_light(ambient_light, enigma::light::LightType::Ambient);
    app_state.add_light(light1, enigma::light::LightType::Point);

    //create background
    let mut background = enigma::object::Object::load_from_gltf_resource(BACKGROUND);
    background.set_name("BACKGROUND".to_string());
    background.transform.set_position([0.0, 0.0, -8.0]);
    let mut background_mat = enigma::material::Material::unlit(event_loop.get_display_clone(), false);
    background_mat.set_texture_from_resource(BACKGROUND_TEXTURE, enigma::material::TextureType::Albedo);
    background.add_material(background_mat);
    app_state.add_object(background);

    //create the player
    let mut player = enigma::object::Object::load_from_gltf_resource(BIRD);
    player.set_name("PLAYER".to_string());
    let mut player_mat = enigma::material::Material::unlit( event_loop.get_display_clone(), false);
    player_mat.set_texture_from_resource(BIRD_TEXTURE, enigma::material::TextureType::Albedo);
    player.add_material(player_mat);
    player.transform.set_position([0.0, 0.0, 0.0]);
    player.transform.set_scale([2.0, 2.0, 2.0]);
    player.transform.set_rotation([0.0, 0.0, 0.0]);

    app_state.add_object(player);

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

fn player_update(app_state: &mut enigma::AppState){
    let player_option = app_state.get_object_mut("PLAYER");
    match player_option {
        Some(player) => {
            if player.transform.get_position().y > -5.0 {
                player.transform.move_dir([0.0, -0.05, 0.0]);
                player.transform.rotate([0.0, 0.0, -0.7])
            }
        },
        None => {}
    }
}

fn player_jump(app_state: &mut enigma::AppState){
    let player_option = app_state.get_object_mut("PLAYER");
    match player_option {
        Some(player) => {
            if player.transform.get_position().y < 5.0 {
                player.transform.move_dir([0.0, 1.0, 0.0]);
                player.transform.set_rotation([0.0, 0.0, 35.0])
            }
        },
        None => {}
    }
}

fn update_pipes(app_state: &mut enigma::AppState){
    for object in app_state.get_objects_mut(){
        if object.name.contains("PIPE") || object.name.contains("COIN") {
            object.transform.move_dir([-0.05, 0.0, 0.0]);
            if object.transform.get_position().x < -20.0 {
                object.transform.move_dir([40.0, 0.0, 0.0]);
                if object.name.contains("COIN") {
                    object.transform.set_scale([0.5, 0.5, 0.5]);
                    object.transform.move_dir([0.0, -10.0, 0.0]);
                }
            }
        }

        if object.name.contains("COIN") {
            object.transform.rotate([0.0, 5.0, 0.0]);
        }
    }
}

fn spawn_pipes(app_state: &mut enigma::AppState, event_loop: &mut enigma::EventLoop, x_offset: f32, y_offset: f32){
    let pipe_spacing = 7.0;

    let mut pipe1 = enigma::object::Object::load_from_gltf_resource(PIPE);
    pipe1.set_name(String::from("PIPE1"));
    let mut pipe1_mat = enigma::material::Material::lit_pbr(event_loop.get_display_clone(), false);
    pipe1_mat.set_color([0.0, 1.0, 0.0]);

    pipe1.add_material(pipe1_mat);
    pipe1.transform.set_position([5.0 + x_offset, pipe_spacing + y_offset, 0.0]);
    pipe1.transform.set_scale([1.0, 1.0, 0.5]);
    app_state.add_object(pipe1);

    let mut pipe2 = enigma::object::Object::load_from_gltf_resource(PIPE);
    pipe2.set_name(String::from("PIPE2"));
    let mut pipe2_mat = enigma::material::Material::lit_pbr(event_loop.get_display_clone(), false);
    pipe2_mat.set_color([0.0, 1.0, 0.0]);
    pipe2.add_material(pipe2_mat);
    pipe2.transform.set_position([5.0 + x_offset, -pipe_spacing + y_offset, 0.0]);
    pipe2.transform.set_scale([1.0, 1.0, 0.5]);
    app_state.add_object(pipe2);

    let mut coin = enigma::object::Object::load_from_gltf_resource(COIN);
    coin.set_name(String::from("COIN"));
    coin.transform.set_scale([0.5, 0.5, 0.5]);
    let mut coin_mat = enigma::material::Material::lit_pbr(event_loop.get_display_clone(), false);
    coin_mat.set_color([1.0, 0.8, 0.0]);
    coin.add_material(coin_mat);
    coin.transform.set_position([5.0 + x_offset, 0.0 + y_offset, 0.0]);
    app_state.add_object(coin);
}

fn check_collision(app_state: &mut enigma::AppState){
    let player_option = app_state.get_object_mut("PLAYER");
    match player_option {
        Some(player) => unsafe {
            let player_bounds = player.get_bounding_box();
            for object in app_state.get_objects_mut(){
                if object.name.contains("COIN") {
                    let object_bounds = object.get_bounding_box();
                    if enigma::collision_world::is_colliding(&player_bounds, &object_bounds){
                        object.transform.set_scale([0.0, 0.0, 0.0]);
                        object.transform.move_dir([0.0, 10.0, 0.0]);
                        SCORE += 1;
                        continue;
                    }
                }
                if object.name.contains("PIPE") {
                    let object_bounds = object.get_bounding_box();
                    if enigma::collision_world::is_colliding(&player_bounds, &object_bounds){
                        SCORE = 0;
                        continue;
                    }
                }
            }
        },
        None => {}
    }
}