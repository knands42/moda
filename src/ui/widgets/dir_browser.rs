use std::path::{Path, PathBuf};

pub struct DirBrowser {
    pub visible: bool,
    current_dir: PathBuf,
    entries: Vec<PathBuf>,
    error: Option<String>,
}

impl Default for DirBrowser {
    fn default() -> Self {
        Self::new()
    }
}

impl DirBrowser {
    pub fn new() -> Self {
        let start = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        let mut browser = Self {
            visible: false,
            current_dir: start,
            entries: Vec::new(),
            error: None,
        };
        browser.refresh();
        browser
    }

    fn refresh(&mut self) {
        self.entries = match std::fs::read_dir(&self.current_dir) {
            Ok(rd) => rd
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                .map(|e| e.path())
                .collect(),
            Err(e) => {
                self.error = Some(format!("Cannot read directory: {}", e));
                return;
            }
        };
        self.entries.sort();
        self.error = None;
    }

    pub fn show(&mut self, ctx: &egui::Context, on_select: &mut Option<PathBuf>) {
        let mut open = true;
        egui::Window::new("Select Game Directory")
            .open(&mut open)
            .resizable(true)
            .default_size([500.0, 400.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                self.ui_content(ui, on_select);
            });
        if !open {
            self.visible = false;
        }
    }

    fn ui_content(&mut self, ui: &mut egui::Ui, on_select: &mut Option<PathBuf>) {
        ui.vertical(|ui| {
            ui.label(
                egui::RichText::new("Choose the game installation folder")
                    .size(14.0)
                    .color(super::super::style::TEXT_MUTED),
            );
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                let path_text = self.current_dir.to_string_lossy().to_string();
                let mut edit = path_text.clone();
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut edit)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY),
                );
                if resp.changed() && !edit.is_empty() {
                    let new_path = PathBuf::from(&edit);
                    if new_path.exists() && new_path.is_dir() {
                        self.current_dir = new_path;
                        self.refresh();
                    }
                }
            });
            ui.add_space(4.0);

            if let Some(ref err) = self.error {
                ui.label(
                    egui::RichText::new(err)
                        .color(super::super::style::ERROR)
                        .size(12.0),
                );
                ui.add_space(4.0);
            }

            let avail = ui.available_height();
            egui::ScrollArea::vertical()
                .max_height(avail - 40.0)
                .show(ui, |ui| {
                    let up_btn =
                        egui::Button::selectable(false, "..").fill(super::super::style::CARD_BG);
                    if ui.add(up_btn).clicked() {
                        self.navigate_up();
                    }

                    ui.separator();

                    let entries_copy = self.entries.clone();
                    for entry in &entries_copy {
                        let name = entry
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        if name.starts_with('.') {
                            continue;
                        }

                        let label = egui::Label::new(
                            egui::RichText::new(format!("\u{1F4C1}  {}", name)).size(13.0),
                        );
                        let resp = ui.add(label).interact(egui::Sense::click());
                        if resp.double_clicked() {
                            self.navigate_to(entry);
                        }
                    }
                });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui
                    .button(egui::RichText::new("Cancel").size(13.0))
                    .clicked()
                {
                    self.visible = false;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let select_btn = egui::Button::new(
                        egui::RichText::new("Select This Directory")
                            .size(13.0)
                            .color(egui::Color32::WHITE),
                    )
                    .fill(super::super::style::ACCENT)
                    .min_size(egui::vec2(160.0, 30.0));
                    if ui.add(select_btn).clicked() {
                        *on_select = Some(self.current_dir.clone());
                        self.visible = false;
                    }
                });
            });
        });
    }

    fn navigate_to(&mut self, path: &Path) {
        if path.is_dir() {
            self.current_dir = path.to_path_buf();
            self.refresh();
        }
    }

    fn navigate_up(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.refresh();
        }
    }
}
