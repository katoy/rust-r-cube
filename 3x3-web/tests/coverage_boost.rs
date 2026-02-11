use rubiks_cube_3x3::cube::validation::{
    check_corner_parity, check_edge_parity, CENTER_STICKERS, CORNER_STICKERS, EDGE_STICKERS,
};
use rubiks_cube_3x3::cube::{Color, Cube, Face, Move};
use rubiks_cube_3x3::kociemba::coord::FaceCube;
use rubiks_cube_3x3::kociemba::RawCube;
use rubiks_cube_3x3::solver;

#[test]
fn test_enums_comprehensive() {
    for m in [
        Move::M,
        Move::E,
        Move::S,
        Move::X,
        Move::Y,
        Move::Z,
        Move::U,
    ] {
        let _ = m.geometric_params();
    }
    assert_eq!(Move::M2.split_to_single(), Some(Move::M));
    assert_eq!(Move::E2.split_to_single(), Some(Move::E));
    assert_eq!(Move::S2.split_to_single(), Some(Move::S));
}

#[test]
fn test_cube_io_and_colors() {
    let mut colors = [Color::White; 54];
    colors[0] = Color::Yellow;
    assert!(Cube::from_colors(&colors).is_err());
    assert!(Cube::from_file_format("").is_err());
}

#[test]
fn test_validation_errors() {
    let mut cube = Cube::new();
    cube.stickers[7].color = cube.stickers[5].color;
    cube.stickers[37].color = cube.stickers[28].color;
    assert!(check_edge_parity(&cube).is_err());

    let mut cube2 = Cube::new();
    let idxs = CORNER_STICKERS[0];
    cube2.stickers[idxs[1]].color = Color::Yellow;
    assert!(check_corner_parity(&cube2).is_err());

    let mut cube3 = Cube::new();
    let idxs2 = CORNER_STICKERS[2];
    cube3.stickers[idxs2[0]].color = Color::Gray;
    cube3.stickers[idxs2[1]].color = Color::Red;
    cube3.stickers[idxs2[2]].color = Color::Blue;
    assert!(check_corner_parity(&cube3).is_err());
}

#[test]
fn test_mod_restore_errors() {
    let mut cube1 = Cube::new();
    cube1.stickers[CENTER_STICKERS[0]].color = Color::Gray;
    assert!(cube1.restore_orientation_instantly().is_err());

    let mut cube2 = Cube::new();
    cube2.stickers[CORNER_STICKERS[0][0]].color = Color::Gray;
    assert!(cube2.restore_orientation_instantly().is_err());

    let mut cube3 = Cube::new();
    cube3.stickers[EDGE_STICKERS[0][0]].color = Color::Gray;
    assert!(cube3.restore_orientation_instantly().is_err());
}

#[test]
fn test_solver_debug_and_parity() {
    std::env::set_var("SOLVER_DEBUG", "1");
    let mut cube = Cube::new();
    // 奇数パリティの完成状態 (物理的に不可能)
    cube.stickers[CENTER_STICKERS[0]].orientation = 1;
    let _ = solver::solve(&cube, 10, false);
    let _ = solver::solve(&cube, 10, true);

    // 探索失敗パス
    let mut scrambled = Cube::new();
    scrambled.scramble(20);
    let _ = solver::solve(&scrambled, 1, false);

    // センター不整合 get_target_oris fallback
    let mut cube2 = Cube::new();
    cube2.stickers[CENTER_STICKERS[0]].color = Color::Gray;
    let _ = solver::solve(&cube2, 1, false);

    std::env::remove_var("SOLVER_DEBUG");
}

#[test]
fn test_coord_comprehensive() {
    let cube = Cube::new();
    let fc = FaceCube::from_cube(&cube);
    assert_eq!(fc.f.len(), 54);

    let mut cube2 = Cube::new();
    let idxs = CORNER_STICKERS[0];
    let (c0, c1, c2) = (
        cube2.stickers[idxs[0]].color,
        cube2.stickers[idxs[1]].color,
        cube2.stickers[idxs[2]].color,
    );

    cube2.stickers[idxs[0]].color = c2;
    cube2.stickers[idxs[1]].color = c0;
    cube2.stickers[idxs[2]].color = c1;
    let rc1 = RawCube::from_cube(&cube2).unwrap();
    assert_eq!(rc1.co[0], 1);

    cube2.stickers[idxs[0]].color = c1;
    cube2.stickers[idxs[1]].color = c2;
    cube2.stickers[idxs[2]].color = c0;
    let rc2 = RawCube::from_cube(&cube2).unwrap();
    assert_eq!(rc2.co[0], 2);
}

#[test]
fn test_solver_helpers() {
    assert!(solver::is_opposite_face(Face::Up, Face::Down));
    use rubiks_cube_3x3::solver::get_buffer_face;
    let _ = get_buffer_face(Face::Up, Face::Front);
}
