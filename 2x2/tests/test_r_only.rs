use rubiks_cube_2x2::cube::{Cube, Move};
use rubiks_cube_2x2::solver;

#[test]
fn test_solve_single_r_move_with_orientation() {
    // リセット後にRだけ実行した状態
    let mut cube = Cube::new();
    cube.apply_move(Move::R);

    // 向きも揃えるで解法を探索 (タイムアウトしないことを確認)
    println!("向きも揃えるで解法を探索中...");
    let sol_align = solver::solve(&cube, 14, false);
    println!("探索完了: found={}", sol_align.found);

    assert!(sol_align.found, "向きも揃える: 解が見つかるべき");
    assert_eq!(sol_align.moves.len(), 1, "R操作の逆操作1手で完成のはず");
    assert_eq!(sol_align.moves[0], Move::Rp, "Rpで元に戻るはず");

    // 解を適用して完全に揃うことを確認
    let mut check_cube = cube.clone();
    for &mv in &sol_align.moves {
        check_cube.apply_move(mv);
    }
    assert!(
        solver::is_fully_solved(&check_cube),
        "解を適用すると完全に揃うべき"
    );
}
