use rubiks_cube_2x2::cube::{Cube, Move};
use rubiks_cube_2x2::solver::coord::RawCube;

fn main() {
    let mut cube = Cube::new();
    let moves = Move::all_moves();

    for (i, &mv) in moves.iter().enumerate() {
        let mut test_cube = cube.clone();
        test_cube.apply_move(mv);
        let rc_from_cube = RawCube::from_cube(&test_cube, &[0, 1, 2, 3, 4, 5]).unwrap();

        let mut rc_simulated = RawCube::default();
        // m_idx mapping: R(0), Rp(1), R2(2), L(3), ...
        // MoveTable uses 0..18 where 0..3 is U, 3..6 is R, etc.?
        // Need to check how move_cube_18 is indexed.
        println!("Move {}: {:?} -> {:?}", i, mv, rc_from_cube);
    }
}
