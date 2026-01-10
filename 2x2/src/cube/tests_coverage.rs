use super::*;

#[test]
fn test_io_gray_color() {
    let mut cube = Cube::new();
    // 内部フィールドに直接アクセスしてGrayを設定
    cube.stickers[0] = Sticker::new(Color::Gray);

    let s = cube.to_file_format();
    assert!(s.contains(' ')); // Gray is mapped to space
}

#[test]
fn test_set_sticker_color() {
    let mut cube = Cube::new();
    cube.set_sticker_color(0, Color::Red);
    assert_eq!(cube.stickers[0].color, Color::Red);
    assert_eq!(cube.stickers[0].orientation, 0); // Side effect check
}

#[test]
fn test_is_valid_state_ok() {
    let cube = Cube::new();
    assert!(cube.is_valid_state().is_ok());
}

#[test]
fn test_is_valid_state_invalid_color_count() {
    let mut cube = Cube::new();
    cube.set_sticker_color(0, Color::Red); // Whiteが1つ減り、Redが1つ増える
    assert!(cube.is_valid_state().is_err());
}
