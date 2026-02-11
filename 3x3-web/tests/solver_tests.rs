use rubiks_cube_3x3::cube::{Cube, Face, Move};
use rubiks_cube_3x3::kociemba::{RawCube, Search};
use rubiks_cube_3x3::solver::{is_fully_solved, solve};

fn assert_solve_helper(
    setup_moves: &[Move],
    max_depth: usize,
    ignore_orientation: bool,
    expected_full_solve: bool,
) {
    let mut cube = Cube::new();
    for &mv in setup_moves {
        cube.apply_move(mv);
    }

    let solution = solve(&cube, max_depth, ignore_orientation);
    assert!(
        solution.found,
        "解が見つかるはずです (ignore: {})",
        ignore_orientation
    );

    for &mv in &solution.moves {
        cube.apply_move(mv);
    }
    assert!(cube.is_solved(), "色が揃っているはずです");

    if expected_full_solve {
        let normalized = cube.with_clockwise_orientations();
        assert!(is_fully_solved(&normalized), "向きも揃っているはずです");
    }
}

#[test]
fn test_solve_random_scramble_6_moves() {
    let moves = [Move::R, Move::U, Move::F, Move::L, Move::D, Move::B];
    assert_solve_helper(&moves, 24, true, false);
}

#[test]
fn test_solve_random_scramble_20_moves() {
    let moves = [
        Move::U,
        Move::R,
        Move::F,
        Move::D,
        Move::L,
        Move::B,
        Move::U2,
        Move::R2,
        Move::F2,
        Move::D2,
        Move::L2,
        Move::B2,
        Move::Up,
        Move::Rp,
        Move::Fp,
        Move::Dp,
        Move::Lp,
        Move::Bp,
        Move::R,
        Move::U,
    ];
    assert_solve_helper(&moves, 24, true, false);
}

#[test]
fn test_solve_superflip() {
    let moves = [
        Move::U,
        Move::R2,
        Move::F,
        Move::B,
        Move::R,
        Move::B2,
        Move::R,
        Move::U2,
        Move::L,
        Move::B2,
        Move::R,
        Move::Up,
        Move::Dp,
        Move::R2,
        Move::F,
        Move::Rp,
        Move::L,
        Move::B2,
        Move::U2,
        Move::F2,
    ];
    assert_solve_helper(&moves, 24, true, false);
    assert_solve_helper(&moves, 64, false, true);
}

#[test]
fn test_repro_90_pair() {
    let mut cube = Cube::new();
    cube.stickers[Face::Up.start_index() + 4].orientation = 1;
    cube.stickers[Face::Front.start_index() + 4].orientation = 3;
    cube.force_sync_orientation_to_pieces();
    let sol = solve(&cube, 64, false);
    assert!(sol.found);
    let mut final_cube = cube.clone();
    for &m in &sol.moves {
        final_cube.apply_move(m);
    }
    assert!(is_fully_solved(&final_cube));
}

#[test]
fn test_repro_impossible_parity() {
    let mut cube = Cube::new();
    cube.stickers[Face::Up.start_index() + 4].orientation = 1;
    cube.force_sync_orientation_to_pieces();
    let sol = solve(&cube, 64, false);
    assert!(!sol.found, "Single 90-degree center should be impossible");
}

#[test]
fn test_superflip_kociemba_direct() {
    let content =
        "          WOWGWBWRW\nGWGOGRGYG RWRGRBRYR BWBRBOBYB OWOBOGOYO\n          YRYGYBYOY";
    let cube = Cube::from_file_format(content).unwrap();
    let rc = RawCube::from_cube(&cube).unwrap();
    let mut search = Search::default();
    let result = search.solve(&rc, 22);
    assert!(result.is_some());
}
