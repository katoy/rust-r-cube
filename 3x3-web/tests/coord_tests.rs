use rubiks_cube_3x3::cube::{Cube, Move};
use rubiks_cube_3x3::kociemba::coord::{Corner, Edge, RawCube};

#[test]
fn test_raw_cube_initial() {
    let rc = RawCube::default();
    assert_eq!(rc.cp[0], Corner::UFR);
    assert_eq!(rc.co[0], 0);
    assert_eq!(rc.ep[0], Edge::UR);
}

#[test]
fn test_raw_cube_move_u() {
    let mut cube = Cube::new();
    // Use public apply_move instead of private rotation::apply_move
    cube.apply_move(Move::U);

    let rc_from_cube = RawCube::from_cube(&cube).unwrap();

    let identity = RawCube::default();
    let rc_from_move = identity.multiply(RawCube::move_cube(0));

    assert_eq!(rc_from_cube.cp, rc_from_move.cp, "CP mismatch for U");
    assert_eq!(rc_from_cube.co, rc_from_move.co, "CO mismatch for U");
    assert_eq!(rc_from_cube.ep, rc_from_move.ep, "EP mismatch for U");
    assert_eq!(rc_from_cube.eo, rc_from_move.eo, "EO mismatch for U");
}

#[test]
fn test_coordinates_initial() {
    let rc = RawCube::default();
    assert_eq!(rc.get_twist(), 0);
    assert_eq!(rc.get_flip(), 0);
    assert_eq!(rc.get_ud_slice(), 0);
    assert_eq!(rc.get_cp(), 0);
    assert_eq!(rc.get_ep8(), 0);
    assert_eq!(rc.get_slice_p(), 0);
}

#[test]
fn test_twist_symmetry() {
    let mut rc = RawCube::default();
    for twist in 0..2187 {
        rc.set_twist(twist);
        assert_eq!(rc.get_twist(), twist, "Twist failed at {}", twist);
    }
}

#[test]
fn test_flip_symmetry() {
    let mut rc = RawCube::default();
    for flip in 0..2048 {
        rc.set_flip(flip);
        assert_eq!(rc.get_flip(), flip, "Flip failed at {}", flip);
    }
}

#[test]
fn test_slice_symmetry() {
    let mut rc = RawCube::default();
    for slice in 0..495 {
        rc.set_ud_slice(slice);
        assert_eq!(rc.get_ud_slice(), slice, "Slice failed at {}", slice);
    }
}

#[test]
fn test_raw_cube_all_basic_moves() {
    let moves = [Move::U, Move::R, Move::F, Move::D, Move::L, Move::B];
    for (i, &mv) in moves.iter().enumerate() {
        let mut cube = Cube::new();
        cube.apply_move(mv);
        let rc_from_cube = RawCube::from_cube(&cube)
            .unwrap_or_else(|e| panic!("Convert fail for {:?}: {}", mv, e));

        let identity = RawCube::default();
        let rc_from_move = identity.multiply(RawCube::move_cube(i));

        assert_eq!(rc_from_cube.cp, rc_from_move.cp, "CP mismatch for {:?}", mv);
        assert_eq!(rc_from_cube.co, rc_from_move.co, "CO mismatch for {:?}", mv);
        assert_eq!(rc_from_cube.ep, rc_from_move.ep, "EP mismatch for {:?}", mv);
        assert_eq!(rc_from_cube.eo, rc_from_move.eo, "EO mismatch for {:?}", mv);
    }
}
