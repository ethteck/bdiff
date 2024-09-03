use eframe::egui;
use encoding_rs::*;

use crate::bin_file::Endianness;

pub struct StringViewer {
    pub show: bool,
    pub utf8: bool,
    pub utf16: bool,
    pub shift_jis: bool,
    pub euc_jp: bool,
}

impl Default for StringViewer {
    fn default() -> StringViewer {
        StringViewer {
            show: false,
            utf8: true,
            utf16: false,
            shift_jis: false,
            euc_jp: false,
        }
    }
}

impl StringViewer {
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
            ui.with_layout(
                egui::Layout::left_to_right(eframe::emath::Align::Min),
                |ui| {
                    ui.add(egui::Label::new(
                        egui::RichText::new("String Viewer").monospace(),
                    ));

                    ui.menu_button("...", |ui| {
                        ui.checkbox(&mut self.utf8, "UTF-8");
                        ui.checkbox(&mut self.utf16, "UTF-16");
                        ui.checkbox(&mut self.euc_jp, "EUC-JP");
                        ui.checkbox(&mut self.shift_jis, "Shift JIS");
                    });
                },
            );

            egui::Grid::new(format!("string_grid{}", hv_id))
                .striped(true)
                .num_columns(2)
                .show(ui, |ui| {
                    if self.utf8 {
                        ui.add(egui::Label::new(egui::RichText::new("UTF-8").monospace()));
                        ui.text_edit_singleline(
                            &mut String::from_utf8_lossy(selected_bytes).into_owned(),
                        );
                        ui.end_row();
                    }

                    if self.utf16 {
                        let encoding = match endianness {
                            Endianness::Little => UTF_16LE,
                            Endianness::Big => UTF_16BE,
                        };

                        ui.add(egui::Label::new(egui::RichText::new("UTF-16").monospace()));
                        ui.text_edit_singleline(
                            &mut encoding
                                .decode_without_bom_handling_and_without_replacement(
                                    &selected_bytes,
                                )
                                .unwrap_or_default()
                                .to_string(),
                        );
                        ui.end_row();
                    }

                    if self.euc_jp {
                        ui.add(egui::Label::new(egui::RichText::new("EUC-JP").monospace()));
                        ui.text_edit_singleline(
                            &mut EUC_JP
                                .decode_without_bom_handling_and_without_replacement(
                                    &selected_bytes,
                                )
                                .unwrap_or_default()
                                .to_string(),
                        );
                        ui.end_row();
                    }

                    if self.shift_jis {
                        ui.add(egui::Label::new(
                            egui::RichText::new("Shift JIS").monospace(),
                        ));
                        ui.text_edit_singleline(
                            &mut SHIFT_JIS
                                .decode_without_bom_handling_and_without_replacement(
                                    &selected_bytes,
                                )
                                .unwrap_or_default()
                                .to_string(),
                        );
                        ui.end_row();
                    }
                });
        });
    }
}
