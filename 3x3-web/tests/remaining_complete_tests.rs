use rubiks_cube_3x3::cube::{Color, Cube, Move};

/// 解決済みキューブの期待される状態を返す
fn get_solved_state() -> Vec<(Color, u8)> {
    let cube = Cube::new();
    (0..54)
        .map(|i| {
            let sticker = cube.get_sticker(i);
            (sticker.color, sticker.orientation)
        })
        .collect()
}

fn get_all_stickers(cube: &Cube) -> Vec<(Color, u8)> {
    (0..54)
        .map(|i| {
            let sticker = cube.get_sticker(i);
            (sticker.color, sticker.orientation)
        })
        .collect()
}

#[test]
fn test_d_move_all_stickers() {
    let mut cube = Cube::new();
    let mut expected = get_solved_state();

    // D 面自体の回転 (orientation 1)
    for i in 9..18 {
        expected[i] = (Color::Yellow, 1);
    }

    // サイクル: [42,43,44, 33,34,35, 51,52,53, 24,25,26] (F, R, B, L)
    expected[33] = (Color::Green, 0);
    expected[34] = (Color::Green, 0);
    expected[35] = (Color::Green, 0);

    expected[51] = (Color::Red, 0);
    expected[52] = (Color::Red, 0);
    expected[53] = (Color::Red, 0);

    expected[24] = (Color::Blue, 0);
    expected[25] = (Color::Blue, 0);
    expected[26] = (Color::Blue, 0);

    expected[42] = (Color::Orange, 0);
    expected[43] = (Color::Orange, 0);
    expected[44] = (Color::Orange, 0);

    cube.apply_move(Move::D);
    let actual = get_all_stickers(&cube);

    for i in 0..54 {
        assert_eq!(actual[i], expected[i], "Sticker {} mismatch Move::D", i);
    }
}

#[test]
fn test_f_move_all_stickers() {
    let mut cube = Cube::new();
    let mut expected = get_solved_state();

    // F 面自体の回転 (orientation 1)
    for i in 36..=44 {
        expected[i] = (Color::Green, 1);
    }

    // 隣接 12 ステッカー (oris 0)
    // U(6,7,8)->R(27,30,33): Color::White, Ori: 0
    expected[27] = (Color::White, 0);
    expected[30] = (Color::White, 0);
    expected[33] = (Color::White, 0);
    // R(27,30,33)->D(11,10,9): Color::Red, Ori: 0
    expected[11] = (Color::Red, 0);
    expected[10] = (Color::Red, 0);
    expected[9] = (Color::Red, 0);
    // D(11,10,9)->L(26,23,20): Color::Yellow, Ori: 0
    expected[26] = (Color::Yellow, 0);
    expected[23] = (Color::Yellow, 0);
    expected[20] = (Color::Yellow, 0);
    // L(26,23,20)->U(6,7,8): Color::Orange, Ori: 0
    expected[6] = (Color::Orange, 0);
    expected[7] = (Color::Orange, 0);
    expected[8] = (Color::Orange, 0);

    cube.apply_move(Move::F);
    let actual = get_all_stickers(&cube);

    for i in 0..54 {
        assert_eq!(actual[i], expected[i], "Sticker {} mismatch Move::F", i);
    }
}

#[test]
fn test_b_move_all_stickers() {
    let mut cube = Cube::new();
    let mut expected = get_solved_state();

    // B 面自体の回転 (orientation 1)
    for i in 45..=53 {
        expected[i] = (Color::Blue, 1);
    }

    // 隣接 12 ステッカー (oris 0)
    expected[18] = (Color::White, 0);
    expected[21] = (Color::White, 0);
    expected[24] = (Color::White, 0);

    expected[15] = (Color::Orange, 0);
    expected[16] = (Color::Orange, 0);
    expected[17] = (Color::Orange, 0);

    expected[35] = (Color::Yellow, 0);
    expected[32] = (Color::Yellow, 0);
    expected[29] = (Color::Yellow, 0);

    expected[2] = (Color::Red, 0);
    expected[1] = (Color::Red, 0);
    expected[0] = (Color::Red, 0);

    cube.apply_move(Move::B);
    let actual = get_all_stickers(&cube);

    for i in 0..54 {
        assert_eq!(actual[i], expected[i], "Sticker {} mismatch Move::B", i);
    }
}

#[test]
fn test_e_move_all_stickers() {
    let mut cube = Cube::new();
    let mut expected = get_solved_state();

    // E 操作: F->R, R->B, B->L, L->F (oris 0)
    expected[30] = (Color::Green, 0);
    expected[31] = (Color::Green, 0);
    expected[32] = (Color::Green, 0);

    expected[48] = (Color::Red, 0);
    expected[49] = (Color::Red, 0);
    expected[50] = (Color::Red, 0);

    expected[21] = (Color::Blue, 0);
    expected[22] = (Color::Blue, 0);
    expected[23] = (Color::Blue, 0);

    expected[39] = (Color::Orange, 0);
    expected[40] = (Color::Orange, 0);
    expected[41] = (Color::Orange, 0);

    cube.apply_move(Move::E);
    let actual = get_all_stickers(&cube);

    for i in 0..54 {
        assert_eq!(actual[i], expected[i], "Sticker {} mismatch Move::E", i);
    }
}

#[test]
fn test_s_move_all_stickers() {
    let mut cube = Cube::new();
    let mut expected = get_solved_state();

    // S 操作: [3,4,5, 28,31,34, 14,13,12, 25,22,19] (U, R, D, L)
    // 隣接 12 ステッカー (oris 0)
    expected[28] = (Color::White, 0);
    expected[31] = (Color::White, 0);
    expected[34] = (Color::White, 0);

    expected[14] = (Color::Red, 0);
    expected[13] = (Color::Red, 0);
    expected[12] = (Color::Red, 0);

    expected[25] = (Color::Yellow, 0);
    expected[22] = (Color::Yellow, 0);
    expected[19] = (Color::Yellow, 0);

    expected[3] = (Color::Orange, 0);
    expected[4] = (Color::Orange, 0);
    expected[5] = (Color::Orange, 0);

    cube.apply_move(Move::S);
    let actual = get_all_stickers(&cube);

    for i in 0..54 {
        assert_eq!(actual[i], expected[i], "Sticker {} mismatch Move::S", i);
    }
}
