use rubiks_cube_2x2::cube::{Cube, Move};
use rubiks_cube_2x2::solver;
use std::sync::mpsc;

#[test]
fn test_solver_full_coverage_targeted() {
    // 1. is_fully_solved のテスト
    let cube_solved = Cube::new();
    assert!(solver::is_fully_solved(&cube_solved));
    let mut cube_moved = Cube::new();
    cube_moved.apply_move(Move::R);
    assert!(!solver::is_fully_solved(&cube_moved));

    // 2. solve (3引数) のテスト
    let solution_basic = solver::solve(&cube_moved, 2, true);
    assert!(solution_basic.found);

    // 3. solve_with_progress のテスト
    let (tx, rx) = mpsc::channel();
    let solution_prog = solver::solve_with_progress(&cube_moved, 2, true, Some(tx));
    assert!(solution_prog.found);
    while let Ok(_) = rx.try_recv() {} // consume messages

    // 4. break (queue empty) のテスト
    // total_depth = 11 手くらいを指定し、完成状態以外の状態から探索を始め、
    // ターゲットが「絶対に到達できない状態」になるようにすれば、全探索して break する。
    // しかしターゲットは全解決状態(24通り)なので、必ず見つかってしまう。
    // そのため、max_depth を 1 未満などの極端な値にするか...
    // いや、forward_depth が 0 になるように、max_depth = 0 または 1 を指定した場合は
    // BFSループに入らずに終わることが多い。

    // ループ内の break (143行目) を通すには、
    // level_size == 0 になる必要がある。
    // BFSの第N層が空になる = その接続成分をすべて探索しきった。
    // 2x2は連結なので、全状態（120万〜360万）を探索しきれば空になる。
}
