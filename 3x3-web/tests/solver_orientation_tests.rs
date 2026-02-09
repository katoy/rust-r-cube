use rubiks_cube_3x3::cube::{Cube, Move};
use rubiks_cube_3x3::solver::{is_fully_solved, solve};

/// 指定された操作を適用し、ソルバーで解き、結果を検証するヘルパー
fn assert_solve(
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
        "解が見つかるはずです (ignore_orientation: {})",
        ignore_orientation
    );

    // 解を適用
    for &mv in &solution.moves {
        cube.apply_move(mv);
    }

    // 基本の色が揃っていることを確認
    assert!(cube.is_solved(), "色が揃っているはずです");

    if expected_full_solve {
        // 向きも含めて完全に解けていることを確認
        // 方位を標準状態に正規化してチェック（グローバル回転を許容するため）
        let normalized_cube = cube.with_clockwise_orientations();
        assert!(
            is_fully_solved(&normalized_cube),
            "向きも含めて完全に解けているはずです"
        );
    }
}

#[test]
fn test_solve_with_orientation_ignored() {
    // 向きを無視する場合 (ignore_orientation: true)
    assert_solve(&[Move::R, Move::U], 24, true, false);
}

#[test]
fn test_solve_with_orientation_respected() {
    // 向きを考慮する場合 (ignore_orientation: false)
    assert_solve(&[Move::R, Move::Up, Move::F2], 24, false, true);
}

#[test]
fn test_solve_slice_moves_ignored() {
    // スライス操作 (M, S, E) を含む状態からの解決 (向き無視)
    assert_solve(&[Move::M, Move::E, Move::S], 24, true, false);
}

#[test]
fn test_solve_slice_moves_respected() {
    // スライス操作 (M, S, E) を含む状態からの解決 (向き考慮)
    assert_solve(&[Move::M, Move::E, Move::S], 24, false, true);
}

#[test]
fn test_solve_superflip_respected() {
    // スーパーフリップ
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
    assert_solve(&moves, 64, false, true);
}
