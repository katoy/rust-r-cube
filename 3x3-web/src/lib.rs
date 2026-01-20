pub mod cube;
pub mod error;
pub mod gui;
pub mod history;
pub mod solver;
pub mod statistics;

// Web用のエントリポイント
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// Webブラウザでアプリケーションを起動します
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn start(canvas_id: &str) -> Result<(), wasm_bindgen::JsValue> {
    // パニック時のスタックトレースをコンソールに出力
    console_error_panic_hook::set_once();

    // tracing-wasmでログをブラウザコンソールに出力
    tracing_wasm::set_as_global_default();

    tracing::info!("Webアプリケーション起動");

    let web_options = eframe::WebOptions::default();

    // canvas_idからHtmlCanvasElementを取得
    let window = web_sys::window().expect("windowオブジェクト取得失敗");
    let document = window.document().expect("documentオブジェクト取得失敗");
    let canvas = document
        .get_element_by_id(canvas_id)
        .ok_or_else(|| {
            wasm_bindgen::JsValue::from_str(&format!("canvas '{}' が見つかりません", canvas_id))
        })?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| wasm_bindgen::JsValue::from_str("要素がcanvasではありません"))?;

    eframe::WebRunner::new()
        .start(
            canvas,
            web_options,
            Box::new(|cc| Ok(Box::new(gui::app::CubeApp::new(cc)))),
        )
        .await?;

    Ok(())
}
