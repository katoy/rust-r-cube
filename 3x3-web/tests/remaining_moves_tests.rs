use rubiks_cube_3x3::cube::{Color, Cube, Move};

/// F 操作の詳細検証
#[test]
fn test_f_move_detailed_verification() {
    let mut cube = Cube::new();
    cube.apply_move(Move::F);

    // F 操作のサイクル: [6,7,8, 27,30,33, 11,10,9, 26,23,20] (U, R, D, L)

    // U面下端(6,7,8) -> R面左端(27,30,33)
    assert_eq!(cube.get_sticker(27).color, Color::White);
    assert_eq!(cube.get_sticker(30).color, Color::White);
    assert_eq!(cube.get_sticker(33).color, Color::White);

    // Orientation チェック: F 操作（U->R）は 90度回転 (1)
    assert_eq!(
        cube.get_sticker(27).orientation,
        1,
        "U->R should rotate by 90 deg"
    );
    assert_eq!(
        cube.get_sticker(30).orientation,
        1,
        "U->R should rotate by 90 deg"
    );
    assert_eq!(
        cube.get_sticker(33).orientation,
        1,
        "U->R should rotate by 90 deg"
    );
}

/// B 操作の詳細検証
#[test]
fn test_b_move_detailed_verification() {
    let mut cube = Cube::new();
    cube.apply_move(Move::B);

    // B 操作のサイクル: [2,1,0, 18,21,24, 15,16,17, 35,32,29] (U, L, D, R)

    // U面上端(2,1,0) -> L面左端(18,21,24)
    assert_eq!(cube.get_sticker(18).color, Color::White);
    assert_eq!(cube.get_sticker(21).color, Color::White);
    assert_eq!(cube.get_sticker(24).color, Color::White);

    // Orientation チェック: B 操作（U->L）は 270度回転 (3)
    assert_eq!(
        cube.get_sticker(18).orientation,
        3,
        "U->L should rotate by 270 deg"
    );
    assert_eq!(
        cube.get_sticker(21).orientation,
        3,
        "U->L should rotate by 270 deg"
    );
    assert_eq!(
        cube.get_sticker(24).orientation,
        3,
        "U->L should rotate by 270 deg"
    );
}

/// S 操作の詳細検証
#[test]
fn test_s_move_detailed_verification() {
    let mut cube = Cube::new();
    cube.apply_move(Move::S);

    // S 操作のサイクル: [3,4,5, 28,31,34, 14,13,12, 25,22,19] (U, R, D, L)

    // U面中段(3,4,5) -> R面中段(28,31,34)
    assert_eq!(cube.get_sticker(28).color, Color::White);
    assert_eq!(cube.get_sticker(31).color, Color::White);
    assert_eq!(cube.get_sticker(34).color, Color::White);

    // Orientation チェック: S 操作（U->R）は 90度回転 (1)
    assert_eq!(
        cube.get_sticker(28).orientation,
        1,
        "U->R should rotate by 90 deg in S move"
    );
    assert_eq!(
        cube.get_sticker(31).orientation,
        1,
        "U->R should rotate by 90 deg in S move"
    );
    assert_eq!(
        cube.get_sticker(34).orientation,
        1,
        "U->R should rotate by 90 deg in S move"
    );
}

/// U 操作の詳細検証
#[test]
fn test_u_move_detailed_verification() {
    let mut cube = Cube::new();
    cube.apply_move(Move::U);

    // U 操作のサイクル: [45,46,47, 27,28,29, 36,37,38, 18,19,20] (B, R, F, L)

    // B面上端(45,46,47) -> R面上端(27,28,29)
    assert_eq!(cube.get_sticker(27).color, Color::Orange);
    assert_eq!(cube.get_sticker(28).color, Color::Orange);
    assert_eq!(cube.get_sticker(29).color, Color::Orange);

    // Orientation チェック: 水平移動なので回転なし
    assert_eq!(cube.get_sticker(27).orientation, 0, "B->R horizontal move");
    assert_eq!(cube.get_sticker(28).orientation, 0, "B->R horizontal move");
    assert_eq!(cube.get_sticker(29).orientation, 0, "B->R horizontal move");
}

/// D 操作の詳細検証
#[test]
fn test_d_move_detailed_verification() {
    let mut cube = Cube::new();
    cube.apply_move(Move::D);

    // D 操作のサイクル: [42,43,44, 33,34,35, 51,52,53, 24,25,26] (F, R, B, L)

    // F面下端(42,43,44) -> R面下端(33,34,35)
    assert_eq!(cube.get_sticker(33).color, Color::Red);
    assert_eq!(cube.get_sticker(34).color, Color::Red);
    assert_eq!(cube.get_sticker(35).color, Color::Red);

    // Orientation チェック: 水平移動なので回転なし
    assert_eq!(cube.get_sticker(33).orientation, 0, "F->R horizontal move");
    assert_eq!(cube.get_sticker(34).orientation, 0, "F->R horizontal move");
    assert_eq!(cube.get_sticker(35).orientation, 0, "F->R horizontal move");
}

/// E 操作の詳細検証
#[test]
fn test_e_move_detailed_verification() {
    let mut cube = Cube::new();
    cube.apply_move(Move::E);

    // E 操作のサイクル: [39,40,41, 30,31,32, 48,49,50, 21,22,23] (F, R, B, L)

    // F面中段(39,40,41) -> R面中段(30,31,32)
    assert_eq!(cube.get_sticker(30).color, Color::Red);
    assert_eq!(cube.get_sticker(31).color, Color::Red);
    assert_eq!(cube.get_sticker(32).color, Color::Red);

    // Orientation チェック: 水平移動なので回転なし
    assert_eq!(cube.get_sticker(30).orientation, 0, "F->R horizontal move");
    assert_eq!(cube.get_sticker(31).orientation, 0, "F->R horizontal move");
    assert_eq!(cube.get_sticker(32).orientation, 0, "F->R horizontal move");
}
