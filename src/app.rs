// src/app.rs

use crate::crawler;
use eframe::egui;
use egui_extras::Column;
use egui_extras::TableBuilder;
use itertools::Itertools;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use tokio::runtime::Handle;

// We'll do a small chunked read so you see directories appear live!
pub enum AppMessage {
    Subdir(PathBuf),
    Done,
}

pub struct MyApp {
    root_dirs: Vec<PathBuf>,
    search_text: String,
    subdirs: Vec<PathBuf>,
    search_results: Option<(String, Vec<PathBuf>)>,

    // For incremental indexing:
    tx: Sender<AppMessage>,
    rx: Receiver<AppMessage>,
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
            root_dirs: vec!["D:\\Repos".into(), "G:\\ml".into(), "G:\\Repos".into()],
            subdirs: (1..2000)
                .map(|i| PathBuf::from(format!("Subdir {} - {}", i, "asd".repeat(45))))
                .collect(),
            search_text: String::new(),
            search_results: None,
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
        self.subdirs.clear();
        self.search_results = None;
        self.indexing_in_progress = true;

        // Copy current ignore patterns into local variable for the background thread
        let ignore_list = self.ignore_patterns.clone();
        let tx_clone = self.tx.clone();
        let root_dirs = self.root_dirs.clone();
        self.rt_handle.spawn(async move {
            tokio::task::spawn_blocking(move || {
                for root in root_dirs {
                    crawler::gather_descendant_dirs_streaming(root, &tx_clone, &ignore_list);
                }
                let _ = tx_clone.send(AppMessage::Done);
            })
            .await
            .ok();
        });
    }

    fn run_search(&mut self) {
        let needle = self.search_text.trim().to_lowercase();
        let mut results = Vec::new();

        for line in &self.subdirs {
            if line.display().to_string().contains(&needle) {
                results.push(line.to_owned());
            }
        }

        self.search_results = Some((needle, results));
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
            match msg {
                AppMessage::Subdir(path) => {
                    self.subdirs.push(path);
                }
                AppMessage::Done => {
                    self.indexing_in_progress = false;
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Cargo Clean Helper");
            ui.separator();

            let height = ui.available_height();
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
                    body.row(height, |mut row| {
                        // --- Roots Column ---
                        row.col(|ui| {
                            ui.label(format!("Roots ({} entries)", self.root_dirs.len()));
                            egui::ScrollArea::vertical()
                                .max_height(200.0)
                                .show(ui, |ui| {
                                    for path in self.root_dirs.iter() {
                                        ui.horizontal(|ui| {
                                            ui.label(path.display().to_string());
                                        });
                                    }
                                });

                            if ui.button("Copy to clipboard").clicked() {
                                ui.ctx().copy_text(
                                    self.root_dirs
                                        .iter()
                                        .map(|x| x.display().to_string())
                                        .join("\n"),
                                );
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
                        // --- Subdirs Column ---
                        row.col(|ui| {
                            ui.vertical(|ui| {
                                ui.label(format!("Subdirs ({} entries)", self.subdirs.len()));

                                // The scroll area should take most of the available space
                                let available_height = ui.available_height() - 40.0; // Reserve space for button and spacing

                                egui::ScrollArea::vertical()
                                    .auto_shrink([false, false])
                                    .max_height(available_height)
                                    .show(ui, |ui| {
                                        for subdir in &self.subdirs {
                                            ui.horizontal(|ui| {
                                                ui.label(subdir.display().to_string());
                                            });
                                        }
                                    });

                                // Add some spacing before the button
                                ui.add_space(5.0);

                                // Center the button horizontally
                                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                                    if ui.button("Copy to clipboard").clicked() {
                                        ui.ctx().copy_text(
                                            self.subdirs
                                                .iter()
                                                .map(|x| x.display().to_string())
                                                .join("\n"),
                                        );
                                    }
                                });
                            });
                        });

                        // --- Search Column ---
                        row.col(|ui| {
                            ui.label("Search:");
                            ui.text_edit_singleline(&mut self.search_text);

                            if ui.button("Run search").clicked() {
                                self.run_search();
                            }

                            ui.separator();
                            ui.label(format!("Results ({} entries)", self.subdirs.len()));
                            if let Some((_query, results)) = &self.search_results {
                                egui::ScrollArea::vertical()
                                    .max_height(200.0)
                                    .show(ui, |ui| {
                                        for path in results.iter() {
                                            ui.horizontal(|ui| {
                                                ui.label(path.display().to_string());
                                            });
                                        }
                                        if ui.button("Copy to clipboard").clicked() {
                                            ui.ctx().copy_text(
                                                results
                                                    .iter()
                                                    .map(|x| x.display().to_string())
                                                    .join("\n"),
                                            );
                                        }
                                    });
                            } else {
                                ui.label("No results yet.");
                            }
                        });
                    });
                });
        });
    }
}
