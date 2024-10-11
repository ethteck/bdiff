#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
mod bin_file;
mod config;
mod data_viewer;
mod file_view;
mod map_file;
mod settings;
mod string_viewer;
mod symbol_tool;
mod watcher;

use std::path::PathBuf;

use app::BdiffApp;
use argh::FromArgs;
use eframe::{egui::ViewportBuilder, icon_data};

#[derive(FromArgs)]
/// binary differ
struct Args {
    /// input files
    #[argh(positional)]
    files: Vec<PathBuf>,
}

fn main() {
    let args: Args = argh::from_env();

    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_icon(icon_data::from_png_bytes(include_bytes!("../assets/icon.png")).unwrap()),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "bdiff",
        native_options,
        Box::new(|cc| Ok(Box::new(BdiffApp::new(cc, args.files)))),
    );
}
