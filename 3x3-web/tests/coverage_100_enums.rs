use rubiks_cube_3x3::cube::Move;

/// Phase 1: X2/Y2/Z2 の split_to_single() テスト
#[test]
fn test_x2_y2_z2_split_to_single() {
    // X2, Y2, Z2 の split_to_single() をテスト
    assert_eq!(Move::X2.split_to_single(), Some(Move::X));
    assert_eq!(Move::Y2.split_to_single(), Some(Move::Y));
    assert_eq!(Move::Z2.split_to_single(), Some(Move::Z));

    // 他の2回転も確認
    assert_eq!(Move::M2.split_to_single(), Some(Move::M));
    assert_eq!(Move::E2.split_to_single(), Some(Move::E));
    assert_eq!(Move::S2.split_to_single(), Some(Move::S));
}

/// 全ての2回転の split_to_single() をテスト
#[test]
fn test_all_double_moves_split_to_single() {
    let double_moves = vec![
        (Move::U2, Move::U),
        (Move::D2, Move::D),
        (Move::R2, Move::R),
        (Move::L2, Move::L),
        (Move::F2, Move::F),
        (Move::B2, Move::B),
        (Move::M2, Move::M),
        (Move::E2, Move::E),
        (Move::S2, Move::S),
        (Move::X2, Move::X),
        (Move::Y2, Move::Y),
        (Move::Z2, Move::Z),
    ];

    for (double_move, expected_half) in double_moves {
        assert_eq!(
            double_move.split_to_single(),
            Some(expected_half),
            "{:?}.split_to_single() should return Some({:?})",
            double_move,
            expected_half
        );
    }
}

/// 単一回転の split_to_single() は None を返すことを確認
#[test]
fn test_single_moves_split_to_single_none() {
    let single_moves = vec![
        Move::U,
        Move::Up,
        Move::D,
        Move::Dp,
        Move::R,
        Move::Rp,
        Move::L,
        Move::Lp,
        Move::F,
        Move::Fp,
        Move::B,
        Move::Bp,
        Move::M,
        Move::Mp,
        Move::E,
        Move::Ep,
        Move::S,
        Move::Sp,
        Move::X,
        Move::Xp,
        Move::Y,
        Move::Yp,
        Move::Z,
        Move::Zp,
    ];

    for single_move in single_moves {
        assert_eq!(
            single_move.split_to_single(),
            None,
            "{:?}.split_to_single() should return None",
            single_move
        );
    }
}
