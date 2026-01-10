use rubiks_cube_2x2::cube::{Cube, Move};
use rubiks_cube_2x2::solver;

#[test]
fn test_coverage_gap_cube_default() {
    let cube = <Cube as Default>::default();
    assert!(cube.is_solved());
}

#[test]
fn test_coverage_gap_solver_early_breaks() {
    let cube = Cube::new();
    // 既に解決されている場合
    let sol = solver::solve(&cube, 11, true);
    assert!(sol.found);
    assert_eq!(sol.moves.len(), 0);
}

#[test]
fn test_coverage_gap_solver_forward_queue_empty() {
    let cube = Cube::new();
    let mut scrambled = cube.clone();
    scrambled.apply_move(Move::R);

    // 順方向探索で解決に至るケース
    let sol = solver::solve(&scrambled, 1, true);
    assert!(sol.found);
}

#[test]
fn test_coverage_gap_solver_backward_visited_collision() {
    // 逆方向探索の初期化などのカバレッジ用
    let cube = Cube::new();
    let sol = solver::solve(&cube, 11, false);
    assert!(sol.found);
}

#[test]
fn test_coverage_gap_solver_not_found_with_progress() {
    use rubiks_cube_2x2::cube::{Cube, Move};
    use rubiks_cube_2x2::solver;
    use std::sync::mpsc;

    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);

    // 深度1では解けないプログレス付き探索
    // これにより solve_internal の最後の failure path (progress_tx.send(1.0)) が実行される
    let (tx, rx) = mpsc::channel();
    let solution = solver::solve_with_progress(&cube, 1, false, Some(tx));

    assert!(!solution.found);

    // 完了通知(1.0)が送られているはず
    let progress: Vec<f32> = rx.into_iter().collect();
    assert!(progress.contains(&1.0));
}
