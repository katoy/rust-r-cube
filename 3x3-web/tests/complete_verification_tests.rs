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

/// 54個すべてのステッカーの状態を返す
fn get_all_stickers(cube: &Cube) -> Vec<(Color, u8)> {
    (0..54)
        .map(|i| {
            let sticker = cube.get_sticker(i);
            (sticker.color, sticker.orientation)
        })
        .collect()
}

/// M 操作後のすべてのステッカーの期待される状態
#[test]
fn test_m_move_all_stickers() {
    let mut cube = Cube::new();
    let mut expected = get_solved_state();

    // M 操作のサイクル: [1,4,7, 37,40,43, 10,13,16, 52,49,46] (U, F, D, B)
    // colors: U<-B, F<-U, D<-F, B<-D

    // B -> U (orientation 2)
    expected[1] = (Color::Orange, 2);
    expected[4] = (Color::Orange, 2);
    expected[7] = (Color::Orange, 2);

    // U -> F (orientation 0)
    expected[37] = (Color::White, 0);
    expected[40] = (Color::White, 0);
    expected[43] = (Color::White, 0);

    // F -> D (orientation 0)
    expected[10] = (Color::Red, 0);
    expected[13] = (Color::Red, 0);
    expected[16] = (Color::Red, 0);

    // D -> B (orientation 2)
    expected[52] = (Color::Yellow, 2);
    expected[49] = (Color::Yellow, 2);
    expected[46] = (Color::Yellow, 2);

    cube.apply_move(Move::M);
    let actual = get_all_stickers(&cube);

    for i in 0..54 {
        assert_eq!(actual[i], expected[i], "Sticker {} mismatch Move::M", i);
    }
}

/// L 操作後のすべてのステッカーの期待される状態
#[test]
fn test_l_move_all_stickers() {
    let mut cube = Cube::new();
    let mut expected = get_solved_state();

    // L 面自体の回転 (orientation 1)
    expected[18..=26].fill((Color::Green, 1));

    // サイクル: [0,3,6, 36,39,42, 9,12,15, 53,50,47] (U, F, D, B)
    // B -> U (orientation 2)
    expected[0] = (Color::Orange, 2);
    expected[3] = (Color::Orange, 2);
    expected[6] = (Color::Orange, 2);

    // U -> F (orientation 0)
    expected[36] = (Color::White, 0);
    expected[39] = (Color::White, 0);
    expected[42] = (Color::White, 0);

    // F -> D (orientation 0)
    expected[9] = (Color::Red, 0);
    expected[12] = (Color::Red, 0);
    expected[15] = (Color::Red, 0);

    // D -> B (orientation 2)
    expected[53] = (Color::Yellow, 2);
    expected[50] = (Color::Yellow, 2);
    expected[47] = (Color::Yellow, 2);

    cube.apply_move(Move::L);
    let actual = get_all_stickers(&cube);

    for i in 0..54 {
        assert_eq!(actual[i], expected[i], "Sticker {} mismatch Move::L", i);
    }
}

/// R 操作後のすべてのステッカーの期待される状態
#[test]
fn test_r_move_all_stickers() {
    let mut cube = Cube::new();
    let mut expected = get_solved_state();

    // R 面自体の回転 (orientation 1)
    expected[27..=35].fill((Color::Blue, 1));

    // サイクル: [8,5,2, 45,48,51, 17,14,11, 44,41,38] (U, B, D, F)
    // F -> U (orientation 0)
    expected[8] = (Color::Red, 0);
    expected[5] = (Color::Red, 0);
    expected[2] = (Color::Red, 0);

    // U -> B (orientation 2)
    expected[45] = (Color::White, 2);
    expected[48] = (Color::White, 2);
    expected[51] = (Color::White, 2);

    // B -> D (orientation 2)
    expected[17] = (Color::Orange, 2);
    expected[14] = (Color::Orange, 2);
    expected[11] = (Color::Orange, 2);

    // D -> F (orientation 0)
    expected[44] = (Color::Yellow, 0);
    expected[41] = (Color::Yellow, 0);
    expected[38] = (Color::Yellow, 0);

    cube.apply_move(Move::R);
    let actual = get_all_stickers(&cube);

    for i in 0..54 {
        assert_eq!(actual[i], expected[i], "Sticker {} mismatch Move::R", i);
    }
}

/// U 操作後のすべてのステッカーの期待される状態
#[test]
fn test_u_move_all_stickers() {
    let mut cube = Cube::new();
    let mut expected = get_solved_state();

    // U 面自体の回転 (orientation 1)
    expected[0..9].fill((Color::White, 1));

    // サイクル: [45,46,47, 27,28,29, 36,37,38, 18,19,20] (B, R, F, L)
    // 水平移動なので orientation は変わらない (0)
    // B -> R
    expected[27] = (Color::Orange, 0);
    expected[28] = (Color::Orange, 0);
    expected[29] = (Color::Orange, 0);

    // R -> F
    expected[36] = (Color::Blue, 0);
    expected[37] = (Color::Blue, 0);
    expected[38] = (Color::Blue, 0);

    // F -> L
    expected[18] = (Color::Red, 0);
    expected[19] = (Color::Red, 0);
    expected[20] = (Color::Red, 0);

    // L -> B
    expected[45] = (Color::Green, 0);
    expected[46] = (Color::Green, 0);
    expected[47] = (Color::Green, 0);

    cube.apply_move(Move::U);
    let actual = get_all_stickers(&cube);

    for i in 0..54 {
        assert_eq!(actual[i], expected[i], "Sticker {} mismatch Move::U", i);
    }
}

#[test]
fn test_unaffected_stickers() {
    for mv in [
        Move::U,
        Move::D,
        Move::L,
        Move::R,
        Move::F,
        Move::B,
        Move::M,
        Move::E,
        Move::S,
    ] {
        let mut cube = Cube::new();
        let original = get_all_stickers(&cube);

        cube.apply_move(mv);
        let after_move = get_all_stickers(&cube);
        let affected = get_affected_stickers(mv);

        for i in 0..54 {
            if !affected.contains(&i) {
                assert_eq!(
                    after_move[i], original[i],
                    "Move {:?}: Unaffected sticker {} changed",
                    mv, i
                );
            }
        }
    }
}

fn get_affected_stickers(mv: Move) -> Vec<usize> {
    match mv {
        Move::U => {
            let mut affected = vec![0, 1, 2, 3, 4, 5, 6, 7, 8];
            affected.extend_from_slice(&[45, 46, 47, 27, 28, 29, 36, 37, 38, 18, 19, 20]);
            affected
        }
        Move::D => {
            let mut affected = vec![9, 10, 11, 12, 13, 14, 15, 16, 17];
            affected.extend_from_slice(&[42, 43, 44, 33, 34, 35, 51, 52, 53, 24, 25, 26]);
            affected
        }
        Move::L => {
            let mut affected = vec![18, 19, 20, 21, 22, 23, 24, 25, 26];
            affected.extend_from_slice(&[0, 3, 6, 36, 39, 42, 9, 12, 15, 53, 50, 47]);
            affected
        }
        Move::R => {
            let mut affected = vec![27, 28, 29, 30, 31, 32, 33, 34, 35];
            affected.extend_from_slice(&[8, 5, 2, 45, 48, 51, 17, 14, 11, 44, 41, 38]);
            affected
        }
        Move::F => {
            let mut affected = vec![36, 37, 38, 39, 40, 41, 42, 43, 44];
            affected.extend_from_slice(&[6, 7, 8, 27, 30, 33, 11, 10, 9, 26, 23, 20]);
            affected
        }
        Move::B => {
            let mut affected = vec![45, 46, 47, 48, 49, 50, 51, 52, 53];
            affected.extend_from_slice(&[2, 1, 0, 18, 21, 24, 15, 16, 17, 35, 32, 29]);
            affected
        }
        Move::M => vec![1, 4, 7, 37, 40, 43, 10, 13, 16, 52, 49, 46],
        Move::E => vec![39, 40, 41, 30, 31, 32, 48, 49, 50, 21, 22, 23],
        Move::S => vec![3, 4, 5, 28, 31, 34, 14, 13, 12, 25, 22, 19],
        _ => vec![],
    }
}
