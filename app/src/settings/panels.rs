//! Common panels and controls found in the settings window.

use crate::settings::theme::Color;
use crate::settings::SettingsControl;
use eframe::egui::{Align, Direction, Layout, RichText, Ui, Vec2};

/// Adds bottom commands allowing control for saving contents to the window UI.
///
/// This function adds a horizontal layout containing buttons for common save controls:
/// * `Restore Defaults` - Resets the settings to their default values and saves them
/// * `Reload` - Reloads the settings from the JSON file
/// * `Save` - Saves the current settings to the JSON file
///
/// # Arguments
///
/// * `ui` - A mutable reference to the `Ui` object where the commands will be added.
/// * `settings` - A mutable reference to the `Settings` object that will be modified by the commands.
pub fn window_bottom_commands(ui: &mut Ui, settings: &mut impl SettingsControl) {
    ui.separator();
    ui.horizontal(|ui| {
        if ui
            .button(RichText::new(format!(
                "{} Restore Defaults",
                egui_phosphor::regular::ARROW_COUNTER_CLOCKWISE
            )))
            .clicked()
        {
            settings.restore_defaults();
            settings.toggle_menu_visibility();
        }
        if ui
            .button(RichText::new(format!(
                "{} Reload",
                egui_phosphor::regular::FLOPPY_DISK
            )))
            .clicked()
        {
            settings.reload();
            settings.toggle_menu_visibility();
        }
        ui.separator();

        ui.horizontal(|ui| {
            ui.with_layout(Layout::right_to_left(Align::RIGHT), |ui| {
                if ui.button("Apply").clicked() {
                    settings.save();
                }
                if ui.button("Save & Close").clicked() {
                    settings.save();
                    settings.toggle_menu_visibility();
                }
            });
        });
    });
}

/// Aligns the label to the left and the color edit button to the right within a horizontal layout.
///
/// This function creates a horizontal layout where the label is left-aligned and the color edit button
/// is right-aligned. The color edit button allows the user to edit the color value.
///
/// # Arguments
///
/// * `ui` - A mutable reference to the `Ui` object where the elements will be added.
/// * `label` - A string slice that holds the label text.
/// * `color` - A mutable reference to the `Color` object that will be modified by the color edit button.
pub fn color_selection(ui: &mut Ui, label: &str, color: &mut Color) {
    ui.allocate_ui_with_layout(
        Vec2::new(200.0, 20.0),
        Layout::centered_and_justified(Direction::LeftToRight).with_main_justify(true),
        |ui| {
            ui.horizontal(|ui| {
                ui.label(label);
                ui.with_layout(Layout::right_to_left(Align::RIGHT), |ui| {
                    ui.color_edit_button_srgba_premultiplied(color.as_bytes_mut());
                });
            });
        },
    );
}
