use rubiks_cube_2x2::cube::{Cube, Move};
use rubiks_cube_2x2::solver::{self, solve_with_progress};
use std::fs;
use std::sync::mpsc;

#[test]
fn test_real_cube_solve_and_verify() {
    let mut cube = Cube::new();
    let scramble = vec![Move::R, Move::U, Move::Fp, Move::D, Move::L, Move::Bp];
    for move_op in &scramble {
        cube.apply_move(*move_op);
    }

    let format = cube.to_file_format();
    let mut cube_from_file = Cube::from_file_format(&format).expect("ファイルフォーマットエラー");

    let (tx, _rx) = mpsc::channel();
    let solution = solve_with_progress(&cube_from_file, 14, true, Some(tx));

    assert!(solution.found);
    for move_op in &solution.moves {
        cube_from_file.apply_move(*move_op);
    }

    assert!(cube_from_file.is_solved());
}

#[test]
fn test_cube_god_solvability() {
    let content = fs::read_to_string("cubes/cube_god.txt").expect("cube_god.txt が見つかりません");
    let cube = Cube::from_file_format(&content).expect("ファイル読み込みに失敗しました");

    // 向きを考慮した解決
    let solution_with_ori = solver::solve(&cube, 11, false);
    assert!(solution_with_ori.found, "向き考慮で解けるはず");

    // 向きを無視した解決
    let solution_without_ori = solver::solve(&cube, 11, true);
    assert!(solution_without_ori.found, "向き無視で解けるはず");
}

#[test]
fn test_user_specified_state_solvability() {
    // ユーザー指定の状態: WWWW / OOOO GGGR RRBG BBRB / YYYY
    let state = "     WWWW\nOOOO GGGR RRBG BBRB\n     YYYY";
    let cube = Cube::from_file_format(state).expect("状態の読み込みに失敗");

    if cube.is_valid_state().is_ok() {
        let solution = solver::solve(&cube, 11, true);
        assert!(solution.found, "有効な状態なら解けるはず");

        let mut check_cube = cube.clone();
        for &mv in &solution.moves {
            check_cube.apply_move(mv);
        }
        assert!(check_cube.is_solved());
    }
}

#[test]
fn test_difficult_patterns_god_number() {
    // R U を5回繰り返すパターン (10手)
    let mut cube = Cube::new();
    for _ in 0..5 {
        cube.apply_move(Move::R);
        cube.apply_move(Move::U);
    }
    let solution = solver::solve(&cube, 11, true);
    assert!(solution.found);
    assert!(solution.moves.len() <= 11);

    // 6 Spot パターン
    let mut cube2 = Cube::new();
    let pattern = vec![
        Move::R,
        Move::U,
        Move::U,
        Move::R,
        Move::R,
        Move::U,
        Move::U,
        Move::R,
        Move::U,
        Move::U,
        Move::R,
        Move::R,
    ];
    for mv in &pattern {
        cube2.apply_move(*mv);
    }
    let solution2 = solver::solve(&cube2, 11, true);
    assert!(solution2.found);
}
