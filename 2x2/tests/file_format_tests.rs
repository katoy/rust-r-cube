use rubiks_cube_2x2::cube::{Color, Cube};

#[test]
fn test_file_format_round_trip() {
    let cube = Cube::new();
    let format = cube.to_file_format();
    let restored = Cube::from_file_format(&format).unwrap();

    // ファイル形式は向きを保存しないため、色のみを比較
    for i in 0..24 {
        assert_eq!(
            cube.get_sticker(i).color,
            restored.get_sticker(i).color,
            "idx {} の色が一致しません",
            i
        );
    }
}

#[test]
fn test_file_format_scrambled() {
    let mut cube = Cube::new();
    use rubiks_cube_2x2::cube::Move;
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);
    cube.apply_move(Move::Fp);

    let format = cube.to_file_format();
    let restored = Cube::from_file_format(&format).unwrap();

    // file_formatは色のみを保存し、向きは保存しないため、
    // 色が一致していることを確認
    for i in 0..24 {
        assert_eq!(cube.get_sticker(i).color, restored.get_sticker(i).color);
    }
}

#[test]
fn test_validate_colors_valid() {
    let colors = [
        Color::White,
        Color::White,
        Color::White,
        Color::White,
        Color::Yellow,
        Color::Yellow,
        Color::Yellow,
        Color::Yellow,
        Color::Green,
        Color::Green,
        Color::Green,
        Color::Green,
        Color::Blue,
        Color::Blue,
        Color::Blue,
        Color::Blue,
        Color::Red,
        Color::Red,
        Color::Red,
        Color::Red,
        Color::Orange,
        Color::Orange,
        Color::Orange,
        Color::Orange,
    ];
    assert!(Cube::validate_colors(&colors).is_ok());
}

#[test]
fn test_validate_colors_too_many_white() {
    let colors = [
        Color::White,
        Color::White,
        Color::White,
        Color::White,
        Color::White,
        Color::Yellow,
        Color::Yellow,
        Color::Yellow,
        Color::Green,
        Color::Green,
        Color::Green,
        Color::Green,
        Color::Blue,
        Color::Blue,
        Color::Blue,
        Color::Blue,
        Color::Red,
        Color::Red,
        Color::Red,
        Color::Red,
        Color::Orange,
        Color::Orange,
        Color::Orange,
        Color::Orange,
    ];
    assert!(Cube::validate_colors(&colors).is_err());
}

#[test]
fn test_validate_colors_too_few_yellow() {
    let colors = [
        Color::White,
        Color::White,
        Color::White,
        Color::White,
        Color::Yellow,
        Color::Yellow,
        Color::Yellow,
        Color::Green,
        Color::Green,
        Color::Green,
        Color::Green,
        Color::Green,
        Color::Blue,
        Color::Blue,
        Color::Blue,
        Color::Blue,
        Color::Red,
        Color::Red,
        Color::Red,
        Color::Red,
        Color::Orange,
        Color::Orange,
        Color::Orange,
        Color::Orange,
    ];
    assert!(Cube::validate_colors(&colors).is_err());
}

#[test]
fn test_from_file_format_invalid_format() {
    let invalid = "INVALID";
    assert!(Cube::from_file_format(invalid).is_err());
}

#[test]
fn test_from_file_format_wrong_color() {
    let invalid = r#"     XXXX
GGGG RRRR BBBB OOOO
     YYYY
"#;
    assert!(Cube::from_file_format(invalid).is_err());
}

#[test]
fn test_from_file_format_too_many_lines() {
    let invalid = r#"     WWWW
GGGG RRRR BBBB OOOO
     YYYY
EXTRA LINE
"#;
    assert!(Cube::from_file_format(invalid).is_err());
}

#[test]
fn test_to_file_format_structure() {
    let cube = Cube::new();
    let format = cube.to_file_format();
    let lines: Vec<&str> = format.lines().collect();
    assert_eq!(lines.len(), 3);

    // First line should have 4 leading spaces and 4 colors
    assert!(lines[0].starts_with("     "));
    assert_eq!(lines[0].trim().len(), 4);

    // Second line should have 4 groups of 4 colors
    let second_line_colors: String = lines[1].chars().filter(|c| !c.is_whitespace()).collect();
    assert_eq!(second_line_colors.len(), 16);

    // Third line should have 4 leading spaces and 4 colors
    assert!(lines[2].starts_with("     "));
    assert_eq!(lines[2].trim().len(), 4);
}
