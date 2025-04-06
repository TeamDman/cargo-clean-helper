// src/app.rs

use crate::crawler;
use eframe::egui;
use egui_extras::Size;
use egui_extras::StripBuilder;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use tokio::runtime::Handle;

// We'll do a small chunked read so you see the directories appear live!
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
    // It's just a reference to the runtime (the runtime itself lives in main).
    rt_handle: Handle,
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
        }
    }

    /// Called when user clicks "Refresh subdirs"
    fn refresh_subdirs(&mut self) {
        // Clear old results
        self.subdirs_text.clear();
        self.indexing_in_progress = true;

        // Copy the lines from the UI to spawn in background:
        let lines: Vec<String> = self
            .root_paths_text
            .lines()
            .map(|line| line.trim().to_owned())
            .filter(|line| !line.is_empty())
            .collect();

        let tx_clone = self.tx.clone();

        // We'll spawn an async task, but `walkdir` is blocking, so let's use spawn_blocking:
        self.rt_handle.spawn(async move {
            tokio::task::spawn_blocking(move || {
                for root in lines {
                    // For each subfolder discovered, send them incrementally:
                    crawler::gather_descendant_dirs_streaming(
                        &root,
                        &tx_clone,
                        &[".git".to_string()],
                    );
                }
                // once done:
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
        // 1) Read from the channel, gather lines
        //    If we read [DONE], we set indexing_in_progress = false
        let mut new_messages = vec![];
        while let Ok(msg) = self.rx.try_recv() {
            new_messages.push(msg);
        }

        // 2) Apply them:
        for msg in new_messages {
            if msg == DONE_SIGNAL {
                self.indexing_in_progress = false;
            } else {
                self.subdirs_text.push_str(&msg);
                self.subdirs_text.push('\n');
            }
        }

        // 3) The UI:
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My Stream Viewer");
            ui.separator();

            // Use StripBuilder to create 3 columns
            StripBuilder::new(ui)
                .size(Size::relative(0.3333))
                .size(Size::relative(0.3333))
                .size(Size::remainder())
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

                        // "Refresh subdirs"
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
