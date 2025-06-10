use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use enigma_3d::{AppState, camera, EventLoop, light, material, object, texture};
use rand::Rng;
use crate::game_resources;

#[derive(PartialEq)]
pub enum CollisionState {
    Coin,
    Pipe,
    None,
}

pub fn setup_scene(app_state: &mut AppState, event_loop:  &mut EventLoop){
    //create a camera
    let camera = camera::Camera::new(Some([0.0, 0.0, 5.0]), Some([0.0, 0.0, 0.0]), Some(90.0), Some(16. / 9.), Some(0.01), Some(1024.));
    app_state.set_camera(camera);

    //create lights
    let light1 = light::Light::new([1.0, 1.0, 5.0], [1.0, 1.0, 1.0], 100.0, None, false);
    let ambient_light = light::Light::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], 0.50, None, false);
    app_state.add_light(ambient_light, light::LightEmissionType::Ambient);
    app_state.add_light(light1, light::LightEmissionType::Source);

    //create background
    let mut background = object::Object::load_from_gltf_resource(game_resources::BACKGROUND, None);
    background.set_name("BACKGROUND".to_string());
    background.transform.set_position([0.0, 0.0, -8.0]);
    let mut background_mat = material::Material::unlit(event_loop.get_display_clone(), false);
    background_mat.set_texture_from_resource(game_resources::BACKGROUND_TEXTURE, material::TextureType::Albedo);
    background.add_material(background_mat.uuid);
    app_state.add_object(background);
    app_state.add_material(background_mat);

    //create skybox
    let skybox_texture = texture::Texture::from_resource(event_loop.get_display_reference(), game_resources::BACKGROUND_TEXTURE);
    app_state.set_skybox_from_texture(skybox_texture, event_loop);

    //create the player
    let mut player = object::Object::load_from_gltf_resource(game_resources::BIRD, None);
    player.set_name("PLAYER".to_string());
    let mut player_mat = material::Material::unlit( event_loop.get_display_clone(), false);
    player_mat.set_texture_from_resource(game_resources::BIRD_TEXTURE, material::TextureType::Albedo);
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

fn spawn_pipes(app_state: &mut AppState, event_loop: &mut EventLoop, x_offset: f32){
    let y_offset = rand::rng().random_range(-2.0..2.0);
    let pipe_spacing = 7.0;

    let mut pipe1_mat = material::Material::lit_pbr(event_loop.get_display_clone(), false);
    pipe1_mat.set_color([0.0, 1.0, 0.0]);

    let mut coin_mat = material::Material::lit_pbr(event_loop.get_display_clone(), false);
    coin_mat.set_color([1.0, 0.8, 0.0]);

    let mut pipe1 = object::Object::load_from_gltf_resource(game_resources::PIPE, None);
    pipe1.set_name(String::from("PIPE1"));
    pipe1.add_material(pipe1_mat.uuid);
    pipe1.transform.set_position([5.0 + x_offset, pipe_spacing + y_offset, 0.0]);
    pipe1.transform.set_scale([1.0, 1.0, 0.5]);

    let mut pipe2 = pipe1.clone();
    pipe2.set_name(String::from("PIPE2"));
    pipe2.transform.set_position([5.0 + x_offset, -pipe_spacing + y_offset, 0.0]);
    pipe2.transform.set_scale([1.0, 1.0, 0.5]);

    let mut coin = object::Object::load_from_gltf_resource(game_resources::COIN, None);
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

pub fn load_highscore() -> i32 {
    if Path::new(game_resources::HIGHSCORE_FILE).exists() {
        if let Ok(mut file) = fs::File::open(game_resources::HIGHSCORE_FILE) {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                return contents.trim().parse::<i32>().unwrap_or(0);
            }
        }
    }
    0 // Default highscore
}

pub fn save_highscore(score: i32) {
    if let Ok(mut file) = fs::File::create(game_resources::HIGHSCORE_FILE) {
        let _ = file.write_all(score.to_string().as_bytes());
    }
}