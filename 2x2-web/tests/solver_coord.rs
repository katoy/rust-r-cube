use rubiks_cube_2x2::cube::{Color, Cube};
use rubiks_cube_2x2::solver::coord::RawCube;

#[test]
fn test_coord_extra_coverage() {
    // Transferred from src/solver/coord.rs
    // Line 89: Invalid corner set
    let mut cube = Cube::new();
    // マニュアルでステッカーの色をいじって物理的に不可能なコーナーを作る
    cube.set_sticker_color(16, Color::Blue);
    let _ = RawCube::from_cube(&cube, &[0, 1, 2, 3, 4, 5]);

    // Lines 94-97: No U/D color
    let mut cube_no_ud = Cube::new();
    for i in 0..24 {
        cube_no_ud.set_sticker_color(i, Color::Red);
    }
    let _ = RawCube::from_cube(&cube_no_ud, &[0, 1, 2, 3, 4, 5]);
}
