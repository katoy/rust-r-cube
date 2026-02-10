use rubiks_cube_3x3::cube::{Cube, Face, Move};
use rubiks_cube_3x3::solver::{
    apply_rot_to_face, get_setup_to_up, is_fully_solved, is_opposite_face, solve,
};

#[test]
fn test_solve_normal_cube() {
    let cube = Cube::new();
    let result = solve(&cube, 20, false);
    assert!(result.found);
    assert_eq!(result.moves.len(), 0);
}

#[test]
fn test_solve_scrambled_cube() {
    let mut cube = Cube::new();
    cube.apply_move(Move::U);
    cube.apply_move(Move::R);

    let result = solve(&cube, 20, false);
    assert!(result.found);

    for m in result.moves {
        cube.apply_move(m);
    }
    assert!(is_fully_solved(&cube));
}

#[test]
fn test_is_opposite_face_coverage() {
    assert!(is_opposite_face(Face::Up, Face::Down));
    assert!(is_opposite_face(Face::Left, Face::Right));
    assert!(is_opposite_face(Face::Front, Face::Back));
}

#[test]
fn test_get_setup_to_up_all_faces() {
    for f in Face::all() {
        let setup = get_setup_to_up(f);
        let mut cube = Cube::new();
        for &m in &setup {
            cube.apply_move(m);
        }
        let res = apply_rot_to_face(f, &setup);
        assert_eq!(
            res,
            Face::Up,
            "Face {:?} should be Up after setup {:?}",
            f,
            setup
        );
    }
}

#[test]
fn test_solve_internal_edge_cases() {
    let cube = Cube::new();
    let sol_depth = solve(&cube, 0, false);
    assert!(sol_depth.found);

    let mut c = Cube::new();
    c.stickers[4].orientation = 1; // 色は揃っているが方位が違う
    let result = solve(&c, 1, false);
    assert!(!result.found);
}
