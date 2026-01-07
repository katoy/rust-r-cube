use rubiks_cube_2x2::cube::{Cube, Move};

/// すべての回転操作について、操作→逆操作で元に戻ることを確認
#[test]
fn test_all_moves_return_to_initial() {
    let moves_to_test = vec![
        (Move::R, Move::Rp, "R"),
        (Move::L, Move::Lp, "L"),
        (Move::U, Move::Up, "U"),
        (Move::D, Move::Dp, "D"),
        (Move::F, Move::Fp, "F"),
        (Move::B, Move::Bp, "B"),
    ];

    for (forward, backward, name) in moves_to_test {
        let initial_cube = Cube::new();
        let mut cube = Cube::new();

        // 操作 → 逆操作
        cube.apply_move(forward);
        cube.apply_move(backward);

        // 完全に元に戻ることを確認
        assert_eq!(cube, initial_cube, "{} -> {}'で元に戻らない", name, name);

        // 個々のステッカーも確認
        for i in 0..24 {
            let initial_sticker = initial_cube.get_sticker(i);
            let current_sticker = cube.get_sticker(i);
            assert_eq!(
                initial_sticker, current_sticker,
                "{} -> {}'後、ステッカー{}が異なる: {:?} != {:?}",
                name, name, i, current_sticker, initial_sticker
            );
        }
    }
}

/// R操作とRp操作で元に戻ることを確認
#[test]
fn test_r_then_rp_returns_to_initial() {
    let initial_cube = Cube::new();
    let mut cube = Cube::new();

    // R -> Rp で元に戻るはず
    cube.apply_move(Move::R);
    cube.apply_move(Move::Rp);

    // Cube全体が等しいか確認
    assert_eq!(
        cube, initial_cube,
        "R->Rp後、キューブ全体が初期状態と異なる"
    );
}
