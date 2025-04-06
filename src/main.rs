// src/main.rs

mod app;
mod crawler;
mod init;

use app::MyApp;
use eframe::egui;
use eyre::Result;
use std::time::Duration;
use tokio::runtime::Runtime;

fn main() -> Result<()> {
    init::init()?;

    // 1) Create a Tokio runtime
    let rt = Runtime::new()?;

    // 2) Keep the runtime alive in a separate thread:
    std::thread::spawn({
        let rt_handle = rt.handle().clone();
        move || {
            // block_on a never-ending future
            rt_handle.block_on(async {
                loop {
                    tokio::time::sleep(Duration::from_secs(3600)).await;
                }
            });
        }
    });

    // 3) Pass the runtime HANDLE (not the entire runtime) into our MyApp.
    let app = MyApp::new(rt.handle().clone());

    // 4) Launch eframe:
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([920.0, 550.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Cargo Clean Helper",
        native_options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
    .map_err(|err| eyre::eyre!("Failed to run eframe: {}", err))?;

    Ok(())
}
