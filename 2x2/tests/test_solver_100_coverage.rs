use rubiks_cube_2x2::cube::{Cube, Move};
use rubiks_cube_2x2::solver::{solve, solve_with_progress};
use std::sync::mpsc;

#[test]
fn test_solver_coverage_hits() {
    // 1. Solve at distance 1 with max_depth 2 (Hits line 202: send(1.0) in initiation)
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    let (tx, rx) = mpsc::channel();
    let sol = solve_with_progress(&cube, 2, false, Some(tx));
    assert!(sol.found);
    let progress: Vec<f32> = rx.into_iter().collect();
    assert!(progress.contains(&1.0));

    // 2. Solve at distance 3 with max_depth 4 (Hits line 237: send(1.0) in backward loop)
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);
    cube.apply_move(Move::F);
    let (tx, rx) = mpsc::channel();
    // forward_depth = 2, backward_depth = 2.
    // backward BFS will find it at depth 1.
    let _sol = solve_with_progress(&cube, 4, false, Some(tx));
    let progress: Vec<f32> = rx.into_iter().collect();
    assert!(progress.contains(&1.0));

    // 3. No solution within max_depth (Hits line 243: continue at max backward depth)
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);
    cube.apply_move(Move::F);
    cube.apply_move(Move::L);
    cube.apply_move(Move::B); // 5 moves away
    let sol = solve(&cube, 2, false);
    assert!(!sol.found);

    // 4. Ignore orientation in backward loop (Hits line 256)
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);
    let sol = solve(&cube, 4, true);
    assert!(sol.found);
}

#[test]
fn test_solver_progress_intervals() {
    // Hits progress update interval (line 148, 221)
    let mut cube = Cube::new();
    for _ in 0..5 {
        cube.scramble(1);
    }
    let (tx, rx) = mpsc::channel();
    let _sol = solve_with_progress(&cube, 6, false, Some(tx));
    let _progress: Vec<f32> = rx.into_iter().collect();
}
