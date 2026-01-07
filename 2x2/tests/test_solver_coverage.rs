use rubiks_cube_2x2::cube::{Cube, Move};
use rubiks_cube_2x2::solver;
use std::sync::mpsc;

/// 進捗送信機能のテスト
#[test]
fn test_solve_with_progress_reporting() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);
    cube.apply_move(Move::F);

    let (tx, rx) = mpsc::channel();

    // 進捗送信ありで解法を探索
    let solution = solver::solve_with_progress(&cube, 11, true, Some(tx));

    assert!(solution.found, "解が見つかるべき");

    // 進捗情報が送信されたことを確認
    let mut progress_values = Vec::new();
    while let Ok(progress) = rx.try_recv() {
        progress_values.push(progress);
    }

    assert!(!progress_values.is_empty(), "進捗情報が送信されるべき");
    assert!(progress_values.contains(&1.0), "最終的に100%になるべき");
}

/// 解が見つからない場合のテスト
#[test]
fn test_solve_no_solution_found() {
    let mut cube = Cube::new();
    // 1手のスクランブル
    cube.apply_move(Move::R);

    let (tx, rx) = mpsc::channel();

    // 深度0では解が見つからない
    let solution = solver::solve_with_progress(&cube, 0, true, Some(tx));

    assert!(!solution.found, "深度0では解が見つからないはず");
    assert_eq!(solution.moves.len(), 0, "解が見つからない場合は空の配列");

    // 進捗情報が送信されたことを確認
    let mut progress_values = Vec::new();
    while let Ok(progress) = rx.try_recv() {
        progress_values.push(progress);
    }

    assert!(!progress_values.is_empty(), "進捗情報が送信されるべき");
    assert!(progress_values.contains(&1.0), "最終的に100%になるべき");
}

/// 逆方向探索で解が見つかる場合のテスト
#[test]
fn test_solve_backward_collision() {
    let mut cube = Cube::new();
    // 深いスクランブル
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);
    cube.apply_move(Move::F);
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);

    let (tx, _rx) = mpsc::channel();

    // 逆方向探索で衝突が起こるようにする
    let solution = solver::solve_with_progress(&cube, 11, true, Some(tx));

    assert!(solution.found, "解が見つかるべき");
}

/// 向きも揃える場合の逆方向探索テスト
#[test]
fn test_solve_with_orientation_backward() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);

    let (tx, _rx) = mpsc::channel();

    // ignore_orientation=falseで逆方向探索をカバー
    let solution = solver::solve_with_progress(&cube, 14, false, Some(tx));

    assert!(solution.found, "解が見つかるべき");
}
