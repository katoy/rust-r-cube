#![cfg(target_arch = "wasm32")]

use rubiks_cube_2x2::cube::{Cube, Move};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// ブラウザ環境でキューブが正しく作成されることをテスト
#[wasm_bindgen_test]
fn test_cube_creation_in_browser() {
    let cube = Cube::new();
    assert!(
        cube.is_solved(),
        "新しく作成されたキューブは解決済み状態であるべき"
    );
}

/// ブラウザ環境で基本的な回転操作が動作することをテスト
#[wasm_bindgen_test]
fn test_basic_rotation_in_browser() {
    let mut cube = Cube::new();

    // R回転
    cube.apply_move(Move::R);
    assert!(!cube.is_solved(), "R回転後はキューブは解決済みでないべき");

    // R'回転で元に戻す
    cube.apply_move(Move::Rp);
    assert!(cube.is_solved(), "R + R'で元の状態に戻るべき");
}

/// ブラウザ環境でUndo機能が動作することをテスト
#[wasm_bindgen_test]
fn test_undo_in_browser() {
    let mut cube = Cube::new();
    let mut history = vec![];

    // 回転を実行
    cube.apply_move(Move::R);
    history.push(Move::R);
    assert!(!cube.is_solved());

    // Undoで元に戻す
    if let Some(last_move) = history.pop() {
        cube.apply_move(last_move.inverse());
    }
    assert!(cube.is_solved(), "Undoで元の状態に戻るべき");
}

/// ブラウザ環境で複数の操作が正しく動作することをテスト
#[wasm_bindgen_test]
fn test_multiple_moves_in_browser() {
    let mut cube = Cube::new();

    // 複数の回転を実行
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);
    cube.apply_move(Move::Rp);
    cube.apply_move(Move::Up);

    assert!(
        cube.is_solved(),
        "R U R' U'は完全な操作シーケンスなので元の状態に戻るべき"
    );
}

/// ブラウザ環境でスクランブルと解法が動作することをテスト
#[wasm_bindgen_test]
fn test_scramble_in_browser() {
    let mut cube = Cube::new();

    // スクランブル
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);
    cube.apply_move(Move::F);

    assert!(
        !cube.is_solved(),
        "スクランブル後はキューブは解決済みでないべき"
    );

    // 逆操作で解く
    cube.apply_move(Move::Fp);
    cube.apply_move(Move::Up);
    cube.apply_move(Move::Rp);

    assert!(cube.is_solved(), "逆操作で元の状態に戻るべき");
}

/// ブラウザ環境でファイル形式への変換が動作することをテスト
#[wasm_bindgen_test]
fn test_file_format_in_browser() {
    let cube = Cube::new();
    let file_format = cube.to_file_format();

    assert!(
        file_format.contains("WWWW"),
        "ファイル形式には白面の情報が含まれるべき"
    );
    assert!(
        file_format.contains("YYYY"),
        "ファイル形式には黄面の情報が含まれるべき"
    );
}

/// ブラウザ環境でファイル形式からの読み込みが動作することをテスト
#[wasm_bindgen_test]
fn test_from_file_format_in_browser() {
    let original = Cube::new();
    let file_format = original.to_file_format();

    match Cube::from_file_format(&file_format) {
        Ok(restored) => {
            assert_eq!(
                restored.to_file_format(),
                file_format,
                "復元されたキューブは元と同じであるべき"
            );
        }
        Err(e) => {
            panic!("ファイル形式からの読み込みに失敗: {:?}", e);
        }
    }
}
