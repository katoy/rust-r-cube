use rubiks_cube_3x3::cube::{Cube, Move};
use rubiks_cube_3x3::kociemba::RawCube;

#[test]
fn test_basic_moves_raw_consistency() {
    let base_moves = [Move::U, Move::R, Move::F, Move::D, Move::L, Move::B];

    for (i, &mv) in base_moves.iter().enumerate() {
        let mut cube = Cube::new();
        cube.apply_move(mv);

        let rc = RawCube::from_cube(&cube);
        match rc {
            Ok(rc_actual) => {
                let rc_expected = RawCube::move_cube(i);
                assert_eq!(
                    rc_actual.cp, rc_expected.cp,
                    "CP mismatch after move {:?}",
                    mv
                );
                assert_eq!(
                    rc_actual.co, rc_expected.co,
                    "CO mismatch after move {:?}",
                    mv
                );
                assert_eq!(
                    rc_actual.ep, rc_expected.ep,
                    "EP mismatch after move {:?}",
                    mv
                );
                assert_eq!(
                    rc_actual.eo, rc_expected.eo,
                    "EO mismatch after move {:?}",
                    mv
                );
            }
            Err(e) => {
                panic!("from_cube failed after move {:?}: {}", mv, e);
            }
        }
    }
}

#[test]
fn test_multiple_moves_consistency() {
    let test_cases = vec![
        vec![Move::U, Move::R],
        vec![Move::R, Move::U],
        vec![Move::F, Move::B, Move::L, Move::R, Move::U, Move::D],
    ];

    for moves in test_cases {
        let mut cube = Cube::new();
        let mut expected_rc = RawCube::default();

        for &mv in &moves {
            cube.apply_move(mv);
            let mv_idx = match mv {
                Move::U => 0,
                Move::R => 1,
                Move::F => 2,
                Move::D => 3,
                Move::L => 4,
                Move::B => 5,
                _ => panic!("Unsupported move in test"),
            };
            expected_rc = expected_rc.multiply(RawCube::move_cube(mv_idx));
        }

        let rc = RawCube::from_cube(&cube);
        match rc {
            Ok(rc_actual) => {
                assert_eq!(
                    rc_actual.cp, expected_rc.cp,
                    "CP mismatch after moves {:?}",
                    moves
                );
                assert_eq!(
                    rc_actual.co, expected_rc.co,
                    "CO mismatch after moves {:?}",
                    moves
                );
                assert_eq!(
                    rc_actual.ep, expected_rc.ep,
                    "EP mismatch after moves {:?}",
                    moves
                );
                assert_eq!(
                    rc_actual.eo, expected_rc.eo,
                    "EO mismatch after moves {:?}",
                    moves
                );
            }
            Err(e) => {
                panic!("from_cube failed after moves {:?}: {}", moves, e);
            }
        }
    }
}
