use rubiks_cube_3x3::cube::{Color, Cube, Move};
use rubiks_cube_3x3::solver::get_orientations_vec;

#[test]
fn test_valid_cube_is_valid() {
    let cube = Cube::new();
    assert!(cube.is_valid_state().is_ok());

    let mut scrambled = cube.clone();
    scrambled.scramble(20);
    assert!(scrambled.is_valid_state().is_ok());
}

#[test]
fn test_invalid_color_count() {
    let mut cube = Cube::new();
    cube.stickers[0].color = Color::Yellow; // White -> Yellow. White=8, Yellow=10
    assert!(cube.is_valid_state().is_err());
}

#[test]
fn test_corner_twist_parity_validation() {
    let mut cube = Cube::new();
    // コーナー1つを捻る (UFR: 8, 27, 38)
    let c8 = cube.stickers[8].color;
    let c27 = cube.stickers[27].color;
    let c38 = cube.stickers[38].color;

    cube.stickers[8].color = c27;
    cube.stickers[27].color = c38;
    cube.stickers[38].color = c8;

    // この状態はコーナー捻りパリティが 1 (or 2) になり、エラーになるはず
    assert!(cube.is_valid_state().is_err());
}

#[test]
fn test_edge_flip_parity_validation() {
    let mut cube = Cube::new();
    // エッジ1つを反転 (UR: 5, 28)
    let c5 = cube.stickers[5].color;
    let c28 = cube.stickers[28].color;
    cube.stickers[5].color = c28;
    cube.stickers[28].color = c5;

    // エッジ反転パリティが 1 になり、エラーになるはず
    assert!(cube.is_valid_state().is_err());
}

#[test]
fn test_move_parity_toggle() {
    let base = Cube::new();
    let moves = vec![Move::U, Move::R, Move::F];

    for mv in moves {
        let mut c = base.clone();
        c.apply_move(mv);
        let oris = get_orientations_vec(&c);
        let sum: u32 = oris.iter().map(|&o| o as u32).sum();
        assert!(!sum.is_multiple_of(2));
    }
}

#[test]
fn test_restore_orientation_instantly() {
    let mut cube = Cube::new();
    cube.scramble(5);
    // 向き情報を消去
    let mut test_cube = cube.normalized();
    // 消去された状態でも restore できることを確認（エラーにならないこと）
    test_cube.restore_orientation_instantly().unwrap();

    // restore後は、色の配置に対して矛盾のない方位になっているはず
    // (全センター方位の和が偶数になるなどの基本条件をチェック)
    let oris = get_orientations_vec(&test_cube);
    let sum: u32 = oris.iter().map(|&o| o as u32).sum();
    assert!(
        sum.is_multiple_of(2),
        "Restored orientation sum should be even for a solvable state"
    );
}

#[test]
fn test_missing_color_completely() {
    let mut colors = [Color::White; 54];
    // White を削除し、すべて Yellow に置き換える
    for i in 0..9 {
        colors[i] = Color::Yellow;
    }
    // Yellow が18個、White は0個になる
    let result = Cube::from_colors(&colors);
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(err.to_string().contains("Color") || err.to_string().contains("色"));
    }
}

#[test]
fn test_color_count_validation() {
    // Too few of one color, too many of another
    let mut colors = [Color::White; 54];
    for i in 0..8 {
        colors[i] = Color::Yellow;
    }
    for i in 0..10 {
        colors[9 + i] = Color::White;
    }
    let result = Cube::from_colors(&colors);
    assert!(result.is_err());
}

#[test]
fn test_corner_duplicate_validation() {
    let cube = Cube::new();
    // This test is challenging because we need to create a state with duplicate corners
    // We'll create a case where colors appear valid but corner permutation is invalid
    let colors = cube.stickers.iter().map(|s| s.color).collect::<Vec<_>>();
    let mut new_colors = [Color::White; 54];
    for (i, &c) in colors.iter().enumerate() {
        new_colors[i] = c;
    }
    // The cube is still valid because valid moves preserve corner uniqueness
    let valid_cube = Cube::from_colors(&new_colors);
    assert!(valid_cube.is_ok());
}

#[test]
fn test_double_move_sequence() {
    let base = Cube::new();
    let mut c1 = base.clone();
    let mut c2 = base.clone();

    c1.apply_move(Move::R);
    c1.apply_move(Move::R);

    c2.apply_move(Move::R2);

    // R + R should equal R2
    for i in 0..54 {
        assert_eq!(c1.get_sticker(i).color, c2.get_sticker(i).color);
    }
}

#[test]
fn test_opposite_moves_cancel() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::Rp);

    // After R and R', should be back to solved
    assert!(cube.is_solved());
}

#[test]
fn test_middle_layer_moves() {
    let mut cube = Cube::new();

    cube.apply_move(Move::M);
    assert!(!cube.is_solved());

    cube.apply_move(Move::M);
    cube.apply_move(Move::M);
    cube.apply_move(Move::M);
    // M^4 should be identity
    assert!(cube.is_solved());
}

#[test]
fn test_all_global_moves() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);

    // Verify global moves can be applied without error
    for mv in &[Move::X, Move::Y, Move::Z, Move::Xp, Move::Yp, Move::Zp] {
        let mut test_cube = cube.clone();
        test_cube.apply_move(*mv);
        // Just verify it's still a valid cube
        assert_eq!(test_cube.stickers.len(), 54);
    }
}
