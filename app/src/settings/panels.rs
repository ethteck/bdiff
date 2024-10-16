//! Common panels and controls found in the settings window.

use crate::settings::SettingsControl;
use bdiff_hex_view::theme::Color;
use eframe::egui::{Align, Direction, Layout, RichText, Ui, Vec2};

pub fn settings_management_buttons(ui: &mut Ui, settings: &mut impl SettingsControl) {
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
        }
        if ui
            .button(RichText::new(format!(
                "{} Reload",
                egui_phosphor::regular::FLOPPY_DISK
            )))
            .clicked()
        {
            settings.reload();
        }
        ui.separator();

        if ui.button("Save").clicked() {
            settings.save();
        }
    });
}

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
