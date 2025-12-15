#![cfg_attr(
    // hide console window on Windows in release
    all(target_family = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

use anyhow::Result;
use app::{App, CONFIG_PATH_STORAGE_KEY};
use eframe::egui;

mod app;
mod errors;
mod widgets;

const GUI_SCALE: f32 = 1.5;

pub fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([730.0, 420.0]),
        ..Default::default()
    };
    eframe::run_native(
        "codeforces-tester",
        options,
        Box::new(|ctx| {
            ctx.egui_ctx.set_pixels_per_point(GUI_SCALE);
            let config_path = ctx
                .storage
                .and_then(|s| s.get_string(CONFIG_PATH_STORAGE_KEY))
                .map(Into::into);
            Ok(Box::new(App::new(config_path)))
        }),
    )
}
