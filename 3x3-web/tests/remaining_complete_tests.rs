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
    expected[9..18].fill((Color::Yellow, 1));

    // サイクル: [42,43,44, 33,34,35, 51,52,53, 24,25,26] (F, R, B, L)
    // R receives from F
    expected[33] = (Color::Red, 0);
    expected[34] = (Color::Red, 0);
    expected[35] = (Color::Red, 0);

    // B receives from R
    expected[51] = (Color::Blue, 0);
    expected[52] = (Color::Blue, 0);
    expected[53] = (Color::Blue, 0);

    // L receives from B
    expected[24] = (Color::Orange, 0);
    expected[25] = (Color::Orange, 0);
    expected[26] = (Color::Orange, 0);

    // F receives from L
    expected[42] = (Color::Green, 0);
    expected[43] = (Color::Green, 0);
    expected[44] = (Color::Green, 0);

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
    expected[36..=44].fill((Color::Red, 1));

    // 隣接 12 ステッカー (oris 1)
    // U(6,7,8)->R(27,30,33): Color::White, Ori: 1
    expected[27] = (Color::White, 1);
    expected[30] = (Color::White, 1);
    expected[33] = (Color::White, 1);
    // R(27,30,33)->D(11,10,9): Color::Blue, Ori: 1
    expected[11] = (Color::Blue, 1);
    expected[10] = (Color::Blue, 1);
    expected[9] = (Color::Blue, 1);
    // D(11,10,9)->L(26,23,20): Color::Yellow, Ori: 1
    expected[26] = (Color::Yellow, 1);
    expected[23] = (Color::Yellow, 1);
    expected[20] = (Color::Yellow, 1);
    // L(26,23,20)->U(6,7,8): Color::Green, Ori: 1
    expected[6] = (Color::Green, 1);
    expected[7] = (Color::Green, 1);
    expected[8] = (Color::Green, 1);

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
    expected[45..=53].fill((Color::Orange, 1));

    // 隣接 12 ステッカー (oris 3)
    // L receives from U (White)
    expected[18] = (Color::White, 3);
    expected[21] = (Color::White, 3);
    expected[24] = (Color::White, 3);

    // D receives from L (Green)
    expected[15] = (Color::Green, 3);
    expected[16] = (Color::Green, 3);
    expected[17] = (Color::Green, 3);

    // R receives from D (Yellow)
    expected[35] = (Color::Yellow, 3);
    expected[32] = (Color::Yellow, 3);
    expected[29] = (Color::Yellow, 3);

    // U receives from R (Blue)
    expected[2] = (Color::Blue, 3);
    expected[1] = (Color::Blue, 3);
    expected[0] = (Color::Blue, 3);

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
    // R receives from F
    expected[30] = (Color::Red, 0);
    expected[31] = (Color::Red, 0);
    expected[32] = (Color::Red, 0);

    // B receives from R
    expected[48] = (Color::Blue, 0);
    expected[49] = (Color::Blue, 0);
    expected[50] = (Color::Blue, 0);

    // L receives from B
    expected[21] = (Color::Orange, 0);
    expected[22] = (Color::Orange, 0);
    expected[23] = (Color::Orange, 0);

    // F receives from L
    expected[39] = (Color::Green, 0);
    expected[40] = (Color::Green, 0);
    expected[41] = (Color::Green, 0);

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
    // 隣接 12 ステッカー (oris 1)
    // R receives from U
    expected[28] = (Color::White, 1);
    expected[31] = (Color::White, 1);
    expected[34] = (Color::White, 1);

    // D receives from R
    expected[14] = (Color::Blue, 1);
    expected[13] = (Color::Blue, 1);
    expected[12] = (Color::Blue, 1);

    // L receives from D
    expected[25] = (Color::Yellow, 1);
    expected[22] = (Color::Yellow, 1);
    expected[19] = (Color::Yellow, 1);

    // U receives from L
    expected[3] = (Color::Green, 1);
    expected[4] = (Color::Green, 1);
    expected[5] = (Color::Green, 1);

    cube.apply_move(Move::S);
    let actual = get_all_stickers(&cube);

    for i in 0..54 {
        assert_eq!(actual[i], expected[i], "Sticker {} mismatch Move::S", i);
    }
}
