use rubiks_cube_2x2::gui::CubeApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_title("2x2 ルービックキューブ"),
        ..Default::default()
    };

    eframe::run_native(
        "2x2 ルービックキューブ",
        options,
        Box::new(|cc| Ok(Box::new(CubeApp::new(cc)))),
    )
}
