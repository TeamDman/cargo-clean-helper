mod app;
mod crawler;
mod init;
use app::MyApp;
use eframe::egui;

fn main() -> eyre::Result<()> {
    init::init()?;

    // Options for how to create the native window:
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Cargo Clean Helper",
        options,
        Box::new(|_cc| {
            // This gives us image support:
            // G:\Programming\Repos\egui\examples\hello_world\src\main.rs
            // egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::<MyApp>::default())
        }),
    )
    .map_err(|e| eyre::eyre!("Failed to run app: {}", e))?;
    Ok(())
}
