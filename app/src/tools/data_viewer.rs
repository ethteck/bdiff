use eframe::egui;

use crate::bin_file::Endianness;

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

macro_rules! create_display_type {
    ($ui:expr, $selected_bytes:expr, $endianness:expr, $delimiter:expr, $represented_type:ty, $type_name:literal, $value:expr, $size:expr) => {
        display_type(
            $ui,
            $selected_bytes,
            $value,
            $type_name,
            $size,
            |chunk| {
                format!(
                    "{:}",
                    match $endianness {
                        Endianness::Little =>
                            <$represented_type>::from_le_bytes(chunk.try_into().unwrap_or_default()),
                        Endianness::Big =>
                            <$represented_type>::from_be_bytes(chunk.try_into().unwrap_or_default()),
                    }
                )
            },
            $delimiter,
        );
    };
    ($ui:expr, $selected_bytes:expr, $endianness:expr, $delimiter:expr, $represented_type:ty, $type_name:literal, $value:expr, $size:expr, $float_buffer:expr) => {
        display_type(
            $ui,
            $selected_bytes,
            $value,
            $type_name,
            $size,
            |chunk| {
                $float_buffer
                    .format(match $endianness {
                        Endianness::Little => {
                            f64::from_le_bytes(chunk.try_into().unwrap_or_default())
                        }
                        Endianness::Big => f64::from_be_bytes(chunk.try_into().unwrap_or_default()),
                    })
                    .to_string()
            },
            $delimiter,
        );
    };
}

impl DataViewer {
    pub fn display(
        &mut self,
        ui: &mut egui::Ui,
        hv_id: usize,
        selected_bytes: &[u8],
        endianness: Endianness,
    ) {
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
                        self.display_data_types(ui, selected_bytes, endianness)
                    });
            });
        });
    }

    fn display_data_types(
        &mut self,
        ui: &mut egui::Ui,
        selected_bytes: &[u8],
        endianness: Endianness,
    ) {
        let mut float_buffer = dtoa::Buffer::new();
        let delimiter = ", ";

        create_display_type!(
            ui,
            &selected_bytes,
            endianness,
            delimiter,
            i8,
            "s8",
            self.s8,
            1
        );
        create_display_type!(
            ui,
            &selected_bytes,
            endianness,
            delimiter,
            u8,
            "u8",
            self.u8,
            1
        );
        create_display_type!(
            ui,
            &selected_bytes,
            endianness,
            delimiter,
            i16,
            "s16",
            self.s16,
            2
        );
        create_display_type!(
            ui,
            &selected_bytes,
            endianness,
            delimiter,
            u16,
            "u16",
            self.u16,
            2
        );
        create_display_type!(
            ui,
            &selected_bytes,
            endianness,
            delimiter,
            i32,
            "s32",
            self.s32,
            4
        );
        create_display_type!(
            ui,
            &selected_bytes,
            endianness,
            delimiter,
            u32,
            "u32",
            self.u32,
            4
        );
        create_display_type!(
            ui,
            &selected_bytes,
            endianness,
            delimiter,
            i64,
            "s64",
            self.s64,
            8
        );
        create_display_type!(
            ui,
            &selected_bytes,
            endianness,
            delimiter,
            u64,
            "u64",
            self.u64,
            8
        );
        create_display_type!(
            ui,
            &selected_bytes,
            endianness,
            delimiter,
            f32,
            "f32",
            self.f32,
            4,
            float_buffer
        );
        create_display_type!(
            ui,
            &selected_bytes,
            endianness,
            delimiter,
            f64,
            "f64",
            self.f64,
            8,
            float_buffer
        );
    }
}
