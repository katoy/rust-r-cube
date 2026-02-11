use rubiks_cube_3x3::cube::{Cube, Move};
use rubiks_cube_3x3::solver::solve;

/// SOLVER_DEBUG=1 環境変数でのデバッグログテスト
#[test]
fn test_solver_debug_logging_comprehensive() {
    std::env::set_var("SOLVER_DEBUG", "1");

    // 様々な状態でソルバーを実行してデバッグログをトリガー

    // 1. 簡単なスクランブル
    let mut cube1 = Cube::new();
    cube1.apply_move(Move::U);
    cube1.apply_move(Move::R);
    let _result1 = solve(&cube1, 20, false);

    // 2. 色は揃っているが向きが異なる
    let mut cube2 = Cube::new();
    cube2.stickers[4].orientation = 2; // Up center 180度回転
    let _result2 = solve(&cube2, 20, false);

    // 3. 複雑なスクランブル
    let mut cube3 = Cube::new();
    cube3.scramble(10);
    let _result3 = solve(&cube3, 30, false);

    // 4. ignore_orientation = true
    let mut cube4 = Cube::new();
    cube4.apply_move(Move::R);
    cube4.apply_move(Move::U);
    cube4.apply_move(Move::Rp);
    let _result4 = solve(&cube4, 20, true);

    std::env::remove_var("SOLVER_DEBUG");
}

/// SOLVER_DEBUG=1 で色解決後のデバッグログをテスト
#[test]
fn test_solver_debug_color_solved_paths() {
    std::env::set_var("SOLVER_DEBUG", "1");

    // 色は揃っているが向きが異なる複数のパターン
    let mut cube1 = Cube::new();
    cube1.stickers[4].orientation = 1;
    cube1.stickers[13].orientation = 3;
    let _result1 = solve(&cube1, 30, false);

    // センター2つの向きが異なる
    let mut cube2 = Cube::new();
    cube2.stickers[4].orientation = 2;
    cube2.stickers[22].orientation = 2;
    let _result2 = solve(&cube2, 30, false);

    std::env::remove_var("SOLVER_DEBUG");
}

/// SOLVER_DEBUG=1 で apply_supercube_fixes のデバッグログをテスト
#[test]
fn test_solver_debug_supercube_fixes() {
    std::env::set_var("SOLVER_DEBUG", "1");

    // 向き修正が必要な状態
    let mut cube = Cube::new();

    // 複数のセンターの向きを変更
    cube.stickers[4].orientation = 1; // Up
    cube.stickers[13].orientation = 2; // Front
    cube.stickers[22].orientation = 3; // Right

    let _result = solve(&cube, 30, false);

    std::env::remove_var("SOLVER_DEBUG");
}

/// SOLVER_DEBUG=1 で try_solve_with_rotation のデバッグログをテスト
#[test]
fn test_solver_debug_rotation_attempts() {
    std::env::set_var("SOLVER_DEBUG", "1");

    // 様々な回転状態でテスト
    let mut cube = Cube::new();
    cube.apply_move(Move::X);
    cube.apply_move(Move::U);
    cube.apply_move(Move::R);

    let _result = solve(&cube, 20, false);

    std::env::remove_var("SOLVER_DEBUG");
}

/// SOLVER_DEBUG=1 で get_target_oris のデバッグログをテスト
#[test]
fn test_solver_debug_target_oris() {
    std::env::set_var("SOLVER_DEBUG", "1");

    // 回転された状態で向き修正をトリガー
    let mut cube = Cube::new();
    cube.apply_move(Move::Y);
    cube.stickers[4].orientation = 1;

    let _result = solve(&cube, 30, false);

    std::env::remove_var("SOLVER_DEBUG");
}

/// SOLVER_DEBUG=1 でランダムサーチのデバッグログをテスト
#[test]
fn test_solver_debug_random_search() {
    std::env::set_var("SOLVER_DEBUG", "1");

    // 直接解決できない複雑な状態
    let mut cube = Cube::new();
    cube.scramble(15);

    // 浅い深さで試して、ランダムサーチをトリガー
    let _result = solve(&cube, 128, false);

    std::env::remove_var("SOLVER_DEBUG");
}
