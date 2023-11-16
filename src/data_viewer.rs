use eframe::egui;

use crate::bin_file::Endianness;

pub struct DataViewer {
    pub show: bool,
    pub i8: bool,
    pub u8: bool,
    pub i16: bool,
    pub u16: bool,
    pub i32: bool,
    pub u32: bool,
    pub i64: bool,
    pub u64: bool,
    pub f32: bool,
    pub f64: bool,
}

impl Default for DataViewer {
    fn default() -> DataViewer {
        DataViewer {
            show: false,
            i8: true,
            u8: true,
            i16: true,
            u16: true,
            i32: true,
            u32: true,
            i64: false,
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
    ($ui:expr, $selected_bytes:expr, $endianness:expr, $delimiter:expr, $represented_type:ty, $value:expr, $size:expr) => {
        display_type(
            $ui,
            $selected_bytes,
            $value,
            stringify!($represented_type),
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
    ($ui:expr, $selected_bytes:expr, $endianness:expr, $delimiter:expr, $represented_type:ty, $value:expr, $size:expr, $float_buffer:expr) => {
        display_type(
            $ui,
            $selected_bytes,
            $value,
            stringify!($represented_type),
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
        selected_bytes: Vec<u8>,
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
                            ui.checkbox(&mut self.i8, "i8");
                            ui.checkbox(&mut self.u8, "u8");
                            ui.checkbox(&mut self.i16, "i16");
                            ui.checkbox(&mut self.u16, "u16");
                            ui.checkbox(&mut self.i32, "i32");
                            ui.checkbox(&mut self.u32, "u32");
                            ui.checkbox(&mut self.i64, "i64");
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
        selected_bytes: Vec<u8>,
        endianness: Endianness,
    ) {
        let mut float_buffer = dtoa::Buffer::new();
        let delimiter = ", ";

        create_display_type!(ui, &selected_bytes, endianness, delimiter, i8, self.i8, 1);
        create_display_type!(ui, &selected_bytes, endianness, delimiter, u8, self.u8, 1);
        create_display_type!(ui, &selected_bytes, endianness, delimiter, i16, self.i16, 2);
        create_display_type!(ui, &selected_bytes, endianness, delimiter, u16, self.u16, 2);
        create_display_type!(ui, &selected_bytes, endianness, delimiter, i32, self.i32, 4);
        create_display_type!(ui, &selected_bytes, endianness, delimiter, u32, self.u32, 4);
        create_display_type!(ui, &selected_bytes, endianness, delimiter, i64, self.i64, 8);
        create_display_type!(ui, &selected_bytes, endianness, delimiter, u64, self.u64, 8);
        create_display_type!(
            ui,
            &selected_bytes,
            endianness,
            delimiter,
            f32,
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
            self.f64,
            8,
            float_buffer
        );
    }
}
