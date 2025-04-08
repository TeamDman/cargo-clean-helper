use crate::crawler;
use eframe::egui;
use eframe::egui::ScrollArea;
use egui_extras::Column;
use egui_extras::TableBuilder;
use itertools::Itertools;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::time::Duration;
use tokio::runtime::Handle;

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

    // New for adding root directories:
    new_root_input: String,
}

impl MyApp {
    pub fn new(rt_handle: Handle) -> Self {
        let (tx, rx) = mpsc::channel();

        Self {
            root_dirs: vec!["D:\\Repos".into(), "G:\\ml".into(), "G:\\Repos".into()],
            // subdirs: Vec::new(),
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
            new_root_input: String::new(),
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
        ctx.request_repaint_after(Duration::from_millis(100));
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
            ui.vertical(|ui| {
                ui.heading("Cargo Clean Helper");
                ui.separator();
                show_table(self, ui);
            });
        });
    }
}

fn show_table(app: &mut MyApp, ui: &mut egui::Ui) {
    let height = ui.available_height();
    TableBuilder::new(ui)
        .id_salt("main_table")
        .resizable(true)
        .striped(true)
        // .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        // .column(Column::initial(300.).at_least(150.0))
        // .column(Column::initial(300.).at_least(150.0))
        // .column(Column::initial(200.).at_least(150.0))
        .column(Column::remainder()) // last col must be remainder for it to grow with the window
        .column(Column::remainder()) // last col must be remainder for it to grow with the window
        .column(Column::remainder()) // last col must be remainder for it to grow with the window
        .column(Column::remainder()) // last col must be remainder for it to grow with the window
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.strong(format!("Roots ({} entries)", app.root_dirs.len()));
            });
            header.col(|ui| {
                ui.strong("Ignore Patterns");
            });
            header.col(|ui| {
                ui.strong(format!("Subdirs ({} entries)", app.subdirs.len()));
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
                    show_roots_col(app, ui);
                });

                // --- Ignore Patterns Column ---
                row.col(|ui| {
                    show_ignore_col(app, ui);
                });

                // --- Subdirs Column ---
                row.col(|ui| {
                    show_subdirs_col(app, ui);
                });

                // --- Search Column ---
                row.col(|ui| {
                    show_search_col(app, ui);
                });
            });
        });
}

fn show_search_col(app: &mut MyApp, ui: &mut egui::Ui) {
    ui.label("Search:");
    ui.text_edit_singleline(&mut app.search_text);

    if ui.button("Run search").clicked() {
        app.run_search();
    }

    ui.separator();
    ui.label(format!("Results ({} entries)", app.subdirs.len()));
    if let Some((_query, results)) = &app.search_results {
        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                for path in results.iter() {
                    ui.horizontal(|ui| {
                        ui.label(path.display().to_string());
                    });
                }
                if ui.button("Copy to clipboard").clicked() {
                    ui.ctx()
                        .copy_text(results.iter().map(|x| x.display().to_string()).join("\n"));
                }
            });
    } else {
        ui.label("No results yet.");
    }
}

fn show_subdirs_col(app: &mut MyApp, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        // Calculate available height for the table
        let available_height = ui.available_height() - 60.0; // Reserve space for button and spacing

        ScrollArea::horizontal().show(ui, |ui| {
            // Create the table for subdirectories
            TableBuilder::new(ui)
                .id_salt("subdirs_table")
                .resizable(false)
                .striped(true)
                // .column(Column::initial(400.0).at_least(200.0).resizable(true))
                .column(Column::remainder())
                .max_scroll_height(available_height)
                .body(|body| {
                    body.rows(20.0, app.subdirs.len(), |mut row| {
                        let subdir = &app.subdirs[row.index()];
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(subdir.display().to_string());
                            });
                        });
                    });
                });
        });

        // Add some spacing before the button
        ui.add_space(5.0);

        // Center the button horizontally
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            if ui.button("Copy to clipboard").clicked() {
                ui.ctx().copy_text(
                    app.subdirs
                        .iter()
                        .map(|x| x.display().to_string())
                        .join("\n"),
                );
            }
        });
    });
}

fn show_ignore_col(app: &mut MyApp, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        ui.label("Add new ignore pattern:");
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut app.new_pattern_input);
            if ui.button("Add").clicked() {
                if !app.new_pattern_input.trim().is_empty() {
                    app.ignore_patterns
                        .push(app.new_pattern_input.trim().to_string());
                    app.new_pattern_input.clear();
                }
            }
        });

        ui.separator();
        ui.label("Current ignore patterns:");
        let mut remove_index = None;
        for i in 0..app.ignore_patterns.len() {
            let pattern = app.ignore_patterns[i].clone(); // clone so we don’t borrow immutably
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
            app.ignore_patterns.remove(i);
        }
    });
}
fn show_roots_col(app: &mut MyApp, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        // Add new root directory section
        ui.label("Add new root directory:");
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut app.new_root_input);
            if ui.button("Add").clicked() {
                if !app.new_root_input.trim().is_empty() {
                    app.root_dirs.push(PathBuf::from(app.new_root_input.trim()));
                    app.new_root_input.clear();
                }
            }
        });

        ui.separator();
        ui.label(format!(
            "Current root directories ({}):",
            app.root_dirs.len()
        ));

        // Show existing root directories with remove buttons
        let mut remove_index = None;
        for i in 0..app.root_dirs.len() {
            let path = app.root_dirs[i].display().to_string();
            ui.horizontal(|ui| {
                ui.label(&path);
                if ui.button("-").clicked() {
                    remove_index = Some(i);
                }
            });
            if remove_index.is_some() {
                break;
            }
        }

        // Remove the directory if requested
        if let Some(i) = remove_index {
            app.root_dirs.remove(i);
        }

        ui.separator();

        if ui.button("Copy to clipboard").clicked() {
            ui.ctx().copy_text(
                app.root_dirs
                    .iter()
                    .map(|x| x.display().to_string())
                    .join("\n"),
            );
        }

        let refresh_btn = ui.add_enabled(
            !app.indexing_in_progress,
            egui::Button::new("Refresh subdirs"),
        );
        if refresh_btn.clicked() {
            app.refresh_subdirs();
        }

        if app.indexing_in_progress {
            ui.label("Indexing in progress…");
        }
    });
}
