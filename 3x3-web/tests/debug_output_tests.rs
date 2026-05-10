use rubiks_cube_3x3::cube::{Color, Cube, Face, Move};
use rubiks_cube_3x3::solver::{solve, SolverState};
use std::io::Write;

/// SOLVER_DEBUG 環境変数を活用したデバッグ出力のテスト

#[test]
fn test_solver_debug_enabled() {
    // SOLVER_DEBUG 環境変数を設定してテスト
    std::env::set_var("SOLVER_DEBUG", "1");

    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);

    // テスト: solve 呼び出しでデバッグ出力が生成される
    let result = solve(&cube, 10, false);
    assert!(result.found);

    // 環境変数をクリア
    std::env::remove_var("SOLVER_DEBUG");
}

#[test]
fn test_solver_debug_target_oris() {
    // get_target_oris のデバッグパスをテスト
    std::env::set_var("SOLVER_DEBUG", "1");

    let mut cube = Cube::new();
    // セッターセンターの配置が異なるようにスクランブル
    cube.apply_move(Move::X);
    cube.apply_move(Move::Y);
    cube.apply_move(Move::Z);

    let result = solve(&cube, 20, false);
    // solve が実行され、デバッグ情報が出力される（画面には見えないが実行される）
    let _ = result;

    std::env::remove_var("SOLVER_DEBUG");
}

#[test]
fn test_solver_debug_with_color_only_solution() {
    // 色のみ解決する場合のデバッグパス
    std::env::set_var("SOLVER_DEBUG", "1");

    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::R);
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);

    let result = solve(&cube, 30, false);

    // デバッグパスが実行される
    std::env::remove_var("SOLVER_DEBUG");
}

#[test]
fn test_solver_debug_search_iterations() {
    // search での複数イテレーションデバッグパス
    std::env::set_var("SOLVER_DEBUG", "1");

    let mut cube = Cube::new();
    for _ in 0..3 {
        cube.apply_move(Move::R);
        cube.apply_move(Move::U);
    }

    let result = solve(&cube, 20, false);
    assert!(result.found);

    std::env::remove_var("SOLVER_DEBUG");
}

#[test]
fn test_solver_debug_rotation_attempts() {
    // 回転試行時のデバッグパス
    std::env::set_var("SOLVER_DEBUG", "1");

    let mut cube = Cube::new();
    cube.apply_move(Move::X);
    cube.apply_move(Move::R);

    let result = solve(&cube, 15, false);
    // デバッグ出力が生成される

    std::env::remove_var("SOLVER_DEBUG");
}

#[test]
fn test_solver_state_with_debug() {
    // SolverState のデバッグパス
    std::env::set_var("SOLVER_DEBUG", "1");

    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);
    cube.apply_move(Move::F);

    let mut state = SolverState::new(&cube, 10, false);

    // 複数チャンクを処理
    for _ in 0..5 {
        let (_, done) = state.process_chunk(100);
        if done {
            break;
        }
    }

    let solution = state.get_solution();
    assert!(solution.is_some());

    std::env::remove_var("SOLVER_DEBUG");
}

#[test]
fn test_debug_output_no_panic() {
    // デバッグ出力が有効な場合でも panic しないことを確認
    std::env::set_var("SOLVER_DEBUG", "1");

    let mut cube = Cube::new();
    cube.scramble(5);

    // 複数回 solve を呼び出してもパニックしない
    for _ in 0..3 {
        let _ = solve(&cube, 15, false);
    }

    std::env::remove_var("SOLVER_DEBUG");
}

#[test]
fn test_solver_debug_with_various_depths() {
    // 複数の深さでデバッグパスをテスト
    std::env::set_var("SOLVER_DEBUG", "1");

    let mut cube = Cube::new();
    cube.apply_move(Move::R);

    for depth in [2, 5, 10] {
        let result = solve(&cube, depth, false);
        if depth >= 2 {
            assert!(result.found);
        }
    }

    std::env::remove_var("SOLVER_DEBUG");
}

#[test]
fn test_apply_supercube_fixes_debug() {
    // supercube fixes のデバッグパス
    std::env::set_var("SOLVER_DEBUG", "1");

    let mut cube = Cube::new();
    cube.apply_move(Move::X);
    cube.apply_move(Move::Y);

    let result = solve(&cube, 20, false);
    // apply_supercube_fixes の debug output が実行される可能性がある

    std::env::remove_var("SOLVER_DEBUG");
}

#[test]
fn test_debug_without_env_var() {
    // SOLVER_DEBUG が設定されていない場合
    std::env::remove_var("SOLVER_DEBUG");

    let mut cube = Cube::new();
    cube.apply_move(Move::R);

    let result = solve(&cube, 10, false);
    assert!(result.found);
    // デバッグ出力は生成されない
}

#[test]
fn test_debug_with_empty_env_var() {
    // SOLVER_DEBUG が空の場合も is_ok() は true
    std::env::set_var("SOLVER_DEBUG", "");

    let mut cube = Cube::new();
    cube.apply_move(Move::R);

    let result = solve(&cube, 10, false);
    assert!(result.found);

    std::env::remove_var("SOLVER_DEBUG");
}

#[test]
fn test_solver_state_debug_process_chunk() {
    // SolverState::process_chunk でのデバッグパス
    std::env::set_var("SOLVER_DEBUG", "1");

    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);

    let mut state = SolverState::new(&cube, 8, false);

    // 最初のチャンク処理
    let (processed, _) = state.process_chunk(50);
    assert!(processed > 0 || state.get_solution().is_some());

    std::env::remove_var("SOLVER_DEBUG");
}

#[test]
fn test_multiple_debug_sessions() {
    // 複数回のデバッグセッションをテスト
    let mut cube = Cube::new();
    cube.apply_move(Move::R);

    for i in 0..3 {
        std::env::set_var("SOLVER_DEBUG", "1");

        let result = solve(&cube, 10, false);
        assert!(result.found, "Iteration {}", i);

        std::env::remove_var("SOLVER_DEBUG");
    }
}
