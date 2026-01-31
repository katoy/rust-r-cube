use rubiks_cube_3x3::cube::{Color, Cube, Move};

#[test]
fn test_m_move_detailed() {
    let mut cube = Cube::new();

    // M 操作前の B面中央(46, 49, 52)
    assert_eq!(cube.get_sticker(46).color, Color::Blue);
    assert_eq!(cube.get_sticker(49).color, Color::Blue);
    assert_eq!(cube.get_sticker(52).color, Color::Blue);

    cube.apply_move(Move::M);

    // B -> U: 色は Blue, 向きは 0 (回転なし)
    // 物理移動: B-Bottom(52) -> U-Top(1), B-Mid(49) -> U-Mid(4), B-Top(46) -> U-Bot(7)
    assert_eq!(cube.get_sticker(1).color, Color::Blue);
    assert_eq!(cube.get_sticker(1).orientation, 0);
    assert_eq!(cube.get_sticker(4).color, Color::Blue);
    assert_eq!(cube.get_sticker(4).orientation, 0);
    assert_eq!(cube.get_sticker(7).color, Color::Blue);
    assert_eq!(cube.get_sticker(7).orientation, 0);
}

#[test]
fn test_l_move_detailed() {
    let mut cube = Cube::new();
    cube.apply_move(Move::L);

    // B-Bottom-Right(53) -> U-Top-Left(0)
    assert_eq!(cube.get_sticker(0).color, Color::Blue);
    assert_eq!(cube.get_sticker(0).orientation, 0);

    // L面センター(22) -> 面回転により 1
    assert_eq!(cube.get_sticker(22).color, Color::Orange);
    assert_eq!(cube.get_sticker(22).orientation, 1);
}

#[test]
fn test_r_move_detailed() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);

    // U-Top-Right(2) -> B-Bottom-Left(51)
    assert_eq!(cube.get_sticker(51).color, Color::White);
    assert_eq!(cube.get_sticker(51).orientation, 0);

    // R面センター(31) -> 面回転により 1
    assert_eq!(cube.get_sticker(31).color, Color::Red);
    assert_eq!(cube.get_sticker(31).orientation, 1);
}

#[test]
fn test_inverse_m_moves() {
    let mut cube = Cube::new();
    cube.apply_move(Move::M);
    cube.apply_move(Move::Mp);

    // 元に戻るはず
    for i in 0..54 {
        assert_eq!(
            cube.get_sticker(i).orientation,
            0,
            "Sticker {} orientation should be reset after M, Mp",
            i
        );
    }
}
