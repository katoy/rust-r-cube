use rubiks_cube_2x2::cube::Cube;
use rubiks_cube_2x2::solver;
use std::fs;

#[test]
fn test_cube_god_with_orientation() {
    // cube_god.txt を読み込み
    let content = fs::read_to_string("cubes/cube_god.txt")
        .expect("cube_god.txt が見つかりません");

    let cube = Cube::from_file_format(&content).expect("ファイル読み込みに失敗しました");

    // 向きを揃える解法を検索（ignore_orientation = false）
    println!("=== 向きを揃える解法を検索中 (最大深度14) ===");
    let solution = solver::solve(&cube, 11, false); // max_depth=14

    assert!(
        solution.found,
        "向きを揃える解法が見つかるはずです"
    );

    println!("解法手数: {} 手", solution.moves.len());
    println!("解法: {:?}", solution.moves);
}

#[test]
fn test_cube_god_without_orientation() {
    // cube_god.txt を読み込み
    let content = fs::read_to_string("cubes/cube_god.txt")
        .expect("cube_god.txt が見つかりません");

    let cube = Cube::from_file_format(&content).expect("ファイル読み込みに失敗しました");

    // 向きを無視する解法を検索（ignore_orientation = true）
    let solution = solver::solve(&cube, solver::DEFAULT_MAX_DEPTH, true);

    assert!(
        solution.found,
        "向きを無視する解法が見つかるはずです"
    );

    println!("解法手数: {} 手", solution.moves.len());
    println!("解法: {:?}", solution.moves);
}
