#![cfg_attr(target_arch = "wasm32", allow(dead_code))]

#[cfg(not(target_arch = "wasm32"))]
use rubiks_cube_3x3::gui::app::CubeApp;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    // ロギングの初期化（デスクトップのみ）
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("アプリケーション起動");

    tracing::info!("Setting up eframe options");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 780.0])
            .with_title("3x3 ルービックキューブ"),
        ..Default::default()
    };

    tracing::info!("Calling eframe::run_native");
    eframe::run_native(
        "3x3 ルービックキューブ",
        options,
        Box::new(|cc| {
            tracing::info!("eframe closure called, creating CubeApp");
            let app = CubeApp::new(cc);
            tracing::info!("CubeApp created, returning to eframe");
            Ok(Box::new(app))
        }),
    )
}

// wasm32ターゲット用のダミーmain（ビルドエラー回避用）
#[cfg(target_arch = "wasm32")]
fn main() {}
