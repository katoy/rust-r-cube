use rubiks_cube_2x2::cube::{Color, Cube};
use rubiks_cube_2x2::solver::Solution;

#[test]
fn test_apply_orientation_solution_error() {
    let mut cube = Cube::new();
    // 物理的に不可能な状態（捻じれパリティエラー）を作り出す
    // コーナー 0 (UFL) [2, 9, 16] を 120度回転
    let c0 = cube.stickers[2];
    let c1 = cube.stickers[9];
    let c2 = cube.stickers[16];
    cube.stickers[2] = c1;
    cube.stickers[9] = c2;
    cube.stickers[16] = c0;

    let solution = Solution {
        moves: vec![],
        found: true,
    };
    // apply_orientation_solution は内部で restore_orientation_instantly -> is_valid_state
    // を呼び出し、パリティエラーを検出するはず。
    let result = cube.apply_orientation_solution(&solution);
    assert!(result.is_err());
    let msg_str = result.unwrap_err().to_string();
    assert!(msg_str.contains("向きが無効"));
}

#[test]
fn test_to_file_format_with_gray() {
    let mut cube = Cube::new();
    // Internal API to set color to Gray (which is normally not allowed in valid cube but possible in memory)
    // set_sticker_color is public
    cube.set_sticker_color(0, Color::Gray);

    // to_file_format handles Gray by outputting a space
    let s = cube.to_file_format();

    // Check if the output contains expected representation
    // Face 0 (Up) is the first block.
    // formatted string has 3 lines.
    // Line 1: 5 spaces + 4 chars + ...
    assert!(s.contains("\n     ")); // Basic check
}
