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

fn display_type(
    ui: &mut egui::Ui,
    bytes: &[u8],
    enabled: bool,
    name: impl Into<String>,
    size: usize,
    func: impl FnMut(&[u8]) -> String,
    delimiter: &str,
) {
    if enabled {
        ui.add(egui::Label::new(egui::RichText::new(name).monospace()));
        let mut data = bytes
            .chunks_exact(size)
            .take(100) // prevent too many snibblets
            .map(func)
            .collect::<Vec<String>>()
            .join(delimiter);

        ui.text_edit_singleline(&mut data);
        ui.end_row();
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
                    .show(ui, |ui| self.display_data_types(ui, selected_bytes));
            });
        });
    }

    fn display_data_types(&mut self, ui: &mut egui::Ui, selected_bytes: Vec<u8>) {
        let mut float_buffer = dtoa::Buffer::new();
        let delimiter = ", ";

        display_type(
            ui,
            &selected_bytes,
            self.s8,
            "s8",
            1,
            |chunk| {
                format!(
                    "{:}",
                    i8::from_be_bytes(chunk.try_into().unwrap_or_default())
                )
            },
            delimiter,
        );

        display_type(
            ui,
            &selected_bytes,
            self.u8,
            "u8",
            1,
            |chunk| {
                format!(
                    "{:}",
                    u8::from_be_bytes(chunk.try_into().unwrap_or_default())
                )
            },
            delimiter,
        );

        display_type(
            ui,
            &selected_bytes,
            self.s16,
            "s16",
            2,
            |chunk| {
                format!(
                    "{:}",
                    i16::from_be_bytes(chunk.try_into().unwrap_or_default())
                )
            },
            delimiter,
        );

        display_type(
            ui,
            &selected_bytes,
            self.u16,
            "u16",
            2,
            |chunk| {
                format!(
                    "{:}",
                    u16::from_be_bytes(chunk.try_into().unwrap_or_default())
                )
            },
            delimiter,
        );

        display_type(
            ui,
            &selected_bytes,
            self.s32,
            "s32",
            4,
            |chunk| {
                format!(
                    "{:}",
                    i32::from_be_bytes(chunk.try_into().unwrap_or_default())
                )
            },
            delimiter,
        );

        display_type(
            ui,
            &selected_bytes,
            self.u32,
            "u32",
            4,
            |chunk| {
                format!(
                    "{:}",
                    u32::from_be_bytes(chunk.try_into().unwrap_or_default())
                )
            },
            delimiter,
        );

        display_type(
            ui,
            &selected_bytes,
            self.f32,
            "f32",
            4,
            |chunk| {
                float_buffer
                    .format(f32::from_be_bytes(chunk.try_into().unwrap_or_default()))
                    .to_string()
            },
            delimiter,
        );

        display_type(
            ui,
            &selected_bytes,
            self.s64,
            "s64",
            8,
            |chunk| {
                format!(
                    "{:}",
                    i64::from_be_bytes(chunk.try_into().unwrap_or_default())
                )
            },
            delimiter,
        );

        display_type(
            ui,
            &selected_bytes,
            self.u64,
            "u64",
            8,
            |chunk| {
                format!(
                    "{:}",
                    u64::from_be_bytes(chunk.try_into().unwrap_or_default())
                )
            },
            delimiter,
        );

        display_type(
            ui,
            &selected_bytes,
            self.f64,
            "f64",
            8,
            |chunk| {
                float_buffer
                    .format(f64::from_be_bytes(chunk.try_into().unwrap_or_default()))
                    .to_string()
            },
            delimiter,
        );
    }
}
