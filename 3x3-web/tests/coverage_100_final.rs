use rubiks_cube_3x3::cube::{Cube, Move};
use rubiks_cube_3x3::solver::solve;

/// Phase 4: 最終的なカバレッジテスト - SOLVER_DEBUG と残りのパス

/// SOLVER_DEBUG=1 で方位パリティエラーのデバッグログをテスト
#[test]
fn test_solver_debug_orientation_parity_error() {
    std::env::set_var("SOLVER_DEBUG", "1");

    // 正常なキューブで色解決後のデバッグログをトリガー
    let mut cube = Cube::new();

    // センターの向きを変更
    cube.stickers[4].orientation = 1;
    cube.stickers[13].orientation = 2;

    let _result = solve(&cube, 30, false);

    std::env::remove_var("SOLVER_DEBUG");
}

/// SOLVER_DEBUG=1 で apply_supercube_fixes の詳細ログをテスト
#[test]
fn test_solver_debug_supercube_fixes_detailed() {
    std::env::set_var("SOLVER_DEBUG", "1");

    // 複数のセンターの向きを変更
    let mut cube = Cube::new();
    cube.stickers[4].orientation = 2; // Up
    cube.stickers[13].orientation = 1; // Front
    cube.stickers[22].orientation = 3; // Right
    cube.stickers[31].orientation = 2; // Back

    let _result = solve(&cube, 30, false);

    std::env::remove_var("SOLVER_DEBUG");
}

/// SOLVER_DEBUG=1 で色解決後の向き情報デバッグログをテスト
#[test]
fn test_solver_debug_color_solved_orientation_info() {
    std::env::set_var("SOLVER_DEBUG", "1");

    // 色は揃っているが向きが異なる
    let mut cube = Cube::new();
    cube.stickers[4].orientation = 1;
    cube.stickers[22].orientation = 2;

    let _result = solve(&cube, 30, false);

    std::env::remove_var("SOLVER_DEBUG");
}

/// 複雑なスクランブルでのデバッグログ
#[test]
fn test_solver_debug_complex_scramble() {
    std::env::set_var("SOLVER_DEBUG", "1");

    let mut cube = Cube::new();

    // 複雑なスクランブル
    cube.scramble(15);

    let _result = solve(&cube, 128, false);

    std::env::remove_var("SOLVER_DEBUG");
}

/// ignore_orientation=true でのデバッグログ
#[test]
fn test_solver_debug_ignore_orientation() {
    std::env::set_var("SOLVER_DEBUG", "1");

    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);
    cube.apply_move(Move::Rp);
    cube.apply_move(Move::Up);

    // 向きを無視して解く
    let _result = solve(&cube, 20, true);

    std::env::remove_var("SOLVER_DEBUG");
}

/// 回転されたキューブでのデバッグログ
#[test]
fn test_solver_debug_rotated_cube() {
    std::env::set_var("SOLVER_DEBUG", "1");

    let mut cube = Cube::new();

    // X, Y, Z回転
    cube.apply_move(Move::X);
    cube.apply_move(Move::Y);

    // スクランブル
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);

    let _result = solve(&cube, 20, false);

    std::env::remove_var("SOLVER_DEBUG");
}

/// 深い探索でのデバッグログ
#[test]
fn test_solver_debug_deep_search() {
    std::env::set_var("SOLVER_DEBUG", "1");

    let mut cube = Cube::new();

    // 複雑なパターン
    cube.scramble(20);

    // 深い探索
    let _result = solve(&cube, 256, false);

    std::env::remove_var("SOLVER_DEBUG");
}

/// 様々な向き修正パターンのデバッグログ
#[test]
fn test_solver_debug_various_orientation_fixes() {
    std::env::set_var("SOLVER_DEBUG", "1");

    // パターン1: 180度回転
    let mut cube1 = Cube::new();
    cube1.stickers[4].orientation = 2;
    let _result1 = solve(&cube1, 30, false);

    // パターン2: 90度ペア回転
    let mut cube2 = Cube::new();
    cube2.stickers[4].orientation = 1;
    cube2.stickers[13].orientation = 3;
    let _result2 = solve(&cube2, 30, false);

    // パターン3: 複数の向き修正
    let mut cube3 = Cube::new();
    cube3.stickers[4].orientation = 1;
    cube3.stickers[13].orientation = 2;
    cube3.stickers[22].orientation = 3;
    let _result3 = solve(&cube3, 30, false);

    std::env::remove_var("SOLVER_DEBUG");
}

/// ランダムサーチのデバッグログ
#[test]
fn test_solver_debug_random_search_comprehensive() {
    std::env::set_var("SOLVER_DEBUG", "1");

    let mut cube = Cube::new();

    // 難しいパターン
    cube.scramble(18);

    // 浅い深さで試してランダムサーチをトリガー
    let _result = solve(&cube, 64, false);

    std::env::remove_var("SOLVER_DEBUG");
}

/// 全ての回転でのデバッグログ
#[test]
fn test_solver_debug_all_rotations() {
    std::env::set_var("SOLVER_DEBUG", "1");

    for rotation in [Move::X, Move::Y, Move::Z, Move::X2, Move::Y2, Move::Z2] {
        let mut cube = Cube::new();
        cube.apply_move(rotation);
        cube.apply_move(Move::R);
        cube.apply_move(Move::U);

        let _result = solve(&cube, 20, false);
    }

    std::env::remove_var("SOLVER_DEBUG");
}
