use rubiks_cube_3x3::cube::{Cube, Move};
use rubiks_cube_3x3::solver::get_orientations_vec;

fn main() {
    let mut cube = Cube::new();
    // (U Dp Ep) rotates Up CW (1) and Down CCW (3)
    let seq = vec![Move::U, Move::Dp, Move::Ep];
    for &m in &seq {
        cube.apply_move(m);
    }
    println!("Initial state (after sequence):");
    println!("  Solved (colors): {}", cube.is_solved());
    println!("  Orientations: {:?}", get_orientations_vec(&cube));

    let sol = rubiks_cube_3x3::solver::solve(&cube, 24, false);
    println!("Solver found solution: {}", sol.found);
    if sol.found {
        let mut final_cube = cube.clone();
        for &mv in &sol.moves {
            final_cube.apply_move(mv);
        }
        println!("Final state:");
        println!("  Solved (colors): {}", final_cube.is_solved());
        println!(
            "  Fully Solved: {}",
            final_cube.is_solved_with_orientation()
        );
        println!("  Orientations: {:?}", get_orientations_vec(&final_cube));
    }
}
