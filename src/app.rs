// src/app.rs

use crate::crawler;
use eframe::egui;
use egui_extras::Column;
use egui_extras::TableBuilder;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use tokio::runtime::Handle;

// We'll do a small chunked read so you see directories appear live!
const DONE_SIGNAL: &str = "[DONE]";

pub struct MyApp {
    root_paths_text: String,
    subdirs_text: String,
    search_text: String,
    search_results_text: String,

    // For incremental indexing:
    tx: Sender<String>,
    rx: Receiver<String>,
    indexing_in_progress: bool,

    // We'll hold a handle to the runtime so we can spawn tasks.
    rt_handle: Handle,

    // New for ignore patterns:
    ignore_patterns: Vec<String>,
    new_pattern_input: String,
}

impl MyApp {
    pub fn new(rt_handle: Handle) -> Self {
        let (tx, rx) = mpsc::channel();

        Self {
            root_paths_text: "D:\\Repos\nG:\\ml\nG:\\Repos".to_owned(),
            subdirs_text: String::new(),
            search_text: String::new(),
            search_results_text: String::new(),
            tx,
            rx,
            indexing_in_progress: false,
            rt_handle,
            ignore_patterns: vec![".git".to_owned()],
            new_pattern_input: String::new(),
        }
    }

    /// Called when user clicks "Refresh subdirs"
    fn refresh_subdirs(&mut self) {
        self.subdirs_text.clear();
        self.indexing_in_progress = true;

        let lines: Vec<String> = self
            .root_paths_text
            .lines()
            .map(|line| line.trim().to_owned())
            .filter(|line| !line.is_empty())
            .collect();

        // Copy current ignore patterns into local variable for the background thread
        let ignore_list = self.ignore_patterns.clone();
        let tx_clone = self.tx.clone();

        self.rt_handle.spawn(async move {
            tokio::task::spawn_blocking(move || {
                for root in lines {
                    crawler::gather_descendant_dirs_streaming(&root, &tx_clone, &ignore_list);
                }
                let _ = tx_clone.send(DONE_SIGNAL.to_owned());
            })
            .await
            .ok();
        });
    }

    fn run_search(&mut self) {
        let needle = self.search_text.trim().to_lowercase();
        let mut results = Vec::new();

        for line in self.subdirs_text.lines() {
            if line.to_lowercase().contains(&needle) {
                results.push(line.to_owned());
            }
        }

        self.search_results_text = results.join("\n");
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1) Read from channel
        let mut new_messages = vec![];
        while let Ok(msg) = self.rx.try_recv() {
            new_messages.push(msg);
        }

        // 2) Apply them
        for msg in new_messages {
            if msg == DONE_SIGNAL {
                self.indexing_in_progress = false;
            } else {
                self.subdirs_text.push_str(&msg);
                self.subdirs_text.push('\n');
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Cargo Clean Helper");
            ui.separator();

            TableBuilder::new(ui)
                .resizable(true)
                .striped(true)
                .column(Column::remainder().at_least(150.0)) // "Roots"
                .column(Column::remainder().at_least(150.0)) // "Subdirs"
                .column(Column::remainder().at_least(150.0)) // "Search"
                .column(Column::remainder().at_least(150.0)) // "Ignore Patterns"
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("Roots");
                    });
                    header.col(|ui| {
                        ui.strong("Ignore Patterns");
                    });
                    header.col(|ui| {
                        ui.strong("Subdirs");
                    });
                    header.col(|ui| {
                        ui.strong("Search");
                    });
                })
                .body(|mut body| {
                    // We only need a single row here, each cell is its own vertical chunk:
                    body.row(0.0, |mut row| {
                        // --- Roots Column ---
                        row.col(|ui| {
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

                            let refresh_btn = ui.add_enabled(
                                !self.indexing_in_progress,
                                egui::Button::new("Refresh subdirs"),
                            );
                            if refresh_btn.clicked() {
                                self.refresh_subdirs();
                            }

                            if self.indexing_in_progress {
                                ui.label("Indexing in progress…");
                            }
                        });

                        // --- Ignore Patterns Column ---
                        row.col(|ui| {
                            ui.label("Add new ignore pattern:");
                            ui.horizontal(|ui| {
                                ui.text_edit_singleline(&mut self.new_pattern_input);
                                if ui.button("Add").clicked() {
                                    if !self.new_pattern_input.trim().is_empty() {
                                        self.ignore_patterns
                                            .push(self.new_pattern_input.trim().to_string());
                                        self.new_pattern_input.clear();
                                    }
                                }
                            });

                            ui.separator();
                            ui.label("Current ignore patterns:");
                            let mut remove_index = None;
                            for i in 0..self.ignore_patterns.len() {
                                let pattern = self.ignore_patterns[i].clone(); // clone so we don’t borrow immutably
                                ui.horizontal(|ui| {
                                    ui.label(&pattern);
                                    if ui.button("-").clicked() {
                                        remove_index = Some(i);
                                    }
                                });
                                if remove_index.is_some() {
                                    break;
                                }
                            }
                            if let Some(i) = remove_index {
                                self.ignore_patterns.remove(i);
                            }
                        });

                        // --- Subdirs Column ---
                        row.col(|ui| {
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

                        // --- Search Column ---
                        row.col(|ui| {
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
                    });
                });
        });
    }
}
