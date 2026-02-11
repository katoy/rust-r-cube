use rubiks_cube_3x3::cube::{Color, Cube, Face, Move};
use rubiks_cube_3x3::kociemba::coord::{Corner, Edge, RawCube};
use rubiks_cube_3x3::kociemba::search::{idx_to_move, is_redundant};
use rubiks_cube_3x3::kociemba::{PruningTable, Search};
use rubiks_cube_3x3::solver::{
    apply_rot_to_face, apply_supercube_fixes, get_buffer_face, get_setup_to_up, is_opposite_face,
    solve, SolverState,
};

// ==================== SolverState Tests ====================

#[test]
fn test_solver_state_lifecycle() {
    let cube = Cube::new();
    let mut state = SolverState::new(&cube, 20, false);
    assert!(state.error().is_none());
    assert_eq!(state.estimate_progress(), 0.5);

    state.process_chunk(1);
    assert!(state.finished);
    assert_eq!(state.estimate_progress(), 1.0);
    assert!(state.get_solution().is_some());

    // 完了後の複数回呼び出し
    let (p, f) = state.process_chunk(1);
    assert_eq!(p, 0);
    assert!(f);
}

#[test]
fn test_solver_state_scrambled() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    let mut state = SolverState::new(&cube, 20, false);
    state.process_chunk(1);
    assert!(state.get_solution().unwrap().found);
}

// ==================== Kociemba Coord & Table Tests ====================

#[test]
fn test_raw_cube_initial() {
    let rc = RawCube::default();
    assert_eq!(rc.cp[0], Corner::UFR);
    assert_eq!(rc.co[0], 0);
    assert_eq!(rc.ep[0], Edge::UR);
}

#[test]
fn test_raw_cube_symmetry() {
    let mut rc = RawCube::default();
    // Twist
    for t in [0, 100, 2186] {
        rc.set_twist(t);
        assert_eq!(rc.get_twist(), t);
    }
    // Flip
    for f in [0, 50, 2047] {
        rc.set_flip(f);
        assert_eq!(rc.get_flip(), f);
    }
    // Slice
    for s in [0, 30, 494] {
        rc.set_ud_slice(s);
        assert_eq!(rc.get_ud_slice(), s);
    }
}

#[test]
fn test_pruning_tables_initial() {
    let pruning = PruningTable::get();
    assert_eq!(pruning.twist_slice[0], 0);
    assert_eq!(pruning.flip_slice[0], 0);
    assert_eq!(pruning.cp_slice[0], 0);
    assert_eq!(pruning.ep8_slice[0], 0);
}

#[test]
fn test_search_helpers() {
    assert_eq!(idx_to_move(0), Move::U);
    assert!(!is_redundant(3, 0)); // U then D is OK
    assert!(is_redundant(0, 3)); // D then U is redundant
}

// ==================== Solver Coordination & Fixes Tests ====================

#[test]
fn test_opposite_face_checks() {
    assert!(is_opposite_face(Face::Up, Face::Down));
    assert!(!is_opposite_face(Face::Up, Face::Front));
}

#[test]
fn test_get_setup_to_up() {
    for f in Face::all() {
        let setup = get_setup_to_up(f);
        let mut cube = Cube::new();
        for &m in &setup {
            cube.apply_move(m);
        }
        assert_eq!(apply_rot_to_face(f, &setup), Face::Up);
    }
}

#[test]
fn test_get_buffer_face_all() {
    let pairs = [
        (Face::Up, Face::Down),
        (Face::Front, Face::Back),
        (Face::Left, Face::Right),
    ];
    for (f1, f2) in pairs {
        let buffer = get_buffer_face(f1, f2);
        assert!(!is_opposite_face(f1, buffer));
        assert!(!is_opposite_face(f2, buffer));
    }
}

// ==================== Debug Logging Coverage (SOLVER_DEBUG=1) ====================

#[test]
fn test_solver_debug_all_paths() {
    std::env::set_var("SOLVER_DEBUG", "1");
    let mut cube = Cube::new();

    // Path 1: Solved
    let _ = solve(&cube, 20, false);

    // Path 2: Color solved, orientation mismatch
    cube.stickers[4].orientation = 2;
    let _ = solve(&cube, 20, false);

    // Path 3: Scrambled
    let mut cube2 = Cube::new();
    cube2.apply_move(Move::R);
    let _ = solve(&cube2, 20, false);

    // Path 4: Fixes trigger
    let mut cube3 = Cube::new();
    cube3.stickers[4].orientation = 1;
    cube3.stickers[13].orientation = 1;
    let mut search = Search::new();
    let _ = apply_supercube_fixes(&cube3, &mut search);

    std::env::remove_var("SOLVER_DEBUG");
}

// ==================== Edge Case & Error Coverage ====================

#[test]
fn test_solver_error_conditions() {
    let cube = Cube::new();
    // Depth 0 on solved cube
    assert!(solve(&cube, 0, false).found);

    // Invalid cube state
    let mut invalid = cube.clone();
    invalid.stickers[0].color = Color::Yellow;
    assert!(!solve(&invalid, 20, false).found);

    // Depth exceeded
    let mut scrambled = cube.clone();
    scrambled.scramble(10);
    let res = solve(&scrambled, 1, false);
    assert!(!res.found);
    assert!(res.message.contains("探索深度") || res.message.contains("解が見つかりません"));
}
