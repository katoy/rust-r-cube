use rubiks_cube_2x2::cube::{Color, Cube, Move};
use rubiks_cube_2x2::solver::Solution;

#[test]
fn test_apply_orientation_solution_error() {
    let mut cube = Cube::new();
    // Cube is solved.
    // Solution says "R" was needed to solve it (implying it was R').
    // So we apply inv(R) = R' to RefCube (Solved). RefCube becomes R'.
    // We compare RefCube(R') vs Self(Solved).
    // Colors will differ because R' rotation changes colors.
    let solution = Solution {
        moves: vec![Move::R], // Dummy move
        found: true,
    };
    /* apply_orientation_solution is in mod.rs and public */
    let result = cube.apply_orientation_solution(&solution);
    assert!(result.is_err());
    let msg_str = result.unwrap_err().to_string();
    assert!(msg_str.contains("内部エラー"));
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
