use rubiks_cube_3x3::cube::{Cube, Move};
use rubiks_cube_3x3::solver::{self, is_fully_solved, solve_with_progress};
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
    let solution = solve_with_progress(&cube_from_file, 24, true, Some(tx));

    assert!(solution.found);
    for move_op in &solution.moves {
        cube_from_file.apply_move(*move_op);
    }
    assert!(cube_from_file.is_solved());
}

#[test]
fn test_difficult_patterns_solvability() {
    // 6 Spot Pattern
    let mut cube = Cube::new();
    let pattern = [
        Move::U,
        Move::D2,
        Move::L,
        Move::R2,
        Move::F,
        Move::B2,
        Move::U,
        Move::D2,
    ];
    for mv in &pattern {
        cube.apply_move(*mv);
    }

    let res = solver::solve(&cube, 24, true);
    assert!(res.found);
}

// ==================== Regression Tests (Moved from regression_tests.rs) ====================

#[test]
fn test_solve_multiple_scrambles_regression() {
    let scenarios = vec![
        vec![Move::R, Move::U, Move::F],
        vec![Move::R, Move::U, Move::R, Move::U],
        vec![Move::R, Move::U, Move::F, Move::R, Move::U],
    ];

    for moves in scenarios {
        let mut cube = Cube::new();
        for &mv in &moves {
            cube.apply_move(mv);
        }

        let solution = solver::solve(&cube, 11, true);
        assert!(
            solution.found,
            "Regression: Scramble {:?} should be solvable",
            moves
        );

        let mut check_cube = cube.clone();
        for &mv in &solution.moves {
            check_cube.apply_move(mv);
        }
        assert!(check_cube.is_solved());
    }
}

#[test]
fn test_solve_with_orientation_regression() {
    let moves = vec![Move::R, Move::U, Move::F];
    let mut cube = Cube::new();
    for mv in moves {
        cube.apply_move(mv);
    }

    let solution = solver::solve(&cube, 10, false); // Respect orientation
    assert!(solution.found);

    let mut check_cube = cube.clone();
    for &mv in &solution.moves {
        check_cube.apply_move(mv);
    }
    assert!(is_fully_solved(&check_cube));
}
