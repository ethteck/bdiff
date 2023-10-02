use crate::map_file::MapFile;
use anyhow::Error;
use eframe::egui;

#[derive(Default)]
pub struct MapTool {
    pub show: bool,
    pub last_status: Option<Error>,
    pub map_file: Option<MapFile>,
}

impl MapTool {
    pub fn display(&mut self, ui: &mut egui::Ui) {
        if !self.show {
            return;
        }

        ui.group(|ui| {
            ui.with_layout(egui::Layout::top_down(eframe::emath::Align::Min), |ui| {
                ui.add(egui::Label::new(
                    egui::RichText::new("Map File").monospace(),
                ));

                ui.label(match self.map_file {
                    Some(ref map_file) => format!(
                        "Loaded {:} ({:} symbols)",
                        map_file
                            .path
                            .as_path()
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap(),
                        map_file.data.len()
                    ),
                    None => "No map file loaded".to_owned(),
                });

                ui.with_layout(
                    egui::Layout::left_to_right(eframe::emath::Align::Min),
                    |ui| {
                        if ui
                            .button(match self.map_file {
                                Some(_) => "Load new",
                                None => "Load",
                            })
                            .clicked()
                        {
                            if let Some(path) = rfd::FileDialog::new().pick_file() {
                                self.load_file(&path);
                            }
                        }

                        if self.map_file.is_some() && ui.button("Unload").clicked() {
                            self.map_file = None;
                        }
                    },
                );
            });
        });
    }

    pub fn load_file(&mut self, path: &std::path::Path) {
        let mf = MapFile::from_path(path.to_owned());

        match mf {
            Ok(map_file) => {
                self.map_file = Some(map_file);
            }
            Err(e) => {
                self.map_file = None;
                self.last_status = Some(e);
            }
        }
    }
}
