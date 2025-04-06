use crate::crawler;
use eframe::egui;

pub struct MyApp {
    // The user’s text input for the root paths (one per line).
    root_paths_text: String,

    // The list of all subdirectories discovered (line-separated).
    subdirs_text: String,

    // The search input at the top of the third column.
    search_text: String,

    // The final text we show for the third column (filtered).
    search_results_text: String,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            // Provide default values (the three paths):
            root_paths_text: "D:\\Repos\nG:\\ml\nG:\\Repos".to_owned(),
            subdirs_text: String::new(),
            search_text: String::new(),
            search_results_text: String::new(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My Stream Viewer");
            ui.separator();

            // We can layout in columns:
            egui::Grid::new("three_column_layout")
                .num_columns(3)
                .spacing([20.0, 10.0])
                .show(ui, |ui| {
                    // --- Column 1: Root paths ---
                    ui.vertical(|ui| {
                        ui.label(format!(
                            "Roots ({} entries)",
                            self.root_paths_text.lines().count()
                        ));
                        egui::ScrollArea::vertical()
                            .max_height(200.0)
                            .show(ui, |ui| {
                                ui.text_edit_multiline(&mut self.root_paths_text);
                            });
                        if ui.button("Copy to clipboard").clicked() {
                            ui.ctx().copy_text(self.root_paths_text.clone());
                        }
                        if ui.button("Refresh subdirs").clicked() {
                            self.refresh_subdirs();
                        }
                    });
                    ui.end_row();

                    // --- Column 2: Subdirectories ---
                    ui.vertical(|ui| {
                        ui.label(format!(
                            "Subdirs ({} entries)",
                            self.subdirs_text.lines().count()
                        ));
                        egui::ScrollArea::vertical()
                            .max_height(200.0)
                            .show(ui, |ui| {
                                ui.text_edit_multiline(&mut self.subdirs_text);
                            });
                        if ui.button("Copy to clipboard").clicked() {
                            ui.ctx().copy_text(self.subdirs_text.clone());
                        }
                    });
                    ui.end_row();

                    // --- Column 3: Search / results ---
                    ui.vertical(|ui| {
                        // A small label and then a text box for searching:
                        ui.label("Search:");
                        ui.text_edit_singleline(&mut self.search_text);

                        if ui.button("Run search").clicked() {
                            self.run_search();
                        }

                        ui.separator();
                        ui.label(format!(
                            "Results ({} entries)",
                            self.search_results_text.lines().count()
                        ));
                        egui::ScrollArea::vertical()
                            .max_height(200.0)
                            .show(ui, |ui| {
                                ui.text_edit_multiline(&mut self.search_results_text);
                            });
                        if ui.button("Copy to clipboard").clicked() {
                            ui.ctx().copy_text(self.search_results_text.clone());
                        }
                    });
                    ui.end_row();
                });
        });
    }
}

impl MyApp {
    fn refresh_subdirs(&mut self) {
        // Clear existing results
        self.subdirs_text.clear();

        // For each root path line, gather subdirectories:
        for line in self.root_paths_text.lines() {
            let line = line.trim();
            if !line.is_empty() {
                let subdirs = crawler::gather_descendant_dirs(line);
                for path in subdirs {
                    // Add to our subdirs_text
                    self.subdirs_text.push_str(&path);
                    self.subdirs_text.push('\n');
                }
            }
        }
    }

    fn run_search(&mut self) {
        // For now, we do a simple .contains search on the subdirs_text lines:
        let needle = self.search_text.trim().to_lowercase();
        let mut results = Vec::new();

        for line in self.subdirs_text.lines() {
            if line.to_lowercase().contains(&needle) {
                results.push(line);
            }
        }

        self.search_results_text = results.join("\n");
    }
}
