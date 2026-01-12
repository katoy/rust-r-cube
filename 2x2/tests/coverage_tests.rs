use rubiks_cube_2x2::cube::{Color, Cube, Move};
use rubiks_cube_2x2::solver::{self, Solution};
use std::sync::mpsc;

// from_file_formatのエラーケース追加テスト
#[test]
fn test_from_file_format_too_few_lines() {
    let invalid = "     WWWW\nGGGG RRRR BBBB OOOO\n";
    assert!(Cube::from_file_format(invalid).is_err());
}

#[test]
fn test_from_file_format_line1_too_short() {
    let invalid = "     WWW\nGGGG RRRR BBBB OOOO\n     YYYY\n";
    assert!(Cube::from_file_format(invalid).is_err());
}

#[test]
fn test_from_file_format_line2_too_short() {
    let invalid = "     WWWW\nGGGG RRRR BBBB OOO\n     YYYY\n";
    assert!(Cube::from_file_format(invalid).is_err());
}

#[test]
fn test_from_file_format_line3_too_short() {
    let invalid = "     WWWW\nGGGG RRRR BBBB OOOO\n     YYY\n";
    assert!(Cube::from_file_format(invalid).is_err());
}

#[test]
fn test_validate_colors_missing_color() {
    // すべての色が欠けているケース
    let colors = [
        Color::White, Color::White, Color::White, Color::White,
        Color::Yellow, Color::Yellow, Color::Yellow, Color::Yellow,
        Color::Green, Color::Green, Color::Green, Color::Gray, // Greenが1つ少ない
        Color::Blue, Color::Blue, Color::Blue, Color::Blue,
        Color::Red, Color::Red, Color::Red, Color::Red,
        Color::Orange, Color::Orange, Color::Orange, Color::Orange,
    ];
    let result = Cube::validate_colors(&colors);
    assert!(result.is_err());
}

#[test]
fn test_validate_colors_all_wrong() {
    // すべての色が間違っているケース
    let colors = [Color::Gray; 24];
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
    for i in 0..24 {
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
    let c0 = cube.stickers[2];
    let c1 = cube.stickers[9];
    let c2 = cube.stickers[16];
    cube.stickers[2] = c1;
    cube.stickers[9] = c2;
    cube.stickers[16] = c0;

    let solution = Solution { moves: vec![], found: true };
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
