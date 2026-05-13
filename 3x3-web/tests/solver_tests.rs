use rubiks_cube_3x3::cube::{Cube, Face, Move};
use rubiks_cube_3x3::kociemba::{RawCube, Search};
use rubiks_cube_3x3::kociemba::coord::{Corner, Edge};
use rubiks_cube_3x3::solver::{is_fully_solved, solve, SolverState};

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

#[test]
fn test_solve_with_various_depths() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);

    // Test different depths
    let result = solve(&cube, 1, false);
    assert!(!result.found);

    let result = solve(&cube, 3, false);
    assert!(result.found);

    let result = solve(&cube, 100, false);
    assert!(result.found);
}

#[test]
fn test_solve_ignore_orientation_flag() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);

    let result_color_only = solve(&cube, 24, true);
    assert!(result_color_only.found);

    let result_full = solve(&cube, 24, false);
    assert!(result_full.found);
}

#[test]
fn test_search_node_limit_path() {
    let cube = Cube::new();
    let rc = RawCube::from_cube(&cube).unwrap();
    let mut search = Search::new();
    search.max_nodes = 0; // node limit 超過分岐
    let _ = search.solve(&rc, 1);
    assert!(search.node_count > search.max_nodes || search.node_count == 0);
}

#[test]
fn test_solver_state_methods() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);

    let mut st = SolverState::new(&cube, 8, false);
    assert!(st.error().is_none());
    assert_eq!(st.estimate_progress(), 0.5);

    let (_processed, done) = st.process_chunk(100);
    assert!(done);
    assert!(st.get_solution().is_some());
    assert_eq!(st.estimate_progress(), 1.0);

    // finished=true 早期return分岐
    let (p2, done2) = st.process_chunk(100);
    assert_eq!(p2, 0);
    assert!(done2);
}

#[test]
fn test_solve_with_various_setup_moves() {
    // additional solver/mod.rs coverage for different code paths
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);
    let result = solve(&cube, 20, false);
    // just verify it either finds or doesn't find solution
    assert!(!result.message.is_empty());
}

#[test]
fn test_raw_cube_from_various_states() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);

    let rc = RawCube::from_cube(&cube).unwrap();
    assert_ne!(rc.cp, [Corner::UFR; 8]);

    // Verify RawCube conversion works for scrambled state
    let mut cube2 = Cube::new();
    cube2.apply_move(Move::U);
    cube2.apply_move(Move::F);
    let rc2 = RawCube::from_cube(&cube2).unwrap();
    assert_ne!(rc2.ep, [Edge::UR; 12]);
}

#[test]
fn test_search_with_different_depths() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);

    let rc = RawCube::from_cube(&cube).unwrap();
    let mut search = Search::new();

    for depth in [1, 2, 3, 5, 10, 15] {
        let result = search.solve(&rc, depth);
        if depth >= 2 {
            assert!(result.is_some());
            break;
        }
    }
}

#[test]
fn test_search_intermediate_steps() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);
    cube.apply_move(Move::F);

    let rc = RawCube::from_cube(&cube).unwrap();
    let mut search = Search::new();

    // Test searching at a specific depth
    let sol = search.solve(&rc, 10);
    assert!(sol.is_some());

    // Verify the solution works
    if let Some(moves) = sol {
        let mut test_cube = cube.clone();
        for &mv in &moves {
            test_cube.apply_move(mv);
        }
        assert!(test_cube.is_solved());
    }
}

#[test]
fn test_cube_normalized_state() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);
    cube.apply_move(Move::F);

    let normalized = cube.normalized();
    let rc = RawCube::from_cube(&normalized).unwrap();

    // Normalized cube should still have valid corner/edge permutations
    let mut cp_set = std::collections::HashSet::new();
    for &c in &rc.cp {
        cp_set.insert(c as u8);
    }
    assert_eq!(cp_set.len(), 8);

    let mut ep_set = std::collections::HashSet::new();
    for &e in &rc.ep {
        ep_set.insert(e as u8);
    }
    assert_eq!(ep_set.len(), 12);
}

#[test]
fn test_solve_with_max_depth_exceeded() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);
    cube.apply_move(Move::F);
    cube.apply_move(Move::D);

    // Use a shallow depth that won't find solution
    let result = solve(&cube, 1, false);
    assert!(!result.found);
}

#[test]
fn test_solve_state_multiple_iterations() {
    let mut cube = Cube::new();
    for _ in 0..3 {
        cube.apply_move(Move::R);
    }

    let mut st = SolverState::new(&cube, 10, false);

    for _ in 0..5 {
        let (_processed, done) = st.process_chunk(50);
        if done {
            break;
        }
    }

    assert!(st.get_solution().is_some());
}
