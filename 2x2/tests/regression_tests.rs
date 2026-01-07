use rubiks_cube_2x2::cube::{Cube, Move};
use rubiks_cube_2x2::solver;

/// リグレッションテスト: Entry APIのバグを検出
///
/// このテストは、複数の手を使ったスクランブル後に解法を探索します。
/// Entry APIの実装に問題があると、このテストが失敗します。
#[test]
fn test_solve_multiple_scrambles_regression() {
    // テストケース1: 3手のスクランブル
    let mut cube1 = Cube::new();
    cube1.apply_move(Move::R);
    cube1.apply_move(Move::U);
    cube1.apply_move(Move::F);

    let solution1 = solver::solve(&cube1, 11, true);
    assert!(solution1.found, "3手のスクランブル後に解が見つからない");

    // 解を適用して完成状態になることを確認
    let mut check_cube1 = cube1.clone();
    for &mv in &solution1.moves {
        check_cube1.apply_move(mv);
    }
    assert!(check_cube1.is_solved(), "解を適用しても完成状態にならない");

    // テストケース2: 4手のスクランブル
    let mut cube2 = Cube::new();
    cube2.apply_move(Move::R);
    cube2.apply_move(Move::U);
    cube2.apply_move(Move::R);
    cube2.apply_move(Move::U);

    let solution2 = solver::solve(&cube2, 11, true);
    assert!(solution2.found, "4手のスクランブル後に解が見つからない");

    // 解を適用して完成状態になることを確認
    let mut check_cube2 = cube2.clone();
    for &mv in &solution2.moves {
        check_cube2.apply_move(mv);
    }
    assert!(check_cube2.is_solved(), "解を適用しても完成状態にならない");

    // テストケース3: 5手のスクランブル
    let mut cube3 = Cube::new();
    cube3.apply_move(Move::R);
    cube3.apply_move(Move::U);
    cube3.apply_move(Move::F);
    cube3.apply_move(Move::R);
    cube3.apply_move(Move::U);

    let solution3 = solver::solve(&cube3, 11, true);
    assert!(solution3.found, "5手のスクランブル後に解が見つからない");

    // 解を適用して完成状態になることを確認
    let mut check_cube3 = cube3.clone();
    for &mv in &solution3.moves {
        check_cube3.apply_move(mv);
    }
    assert!(check_cube3.is_solved(), "解を適用しても完成状態にならない");
}

/// リグレッションテスト: 向きも揃える場合のEntry APIバグ検出
#[test]
fn test_solve_with_orientation_regression() {
    // テストケース1: 向きも揃える必要がある3手のスクランブル
    let mut cube1 = Cube::new();
    cube1.apply_move(Move::R);
    cube1.apply_move(Move::U);
    cube1.apply_move(Move::F);

    let solution1 = solver::solve(&cube1, 14, false);
    assert!(
        solution1.found,
        "向きも揃える: 3手のスクランブル後に解が見つからない"
    );

    // 解を適用して完全に揃うことを確認
    let mut check_cube1 = cube1.clone();
    for &mv in &solution1.moves {
        check_cube1.apply_move(mv);
    }
    assert!(
        solver::is_fully_solved(&check_cube1),
        "解を適用しても完全に揃わない"
    );

    // テストケース2: 向きも揃える必要がある4手のスクランブル
    let mut cube2 = Cube::new();
    cube2.apply_move(Move::R);
    cube2.apply_move(Move::U);
    cube2.apply_move(Move::R);
    cube2.apply_move(Move::U);

    let solution2 = solver::solve(&cube2, 14, false);
    assert!(
        solution2.found,
        "向きも揃える: 4手のスクランブル後に解が見つからない"
    );

    // 解を適用して完全に揃うことを確認
    let mut check_cube2 = cube2.clone();
    for &mv in &solution2.moves {
        check_cube2.apply_move(mv);
    }
    assert!(
        solver::is_fully_solved(&check_cube2),
        "解を適用しても完全に揃わない"
    );
}
