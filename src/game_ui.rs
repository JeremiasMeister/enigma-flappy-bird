use enigma_3d::{AppState, ui};
use enigma_3d::ui::Vec2;
use crate::game_resources;


pub fn ui_header(context: &ui::Context, app_state: &mut AppState){
    // set font //TODO: this should be on the appstate in future versions
    context.set_fonts({
        let mut fonts = ui::FontDefinitions::default();
        fonts.font_data.insert(
            "press_start".to_owned(),
            ui::FontData::from_static(game_resources::FONT_PRESS_START),
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
            let image = image::load_from_memory(game_resources::HEART_TEXTURE).expect("Failed to load heart texture.");
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
}

pub fn ui_pause(context: &ui::Context, app_state: &mut AppState) {
    // set font //TODO: this should be on the appstate in future versions
    context.set_fonts({
        let mut fonts = ui::FontDefinitions::default();
        fonts.font_data.insert(
            "press_start".to_owned(),
            ui::FontData::from_static(game_resources::FONT_PRESS_START),
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

    let pause = app_state.get_state_data_value::<bool>("PAUSE")
        .map(|l| *l)
        .unwrap_or(false);

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

pub fn ui_popups(context: &ui::Context, app_state: &mut AppState){
    // set font //TODO: this should be on the appstate in future versions
    context.set_fonts({
        let mut fonts = ui::FontDefinitions::default();
        fonts.font_data.insert(
            "press_start".to_owned(),
            ui::FontData::from_static(game_resources::FONT_PRESS_START),
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
}

pub fn update_ui_timers(app_state: &mut AppState) {
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