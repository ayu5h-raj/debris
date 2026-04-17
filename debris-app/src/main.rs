mod app;
mod ui;

fn main() -> eframe::Result<()> {
    if std::env::args().any(|a| a == "--version") {
        println!("debris {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Debris")
            .with_inner_size([900.0, 650.0])
            .with_min_inner_size([700.0, 500.0]),
        ..Default::default()
    };
    eframe::run_native("Debris", options, Box::new(|cc| Ok(Box::new(app::SweepApp::new(cc)))))
}
