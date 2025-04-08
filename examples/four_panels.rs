use eframe::App;
use eframe::CreationContext;
use eframe::NativeOptions;
use eframe::egui::ViewportBuilder;
use eframe::egui::{self};
use egui::ScrollArea;
use egui::Ui;
use egui_extras::Column;
use egui_extras::TableBuilder;
use rand::Rng;
use rand::distr::Alphanumeric;
use std::iter;

struct FourColumnApp {
    strings: Vec<String>,
    selected_idx: Option<usize>,
    col1_width: f32,
    col2_width: f32,
    col3_width: f32,
}

impl Default for FourColumnApp {
    fn default() -> Self {
        // Generate 10,000 random strings of length 256
        let mut rng = rand::thread_rng();
        let strings: Vec<String> = (0..10_000)
            .map(|_| {
                iter::repeat(())
                    .map(|()| rng.sample(Alphanumeric) as char)
                    .take(256)
                    .collect()
            })
            .collect();

        Self {
            strings,
            selected_idx: None,
            col1_width: 200.0,
            col2_width: 200.0,
            col3_width: 200.0,
        }
    }
}

impl App for FourColumnApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let height = ui.available_height();
            TableBuilder::new(ui)
                .resizable(true)
                .column(Column::initial(self.col1_width).at_least(150.0))
                .column(Column::initial(self.col2_width).at_least(150.0))
                .column(Column::remainder().at_least(150.0))
                .column(Column::initial(self.col3_width).at_least(150.0))
                .body(|mut body| {
                    body.row(height, |mut row| {
                        row.col(|ui| {
                            ui.label("Column 1");
                            ui.horizontal(|ui| {
                                ui.label("Content for column 1");
                            });
                        });
                        row.col(|ui| {
                            ui.label("Column 2");
                            ui.horizontal(|ui| {
                                ui.label("Content for column 2");
                            });
                        });
                        row.col(|ui| {
                            ui.label("Column 3");
                            self.render_column_3(ui);
                        });
                        row.col(|ui| {
                            ui.label("Column 4");
                            ui.horizontal(|ui| {
                                ui.label("Content for column 4");
                            });
                        });
                    })
                });
        });
    }
}

impl FourColumnApp {
    fn render_column_3(&mut self, ui: &mut Ui) {
        // Calculate button height and spacing
        let button_height = 24.0;
        let spacing = 8.0;

        // Reserve space for the button and spacing at the bottom
        let table_height = ui.available_height() - button_height - spacing;
        let selected_idx = &mut self.selected_idx;

        // Use a table to display the strings
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .column(Column::remainder())
            .max_scroll_height(table_height)
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong(format!("Strings ({})", self.strings.len()));
                });
            })
            .body(|body| {
                body.rows(24.0, self.strings.len(), |mut row| {
                    let row_idx = row.index();
                    row.col(|ui| {
                        let string = &self.strings[row_idx];
                        
                        // Show a truncated version of the string
                        let display_str = if string.len() > 20 {
                            format!("#{}: {}...", row_idx, &string[..20])
                        } else {
                            format!("#{}: {}", row_idx, string)
                        };
                        
                        let is_selected = *selected_idx == Some(row_idx);
                        if ui.selectable_label(is_selected, display_str).clicked() {
                            *selected_idx = Some(row_idx);
                        }
                    });
                });
            });

        ui.add_space(spacing);

        // Copy to clipboard button at the bottom
        if ui.button("Copy to Clipboard").clicked() {
            if let Some(idx) = self.selected_idx {
                ui.output_mut(|o| {
                    o.copied_text = self.strings[idx].clone();
                });
            }
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([1500.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Four Column App",
        options,
        Box::new(|_cc: &CreationContext<'_>| Ok(Box::new(FourColumnApp::default()))),
    )
}
