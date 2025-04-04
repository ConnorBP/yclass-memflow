#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

#[cfg(all(not(unix), not(windows)))]
compile_error!("Only UNIX and Windows platforms are supported.");

#[cfg(not(target_pointer_width = "64"))]
compile_error!("Only X64 targets are supported.");

mod address;
mod app;
mod class;
mod config;
mod context;
mod field;
mod generator;
mod gui;
mod hotkeys;
mod process;
mod project;
mod state;
mod value;

use config::YClassConfig;
use eframe::{
    egui::{FontData, FontDefinitions, Key, Modifiers},
    epaint::{FontFamily, FontId},
    NativeOptions,
};
use hotkeys::HotkeyManager;
use state::GlobalState;
use std::{cell::RefCell, sync::Arc};

/// Monospaced font id.
const FID_M: FontId = FontId::monospace(16.);

fn main() {

    // logger init
    egui_logger::builder().init().unwrap();

    // start the gui app
    eframe::run_native(
        "memclass",
        NativeOptions {
            // default_theme: Theme::Dark,
            ..Default::default()
        },
        Box::new(|cc| {
            let config = YClassConfig::load_or_default();
            cc.egui_ctx.set_pixels_per_point(config.dpi.unwrap_or(1.));

            let mut fonts = FontDefinitions::default();
            fonts.font_data.insert(
                "roboto-mono".into(),
                Arc::new(FontData::from_static(include_bytes!("../fonts/RobotoMono-Regular.ttf"))),
            );
            fonts
                .families
                .get_mut(&FontFamily::Monospace)
                .unwrap()
                .insert(0, "roboto-mono".into());
            cc.egui_ctx.set_fonts(fonts);

            let mut hotkeys = HotkeyManager::default();
            hotkeys.register("process_info", Key::P, Modifiers::ALT);
            hotkeys.register("attach_memflow", Key::M, Modifiers::ALT);
            hotkeys.register("attach_process", Key::A, Modifiers::ALT);
            hotkeys.register("attach_recent", Key::A, Modifiers::ALT | Modifiers::CTRL);
            hotkeys.register("detach_process", Key::D, Modifiers::ALT);

            Ok(Box::new(app::YClassApp::new(Box::leak(Box::new(RefCell::new(
                GlobalState {
                    config,
                    hotkeys,
                    ..Default::default()
                },
            ))))))
        }),
    )
    .unwrap();
}
