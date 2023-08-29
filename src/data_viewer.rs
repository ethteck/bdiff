use eframe::egui;

pub struct DataViewer {
    pub show: bool,
    pub s8: bool,
    pub u8: bool,
    pub s16: bool,
    pub u16: bool,
    pub s32: bool,
    pub u32: bool,
    pub s64: bool,
    pub u64: bool,
    pub f32: bool,
    pub f64: bool,
}

impl Default for DataViewer {
    fn default() -> DataViewer {
        DataViewer {
            show: false,
            s8: true,
            u8: true,
            s16: true,
            u16: true,
            s32: true,
            u32: true,
            s64: false,
            u64: false,
            f32: true,
            f64: true,
        }
    }
}

impl DataViewer {
    pub fn display(&mut self, ui: &mut egui::Ui, hv_id: usize, selected_bytes: Vec<u8>) {
        if !self.show {
            return;
        }

        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.with_layout(
                    egui::Layout::left_to_right(eframe::emath::Align::Min),
                    |ui| {
                        ui.add(egui::Label::new(
                            egui::RichText::new("Data Viewer").monospace(),
                        ));

                        ui.menu_button("...", |ui| {
                            ui.checkbox(&mut self.s8, "s8");
                            ui.checkbox(&mut self.u8, "u8");
                            ui.checkbox(&mut self.s16, "s16");
                            ui.checkbox(&mut self.u16, "u16");
                            ui.checkbox(&mut self.s32, "s32");
                            ui.checkbox(&mut self.u32, "u32");
                            ui.checkbox(&mut self.s64, "s64");
                            ui.checkbox(&mut self.u64, "u64");
                            ui.checkbox(&mut self.f32, "f32");
                            ui.checkbox(&mut self.f64, "f64");
                        });
                    },
                );

                egui::Grid::new(format!("hex_grid_selection{}", hv_id))
                    .striped(true)
                    .num_columns(2)
                    .show(ui, |ui| {
                        let mut float_buffer = dtoa::Buffer::new();

                        let selection_len = selected_bytes.len();

                        if self.s8 {
                            self.selection_data_row(
                                ui,
                                "s8",
                                if selection_len >= 1 {
                                    format!(
                                        "{:}",
                                        i8::from_be_bytes(
                                            selected_bytes[0..1].try_into().unwrap_or_default(),
                                        )
                                    )
                                } else {
                                    "".to_owned()
                                },
                            );
                        }

                        if self.u8 {
                            self.selection_data_row(
                                ui,
                                "u8",
                                if selection_len >= 1 {
                                    format!(
                                        "{:}",
                                        u8::from_be_bytes(
                                            selected_bytes[0..1].try_into().unwrap_or_default(),
                                        )
                                    )
                                } else {
                                    "".to_owned()
                                },
                            );
                        }

                        if self.s16 {
                            self.selection_data_row(
                                ui,
                                "s16",
                                if selection_len >= 2 {
                                    format!(
                                        "{:}",
                                        i16::from_be_bytes(
                                            selected_bytes[0..2].try_into().unwrap_or_default(),
                                        )
                                    )
                                } else {
                                    "".to_owned()
                                },
                            );
                        }

                        if self.u16 {
                            self.selection_data_row(
                                ui,
                                "u16",
                                if selection_len >= 2 {
                                    format!(
                                        "{:}",
                                        u16::from_be_bytes(
                                            selected_bytes[0..2].try_into().unwrap_or_default(),
                                        )
                                    )
                                } else {
                                    "".to_owned()
                                },
                            );
                        }

                        if self.s32 {
                            self.selection_data_row(
                                ui,
                                "s32",
                                if selection_len >= 4 {
                                    format!(
                                        "{:}",
                                        i32::from_be_bytes(
                                            selected_bytes[0..4].try_into().unwrap_or_default(),
                                        )
                                    )
                                } else {
                                    "".to_owned()
                                },
                            );
                        }

                        if self.u32 {
                            self.selection_data_row(
                                ui,
                                "u32",
                                if selection_len >= 4 {
                                    format!(
                                        "{:}",
                                        u32::from_be_bytes(
                                            selected_bytes[0..4].try_into().unwrap_or_default(),
                                        )
                                    )
                                } else {
                                    "".to_owned()
                                },
                            );
                        }

                        if self.f32 {
                            self.selection_data_row(
                                ui,
                                "f32",
                                if selection_len >= 4 {
                                    float_buffer
                                        .format(f32::from_be_bytes(
                                            selected_bytes[0..4].try_into().unwrap_or_default(),
                                        ))
                                        .to_string()
                                } else {
                                    "".to_owned()
                                },
                            );
                        }

                        if self.s64 {
                            self.selection_data_row(
                                ui,
                                "s64",
                                if selection_len >= 8 {
                                    format!(
                                        "{:}",
                                        i64::from_be_bytes(
                                            selected_bytes[0..8].try_into().unwrap_or_default(),
                                        )
                                    )
                                } else {
                                    "".to_owned()
                                },
                            );
                        }

                        if self.u64 {
                            self.selection_data_row(
                                ui,
                                "u64",
                                if selection_len >= 8 {
                                    format!(
                                        "{:}",
                                        u64::from_be_bytes(
                                            selected_bytes[0..8].try_into().unwrap_or_default(),
                                        )
                                    )
                                } else {
                                    "".to_owned()
                                },
                            );
                        }

                        if self.f64 {
                            self.selection_data_row(
                                ui,
                                "f64",
                                if selection_len >= 8 {
                                    float_buffer
                                        .format(f64::from_be_bytes(
                                            selected_bytes[0..8].try_into().unwrap_or_default(),
                                        ))
                                        .to_string()
                                } else {
                                    "".to_owned()
                                },
                            );
                        }
                    });
            });
        });
    }

    fn selection_data_row(&self, ui: &mut egui::Ui, name: impl Into<String>, mut data: String) {
        ui.add(egui::Label::new(egui::RichText::new(name).monospace()));
        ui.text_edit_singleline(&mut data);
        ui.end_row();
    }
}
