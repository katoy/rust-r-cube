use rubiks_cube_3x3::cube::{Color, Cube, Move};
use rubiks_cube_3x3::solver::{self, Solution};
use std::sync::mpsc;

// from_file_formatのエラーケース追加テスト
#[test]
fn test_from_file_format_too_few_lines() {
    let invalid = "          WWWWWWWWW\nGGGGGGGGG RRRRRRRRR BBBBBBBBB OOOOOOOOO\n";
    assert!(Cube::from_file_format(invalid).is_err());
}

#[test]
fn test_from_file_format_line1_too_short() {
    let invalid =
        "          WWWWWWWW\nGGGGGGGGG RRRRRRRRR BBBBBBBBB OOOOOOOOO\n          YYYYYYYYY\n";
    assert!(Cube::from_file_format(invalid).is_err());
}

#[test]
fn test_from_file_format_line2_too_short() {
    let invalid =
        "          WWWWWWWWW\nGGGGGGGGG RRRRRRRRR BBBBBBBBB OOOOOOOO\n          YYYYYYYYY\n";
    assert!(Cube::from_file_format(invalid).is_err());
}

#[test]
fn test_from_file_format_line3_too_short() {
    let invalid =
        "          WWWWWWWWW\nGGGGGGGGG RRRRRRRRR BBBBBBBBB OOOOOOOOO\n          WWWWWWWW\n";
    assert!(Cube::from_file_format(invalid).is_err());
}

#[test]
fn test_validate_colors_missing_color() {
    // すべての色が欠けているケース
    let mut colors = [Color::White; 54];
    for i in 0..9 {
        colors[i] = Color::White;
    }
    for i in 9..18 {
        colors[i] = Color::Yellow;
    }
    for i in 18..27 {
        colors[i] = Color::Green;
    }
    for i in 27..36 {
        colors[i] = Color::Blue;
    }
    for i in 36..45 {
        colors[i] = Color::Red;
    }
    for i in 45..54 {
        colors[i] = Color::Orange;
    }

    colors[20] = Color::Gray; // 1つを Gray に置き換え
    let result = Cube::validate_colors(&colors);
    assert!(result.is_err());
}

#[test]
fn test_validate_colors_all_wrong() {
    // すべての色が間違っているケース
    let colors = [Color::Gray; 54];
    let result = Cube::validate_colors(&colors);
    assert!(result.is_err());
}

#[test]
fn test_cube_default_equals_new() {
    let cube1 = Cube::default();
    let cube2 = Cube::new();
    assert_eq!(cube1, cube2);
}

#[test]
fn test_color_gray_not_in_solved_cube() {
    let cube = Cube::new();
    for i in 0..54 {
        assert_ne!(cube.get_sticker(i).color, Color::Gray);
    }
}

#[test]
fn test_from_file_format_empty_string() {
    assert!(Cube::from_file_format("").is_err());
}

#[test]
fn test_from_file_format_one_line() {
    assert!(Cube::from_file_format("WWWW").is_err());
}

#[test]
fn test_from_file_format_two_lines() {
    let invalid = "     WWWW\nGGGG RRRR BBBB OOOO";
    assert!(Cube::from_file_format(invalid).is_err());
}

#[test]
fn test_from_file_format_whitespace_only() {
    let invalid = "     \n      \n     \n";
    assert!(Cube::from_file_format(invalid).is_err());
}

#[test]
fn test_apply_orientation_solution_error() {
    let mut cube = Cube::new();
    // 物理的に不可能な状態（捻じれパリティエラー）を作り出す
    // UFL: 6, 36, 20
    let c0 = cube.stickers[6];
    let c1 = cube.stickers[36];
    let c2 = cube.stickers[20];
    cube.stickers[6] = c1;
    cube.stickers[36] = c2;
    cube.stickers[20] = c0;

    let solution = Solution {
        moves: vec![],
        found: true,
    };
    let result = cube.apply_orientation_solution(&solution);
    assert!(result.is_err());
}

#[test]
fn test_to_file_format_with_gray() {
    let mut cube = Cube::new();
    cube.set_sticker_color(0, Color::Gray);
    let s = cube.to_file_format();
    assert!(s.contains('.'));
}

#[test]
fn test_coverage_gap_cube_default() {
    let cube = <Cube as Default>::default();
    assert!(cube.is_solved());
}

#[test]
fn test_coverage_gap_solver_early_breaks() {
    let cube = Cube::new();
    let sol = solver::solve(&cube, 11, true);
    assert!(sol.found);
    assert_eq!(sol.moves.len(), 0);
}

#[test]
fn test_coverage_gap_solver_forward_queue_empty() {
    let cube = Cube::new();
    let mut scrambled = cube.clone();
    scrambled.apply_move(Move::R);
    let sol = solver::solve(&scrambled, 1, true);
    assert!(sol.found);
}

#[test]
fn test_coverage_gap_solver_backward_visited_collision() {
    let cube = Cube::new();
    let sol = solver::solve(&cube, 11, false);
    assert!(sol.found);
}

#[test]
fn test_coverage_gap_solver_not_found_with_progress() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);

    let (tx, rx) = mpsc::channel();
    let solution = solver::solve_with_progress(&cube, 1, false, Some(tx));
    assert!(!solution.found);

    let progress: Vec<f32> = rx.into_iter().collect();
    assert!(progress.contains(&1.0));
}
