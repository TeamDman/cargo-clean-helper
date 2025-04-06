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

use egui_extras::Size;
use egui_extras::StripBuilder;

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My Stream Viewer");
            ui.separator();

            // Use StripBuilder to create 3 columns, each taking 1/3 of the width:
            StripBuilder::new(ui)
                .size(Size::relative(0.3333)) // column 1
                .size(Size::relative(0.3333)) // column 2
                .size(Size::remainder()) // column 3
                .horizontal(|mut strip| {
                    // --- Column 1 ---
                    strip.cell(|ui| {
                        ui.label(format!(
                            "Roots ({} entries)",
                            self.root_paths_text.lines().count()
                        ));
                        egui::ScrollArea::vertical()
                            .id_salt("roots_scroll")
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

                    // --- Column 2 ---
                    strip.cell(|ui| {
                        ui.label(format!(
                            "Subdirs ({} entries)",
                            self.subdirs_text.lines().count()
                        ));
                        egui::ScrollArea::vertical()
                            .id_salt("subdirs_scroll")
                            .max_height(200.0)
                            .show(ui, |ui| {
                                ui.text_edit_multiline(&mut self.subdirs_text);
                            });

                        if ui.button("Copy to clipboard").clicked() {
                            ui.ctx().copy_text(self.subdirs_text.clone());
                        }
                    });

                    // --- Column 3 ---
                    strip.cell(|ui| {
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
                            .id_salt("search_scroll")
                            .max_height(200.0)
                            .show(ui, |ui| {
                                ui.text_edit_multiline(&mut self.search_results_text);
                            });

                        if ui.button("Copy to clipboard").clicked() {
                            ui.ctx().copy_text(self.search_results_text.clone());
                        }
                    });
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
