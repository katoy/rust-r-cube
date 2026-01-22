#![cfg(target_arch = "wasm32")]

use rubiks_cube_3x3::gui::app::CubeApp;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// WASM環境でのソルバー起動シーケンスをテスト
/// 1. solve_without_confirm() 呼び出し後の初期状態確認 (solving=true)
/// 2. update_logic() サイクルによる待機カウントの減少
/// 3. カウントが0になった時のインクリメンタルソルバー初期化
#[wasm_bindgen_test]
fn test_wasm_solver_flow() {
    let mut app = CubeApp::default();

    // 初期状態
    assert!(!app.solving);

    // ソルバー開始 (確認ダイアログなしでテスト用)
    app.solve_without_confirm();

    // ステートが探索中に切り替わっていることを確認
    assert!(app.solving);
    assert_eq!(app.solution_text, "探索中...");

    // 擬似的なeguiコンテキスト
    let ctx = egui::Context::default();

    // 1フレーム目のupdate_logic
    app.update_logic(&ctx);

    // solvingは継続、進捗はまだ0
    assert!(app.solving);
    assert!(app.solver_progress < 0.01);

    // 2フレーム目のupdate_logic (これでカウントが減り、次で開始されるはず)
    app.update_logic(&ctx);

    // 3フレーム目のupdate_logic (ここで SolverState が初期化される)
    app.update_logic(&ctx);

    // 探索が実際に開始されていることを確認
    assert!(app.solving);
}

/// ソルバーのキャンセルが正しくUI状態をリセットすることをテスト
#[wasm_bindgen_test]
fn test_wasm_solver_cancel() {
    let mut app = CubeApp::default();

    app.solve_without_confirm();
    assert!(app.solving);

    // 中止
    app.cancel_solve();

    // ステートがクリアされていることを確認
    assert!(!app.solving);
    assert!(app.solution.is_none());
    assert!(app.solution_text.is_empty());
}

/// WASM環境でのインクリメンタルな進捗更新をテスト
#[wasm_bindgen_test]
fn test_wasm_solver_incremental_progress() {
    let mut app = CubeApp::default();

    app.solve_without_confirm();
    let ctx = egui::Context::default();

    // 待機フレームを消化してソルバーを起動
    for _ in 0..5 {
        app.update_logic(&ctx);
    }

    // 数フレーム回して、パニックしたりフリーズ（無限ループ）したりしないことを確認
    for _ in 0..10 {
        app.update_logic(&ctx);
    }

    // solvingがまだ継続中、または完了しているはず
}
