mod app;
mod bin_file;
mod config;
mod data_viewer;
mod hex_view;
mod map_file;
mod map_tool;
mod spacer;
mod string_viewer;
mod watcher;

use std::path::PathBuf;

use app::BdiffApp;
use argh::FromArgs;
use eframe::IconData;

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
        icon_data: Some(
            IconData::try_from_png_bytes(include_bytes!("../assets/icon.png")).unwrap(),
        ),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "bdiff",
        native_options,
        Box::new(|cc| Box::new(BdiffApp::new(cc, args.files))),
    );
}
