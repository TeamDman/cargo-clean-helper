// src/app.rs

use crate::crawler;
use eframe::egui;
use egui_extras::{Column, Size, StripBuilder, TableBuilder};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use tokio::runtime::Handle;

const DONE_SIGNAL: &str = "[DONE]";

pub struct MyApp {
    root_paths_text: String,
    subdirs_data: Vec<String>,

    /// Patterns for directories we want to ignore (.git, etc.)
    ignore_patterns: Vec<String>,
    /// A temporary buffer where the user types a new pattern before "Add pattern"
    new_ignore_pattern: String,

    search_text: String,
    search_results_data: Vec<String>,

    tx: Sender<String>,
    rx: Receiver<String>,
    indexing_in_progress: bool,
    rt_handle: Handle,
}

impl MyApp {
    pub fn new(rt_handle: Handle) -> Self {
        let (tx, rx) = mpsc::channel();

        Self {
            root_paths_text: "D:\\Repos\nG:\\ml\nG:\\Repos".to_owned(),
            subdirs_data: Vec::new(),
            ignore_patterns: vec![".git".to_string()], // default ignoring .git
            new_ignore_pattern: String::new(),
            search_text: String::new(),
            search_results_data: Vec::new(),
            tx,
            rx,
            indexing_in_progress: false,
            rt_handle,
        }
    }

    fn refresh_subdirs(&mut self) {
        self.subdirs_data.clear();
        self.indexing_in_progress = true;

        let lines: Vec<String> = self
            .root_paths_text
            .lines()
            .map(|line| line.trim().to_owned())
            .filter(|line| !line.is_empty())
            .collect();

        let tx_clone = self.tx.clone();
        // Clone ignore_patterns so we can move it into the blocking task:
        let ignore_patterns_clone = self.ignore_patterns.clone();

        self.rt_handle.spawn(async move {
            tokio::task::spawn_blocking(move || {
                for root in lines {
                    crawler::gather_descendant_dirs_streaming(&root, &tx_clone, &ignore_patterns_clone);
                }
                let _ = tx_clone.send(DONE_SIGNAL.to_owned());
            })
            .await
            .ok();
        });
    }

    fn run_search(&mut self) {
        let needle = self.search_text.trim().to_lowercase();
        let results = self
            .subdirs_data
            .iter()
            .filter(|subdir| subdir.to_lowercase().contains(&needle))
            .cloned()
            .collect();
        self.search_results_data = results;
    }

    /// Adds a new pattern from `new_ignore_pattern` if it's not empty.
    fn add_ignore_pattern(&mut self) {
        let pattern = self.new_ignore_pattern.trim();
        if !pattern.is_empty() {
            self.ignore_patterns.push(pattern.to_owned());
        }
        // Clear the text input
        self.new_ignore_pattern.clear();
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1) Pull from channel:
        while let Ok(msg) = self.rx.try_recv() {
            if msg == DONE_SIGNAL {
                self.indexing_in_progress = false;
            } else {
                self.subdirs_data.push(msg);
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My Stream Viewer");
            ui.separator();

            // We'll use four columns this time:
            // 1) Root paths
            // 2) Ignore patterns
            // 3) Subdirs
            // 4) Search results
            StripBuilder::new(ui)
                .size(Size::relative(0.25))
                .size(Size::relative(0.25))
                .size(Size::relative(0.25))
                .size(Size::remainder())
                .horizontal(|mut strip| {
                    // --- Column 1: Root paths ---
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

                        if ui.button("Copy roots to clipboard").clicked() {
                            ui.ctx().copy_text(self.root_paths_text.clone());
                        }

                        let refresh_btn = ui.add_enabled(
                            !self.indexing_in_progress,
                            egui::Button::new("Refresh subdirs"),
                        );
                        if refresh_btn.clicked() {
                            self.refresh_subdirs();
                        }

                        if self.indexing_in_progress {
                            ui.colored_label(egui::Color32::YELLOW, "Indexing in progress…");
                        }
                    });

                    // --- Column 2: Ignore patterns ---
                    strip.cell(|ui| {
                        ui.label("Ignore patterns (prunes entire sub-tree)");
                        ui.horizontal(|ui| {
                            // Text box for new pattern
                            ui.text_edit_singleline(&mut self.new_ignore_pattern);
                            if ui.button("Add").clicked() {
                                self.add_ignore_pattern();
                            }
                        });

                        // Show each pattern with a minus button to remove it
                        egui::ScrollArea::vertical()
                            .id_salt("ignore_scroll")
                            .max_height(200.0)
                            .show(ui, |ui| {
                                let mut remove_indices = Vec::new();
                                for (i, pat) in self.ignore_patterns.iter().enumerate() {
                                    ui.horizontal(|ui| {
                                        ui.label(pat);
                                        if ui.button("-").clicked() {
                                            remove_indices.push(i);
                                        }
                                    });
                                }

                                // Actually remove patterns after we finish iteration:
                                for &i in remove_indices.iter().rev() {
                                    self.ignore_patterns.remove(i);
                                }
                            });
                    });

                    // --- Column 3: Subdirs Table ---
                    strip.cell(|ui| {
                        ui.label(format!("Subdirs ({} entries)", self.subdirs_data.len()));

                        egui::ScrollArea::vertical()
                            .id_salt("subdirs_scroll")
                            .max_height(300.0)
                            .show(ui, |ui| {
                                let row_height = ui.text_style_height(&egui::TextStyle::Body);
                                TableBuilder::new(ui)
                                    .striped(true)
                                    .resizable(true)
                                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                                    .column(Column::auto())      // index
                                    .column(Column::remainder()) // path
                                    .body(|mut body| {
                                        for (i, subdir) in self.subdirs_data.iter().enumerate() {
                                            body.row(row_height, |mut row| {
                                                row.col(|ui| {
                                                    ui.label(i.to_string());
                                                });
                                                row.col(|ui| {
                                                    ui.label(subdir);
                                                });
                                            });
                                        }
                                    });
                            });

                        if ui.button("Copy subdirs to clipboard").clicked() {
                            let text = self.subdirs_data.join("\n");
                            ui.ctx().copy_text(text);
                        }
                    });

                    // --- Column 4: Search & Results Table ---
                    strip.cell(|ui| {
                        ui.label("Search:");
                        ui.text_edit_singleline(&mut self.search_text);

                        if ui.button("Run search").clicked() {
                            self.run_search();
                        }

                        ui.separator();
                        ui.label(format!(
                            "Results ({} entries)",
                            self.search_results_data.len()
                        ));

                        egui::ScrollArea::vertical()
                            .id_salt("search_scroll")
                            .max_height(300.0)
                            .show(ui, |ui| {
                                let row_height = ui.text_style_height(&egui::TextStyle::Body);
                                TableBuilder::new(ui)
                                    .striped(true)
                                    .resizable(true)
                                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                                    .column(Column::auto())
                                    .column(Column::remainder())
                                    .body(|mut body| {
                                        for (i, result) in
                                            self.search_results_data.iter().enumerate()
                                        {
                                            body.row(row_height, |mut row| {
                                                row.col(|ui| {
                                                    ui.label(i.to_string());
                                                });
                                                row.col(|ui| {
                                                    ui.label(result);
                                                });
                                            });
                                        }
                                    });
                            });

                        if ui.button("Copy results to clipboard").clicked() {
                            let text = self.search_results_data.join("\n");
                            ui.ctx().copy_text(text);
                        }
                    });
                });
        });

        // If you want continuous redraw while indexing:
        if self.indexing_in_progress {
            ctx.request_repaint();
        }
    }
}
