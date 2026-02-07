use rubiks_cube_3x3::cube::{Cube, Move};
use rubiks_cube_3x3::solver::{is_fully_solved, solve};

#[test]
fn test_solve_with_orientation_ignored() {
    // 向きを無視する場合 (ignore_orientation: true)
    // センターの色位置は変わらないが、エッジやコーナーの向きが変わる操作を適用
    let mut cube = Cube::new();

    // M操作 (スライス操作) はセンターの絶対的な位置関係は維持するが、
    // Kociembaソルバーの内部表現（色ベース）から見ると、特定の面の色配置が変わる。
    // ここでは単純な R U のような操作でテストする。
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);

    let solution = solve(&cube, 24, true);
    assert!(
        solution.found,
        "Solution should be found with orientation ignored"
    );

    // 解を適用
    for &mv in &solution.moves {
        cube.apply_move(mv);
    }

    // 色が揃っていることを確認 (is_solved は色のみをチェックするはず)
    assert!(cube.is_solved(), "Cube colors should be solved");
}

#[test]
fn test_solve_with_orientation_respected() {
    // 向きを考慮する場合 (ignore_orientation: false)
    // 注意: 現在の Kociemba 実装は、センターの色を基準に方位を特定するため、
    // 実質的に「色を揃える = 正しい方位の完成状態に導く」という動作になる。
    // ただし、M, S, E 操作によってセンターの色位置が入れ替わった場合、
    // 正しい「向き（orientation）」の定義が重要になる。

    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::Up);
    cube.apply_move(Move::F2);

    let solution = solve(&cube, 24, false);
    assert!(
        solution.found,
        "Solution should be found with orientation respected"
    );

    for &mv in &solution.moves {
        cube.apply_move(mv);
    }

    // 色が揃っていること
    assert!(cube.is_solved(), "Cube colors should be solved");
    // 向きも含めて完全に解けていること
    assert!(
        is_fully_solved(&cube),
        "Cube should be fully solved (including orientation)"
    );
}

#[test]
fn test_solve_slice_moves_ignored() {
    // スライス操作 (M, S, E) を含む状態からの解決 (向き無視)
    let mut cube = Cube::new();
    cube.apply_move(Move::M);
    cube.apply_move(Move::E);
    cube.apply_move(Move::S);

    let solution = solve(&cube, 24, true);
    assert!(
        solution.found,
        "Solution should be found for slice moves (ignored)"
    );

    for &mv in &solution.moves {
        cube.apply_move(mv);
    }

    assert!(
        cube.is_solved(),
        "Cube colors should be solved after slice moves"
    );
}

#[test]
fn test_solve_slice_moves_respected() {
    // スライス操作 (M, S, E) を含む状態からの解決 (向き考慮)
    let mut cube = Cube::new();
    cube.apply_move(Move::M);
    cube.apply_move(Move::E);
    cube.apply_move(Move::S);

    let solution = solve(&cube, 24, false);
    assert!(
        solution.found,
        "Solution should be found for slice moves (respected)"
    );

    for &mv in &solution.moves {
        cube.apply_move(mv);
    }

    assert!(cube.is_solved(), "Cube colors should be solved");

    // 方位（センターの向き等）を標準状態に正規化してチェック
    let normalized_cube = cube.with_clockwise_orientations();
    assert!(
        is_fully_solved(&normalized_cube),
        "Cube should be fully solved including orientation after slice moves (normalized)"
    );
}

#[test]
fn test_solve_superflip_respected() {
    // スーパーフリップ
    let mut cube = Cube::new();
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
    for &mv in &moves {
        cube.apply_move(mv);
    }

    let solution = solve(&cube, 64, false);
    assert!(
        solution.found,
        "Solution should be found for superflip (respected)"
    );

    for &mv in &solution.moves {
        cube.apply_move(mv);
    }

    assert!(
        cube.is_solved(),
        "Cube colors should be solved for superflip"
    );

    // スーパーフリップ解決後、方位を正規化してチェック
    let normalized_cube = cube.with_clockwise_orientations();
    assert!(
        is_fully_solved(&normalized_cube),
        "Cube should be fully solved for superflip (normalized)"
    );
}
