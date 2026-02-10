use rubiks_cube_3x3::cube::{Cube, Face, Move};
use rubiks_cube_3x3::solver::{
    get_orientations_vec, is_fully_solved, is_orientation_solvable, solve,
};

/// Test debug logging paths in is_fully_solved
#[test]
fn test_solver_debug_logging() {
    std::env::set_var("SOLVER_DEBUG", "1");

    let cube = Cube::new();
    assert!(is_fully_solved(&cube));

    // Test unmatched orientation debug path
    let mut cube2 = Cube::new();
    cube2.stickers[4].orientation = 11; // Invalid orientation
    assert!(!is_fully_solved(&cube2));

    std::env::remove_var("SOLVER_DEBUG");
}

/// Test orientation parity error cases
#[test]
fn test_orientation_parity_errors() {
    let mut cube = Cube::new();
    // Find Up center piece and rotate it manually
    // Piece at (0, 1, 0) is the Up center
    for piece in &mut cube.pieces {
        if piece.current_pos.y.round() as i8 == 1
            && piece.current_pos.x.round() as i8 == 0
            && piece.current_pos.z.round() as i8 == 0
        {
            piece.rotate(glam::Vec3::Y, std::f32::consts::FRAC_PI_2);
            break;
        }
    }
    cube.sync_stickers();

    assert!(!is_orientation_solvable(&cube));

    let oris = get_orientations_vec(&cube);
    let sum: u32 = oris.iter().map(|&o| o as u32).sum();
    assert_eq!(sum % 2, 1, "Should have odd parity");

    // Try to solve - should fail with parity message
    let solution = solve(&cube, 20, false);
    assert!(!solution.found);
    assert!(
        solution.message.contains("方位パリティが異常") || solution.message.contains("パリティ")
    );
}

/// Test ignore_orientation flag with parity errors
#[test]
fn test_ignore_orientation_with_parity() {
    std::env::set_var("SOLVER_DEBUG", "1");

    let mut cube = Cube::new();
    // Apply some moves
    cube.apply_move(Move::U);
    cube.apply_move(Move::R);

    // Manually break parity
    cube.stickers[Face::Up.start_index() + 4].orientation = 1;

    // With ignore_orientation=true, should get color-only solution
    let _solution = solve(&cube, 20, true);

    // May or may not find a solution depending on whether color can be solved
    // The important thing is testing the code path

    std::env::remove_var("SOLVER_DEBUG");
}

/// Test max_depth exceeded case with orientation fixes
#[test]
fn test_max_depth_exceeded_with_fixes() {
    let mut cube = Cube::new();
    cube.apply_move(Move::U);
    cube.apply_move(Move::Ep); // This creates center orientation issue

    // Very low depth - should hit the max_depth limit after fixes
    let solution = solve(&cube, 2, false);

    // Should either find a solution or not, but this tests the depth check path
    if solution.found {
        assert!(solution.moves.len() <= 2);
    }
}

/// Test solve with SOLVER_DEBUG enabled for all paths
#[test]
fn test_solve_with_debug_all_paths() {
    std::env::set_var("SOLVER_DEBUG", "1");

    // Test 1: Simple solve
    let mut cube1 = Cube::new();
    cube1.apply_move(Move::U);
    let _ = solve(&cube1, 20, false);

    // Test 2: Scrambled cube
    let mut cube2 = Cube::new();
    cube2.scramble(10);
    let _ = solve(&cube2, 30, false);

    // Test 3: Cube with center rotation
    let mut cube3 = Cube::new();
    cube3.apply_move(Move::U);
    cube3.apply_move(Move::Dp);
    cube3.apply_move(Move::Ep);
    let _ = solve(&cube3, 30, false);

    std::env::remove_var("SOLVER_DEBUG");
}

/// Test color_only_solution saving when ignore_orientation=false
#[test]
fn test_color_only_solution_ignore_false() {
    // Create a state where color is solvable but orientation has issues
    let mut cube = Cube::new();
    cube.apply_move(Move::U);

    // Break parity
    cube.stickers[Face::Up.start_index() + 4].orientation = 1;

    let solution = solve(&cube, 15, false);
    // The solver may or may not find a solution depending on the internal logic
    // The key is testing the code path for color_only_solution handling
    if !solution.found {
        assert!(solution.message.contains("パリティ") || solution.message.contains("解決"));
    }
}

/// Test various edge cases in try_solve_with_rotation
#[test]
fn test_edge_cases_comprehensive() {
    std::env::set_var("SOLVER_DEBUG", "1");

    // Test boundary conditions
    let cube = Cube::new();

    // Test with very low depth
    let sol1 = solve(&cube, 0, false);
    assert!(sol1.found); // Already solved

    // Test with scrambled cube and low depth
    let mut cube2 = Cube::new();
    cube2.apply_move(Move::U);
    cube2.apply_move(Move::R);
    cube2.apply_move(Move::F);
    let _sol2 = solve(&cube2, 1, false);
    // May or may not find solution, but tests the path

    std::env::remove_var("SOLVER_DEBUG");
}

/// Test final_cube orientation check paths
#[test]
fn test_final_orientation_checks() {
    std::env::set_var("SOLVER_DEBUG", "1");

    let mut cube = Cube::new();
    // Create a state that needs orientation fixes
    cube.apply_move(Move::U);
    cube.apply_move(Move::Dp);
    cube.apply_move(Move::Ep);
    cube.apply_move(Move::E);

    let solution = solve(&cube, 50, false);

    if solution.found {
        let mut verify = cube.clone();
        for &m in &solution.moves {
            verify.apply_move(m);
        }
        assert!(is_fully_solved(&verify));
    }

    std::env::remove_var("SOLVER_DEBUG");
}
