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

    println!(
        "Solution found: {}, moves: {:?}",
        solution.found, solution.moves
    );
    assert!(solution.found);
    for move_op in &solution.moves {
        cube_from_file.apply_move(*move_op);
    }

    if !cube_from_file.is_solved() {
        println!("Cube NOT solved!");
        println!("{}", cube_from_file.to_file_format());
    }
    assert!(cube_from_file.is_solved());
}

#[test]
fn test_cube_god_solvability() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest_dir.join("cubes").join("cube_god.txt");
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("cube_god.txt が見つかりません: {}", path.display()));
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
        println!(
            "User specified state solution: found={}, moves={:?}",
            solution.found, solution.moves
        );
        assert!(solution.found, "有効な状態なら解けるはず");

        let mut check_cube = cube.clone();
        for &mv in &solution.moves {
            check_cube.apply_move(mv);
        }
        if !check_cube.is_solved() {
            println!("User specified state NOT solved!");
            println!("{}", check_cube.to_file_format());
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

// --- Regression Tests (merged from regression_tests.rs) ---

#[test]
fn test_solve_multiple_scrambles_regression() {
    let mut cube1 = Cube::new();
    cube1.apply_move(Move::R);
    cube1.apply_move(Move::U);
    cube1.apply_move(Move::F);

    let solution1 = solver::solve(&cube1, 11, true);
    assert!(solution1.found);

    let mut check_cube1 = cube1.clone();
    for &mv in &solution1.moves {
        check_cube1.apply_move(mv);
    }
    assert!(check_cube1.is_solved());

    let mut cube2 = Cube::new();
    cube2.apply_move(Move::R);
    cube2.apply_move(Move::U);
    cube2.apply_move(Move::R);
    cube2.apply_move(Move::U);

    let solution2 = solver::solve(&cube2, 11, true);
    assert!(solution2.found);

    let mut check_cube2 = cube2.clone();
    for &mv in &solution2.moves {
        check_cube2.apply_move(mv);
    }
    assert!(check_cube2.is_solved());
}

#[test]
fn test_solve_with_orientation_regression() {
    let mut cube1 = Cube::new();
    cube1.apply_move(Move::R);
    cube1.apply_move(Move::U);
    cube1.apply_move(Move::F);

    let solution1 = solver::solve(&cube1, 6, false);
    assert!(solution1.found);

    let mut check_cube1 = cube1.clone();
    for &mv in &solution1.moves {
        check_cube1.apply_move(mv);
    }
    assert!(solver::is_fully_solved(&check_cube1));
}

// --- Workflow Tests (merged from workflow_tests.rs) ---

#[test]
fn test_specific_cube_file_operations() {
    let input_content = r#"     WWWW
GGGG RRRR BBBB OOOO
     YYYY
"#;
    let cube = Cube::from_file_format(input_content).expect("Error");
    let output_content = cube.to_file_format();

    let input_normalized: String = input_content
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    let output_normalized: String = output_content
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    assert_eq!(input_normalized, output_normalized);
}

#[test]
fn test_valid_cube_complete_workflow() {
    let mut cube = Cube::new();
    let scramble = vec![Move::R, Move::U, Move::Fp, Move::D, Move::L];
    for move_op in &scramble {
        cube.apply_move(*move_op);
    }

    let saved_content = cube.to_file_format();
    let cube_from_file = Cube::from_file_format(&saved_content).expect("Error");

    let (tx, _rx) = std::sync::mpsc::channel();
    let mut cube_clone = cube_from_file.clone();
    let solution = solve_with_progress(&cube_from_file, 14, true, Some(tx));

    assert!(solution.found);
    for move_op in &solution.moves {
        cube_clone.apply_move(*move_op);
    }

    for face_start in [0, 4, 8, 12, 16, 20] {
        let first_color = cube_clone.get_sticker(face_start).color;
        for offset in 1..4 {
            assert_eq!(
                cube_clone.get_sticker(face_start + offset).color,
                first_color
            );
        }
    }
}
