use rubiks_cube_2x2::gui::CubeApp;

fn main() -> eframe::Result<()> {
    // ロギングの初期化
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("アプリケーション起動");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 780.0])
            .with_title("2x2 ルービックキューブ"),
        ..Default::default()
    };

    eframe::run_native(
        "2x2 ルービックキューブ",
        options,
        Box::new(|cc| Ok(Box::new(CubeApp::new(cc)))),
    )
}
