use std::path::Path;

use iced::window::{self, icon};
pub use iced::window::{close, Settings};

pub fn settings() -> Settings {
    let size: (u32, u32) = (1000, 1300);
    let icon: Result<window::Icon, icon::Error> = load_icon();

    match icon {
        Ok(icon) => Settings {
            size,
            icon: Some(icon),
            ..Default::default()
        },
        Err(_) => Settings {
            size,
            ..Default::default()
        },
    }
}

fn load_icon() -> Result<icon::Icon, icon::Error> {
    let icon_path = Path::new("assets/icon.png");
    let icon = icon::from_file(icon_path)?;

    Ok(icon)
}
