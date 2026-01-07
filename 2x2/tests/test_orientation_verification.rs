use rubiks_cube_2x2::cube::{Cube, Move};
use rubiks_cube_2x2::solver;

/// 向きを揃える解法のテスト
#[test]
fn test_orientation_alignment_verification() {
    let mut cube = Cube::new();

    // 簡単なスクランブル
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);

    // 向きも揃える解法を探索
    let solution = solver::solve(&cube, 14, false);

    assert!(solution.found, "解が見つかるべき");

    // 解法を適用
    let mut test_cube = cube.clone();
    for &mv in &solution.moves {
        test_cube.apply_move(mv);
    }

    // 完全に解けているか確認
    assert!(
        solver::is_fully_solved(&test_cube),
        "解法適用後、完全に解けているべき。解法: {:?}",
        solution.moves
    );

    // 初期状態と一致するか確認
    let initial_cube = Cube::new();
    assert_eq!(test_cube, initial_cube, "解法適用後、初期状態と一致すべき");
}

/// より複雑なスクランブルでのテスト
#[test]
fn test_complex_orientation_alignment() {
    let mut cube = Cube::new();

    // より複雑なスクランブル
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);
    cube.apply_move(Move::F);
    cube.apply_move(Move::R);

    // 向きも揃える解法を探索
    let solution = solver::solve(&cube, 14, false);

    assert!(solution.found, "解が見つかるべき");

    // 解法を適用
    let mut test_cube = cube.clone();
    for &mv in &solution.moves {
        test_cube.apply_move(mv);
    }

    // 完全に解けているか確認
    assert!(
        solver::is_fully_solved(&test_cube),
        "解法適用後、完全に解けているべき。解法: {:?}",
        solution.moves
    );
}
