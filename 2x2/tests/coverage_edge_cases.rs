use rubiks_cube_2x2::cube::{Color, Cube};

// from_file_formatのエラーケース追加テスト

#[test]
fn test_from_file_format_too_few_lines() {
    let invalid = r#"     WWWW
GGGG RRRR BBBB OOOO
"#;
    assert!(Cube::from_file_format(invalid).is_err());
}

#[test]
fn test_from_file_format_line1_too_short() {
    let invalid = r#"     WWW
GGGG RRRR BBBB OOOO
     YYYY
"#;
    assert!(Cube::from_file_format(invalid).is_err());
}

#[test]
fn test_from_file_format_line2_too_short() {
    let invalid = r#"     WWWW
GGGG RRRR BBBB OOO
     YYYY
"#;
    assert!(Cube::from_file_format(invalid).is_err());
}

#[test]
fn test_from_file_format_line3_too_short() {
    let invalid = r#"     WWWW
GGGG RRRR BBBB OOOO
     YYY
"#;
    assert!(Cube::from_file_format(invalid).is_err());
}

#[test]
fn test_validate_colors_missing_color() {
    // すべての色が欠けているケース（Grayが多すぎる）
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
        Color::Gray, // Greenが1つ少ない
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
    let result = Cube::validate_colors(&colors);
    assert!(result.is_err());
}

#[test]
fn test_validate_colors_all_wrong() {
    // すべての色が間違っているケース
    let colors = [Color::Gray; 24];
    let result = Cube::validate_colors(&colors);
    assert!(result.is_err());
    // エラーメッセージに"が見つかりません"が含まれることを確認
    if let Err(msg) = result {
        let msg_str = msg.to_string();
        assert!(msg_str.contains("が見つかりません") || msg_str.contains("個必要ですが"));
    }
}

#[test]
fn test_cube_default_equals_new() {
    let cube1 = Cube::default();
    let cube2 = Cube::new();
    assert_eq!(cube1, cube2);
}

#[test]
fn test_color_gray_not_in_solved_cube() {
    // Gray色は完成状態のキューブには使用されない
    let cube = Cube::new();
    for i in 0..24 {
        assert_ne!(cube.get_sticker(i).color, Color::Gray);
    }
}

#[test]
fn test_from_file_format_empty_string() {
    assert!(Cube::from_file_format("").is_err());
}

#[test]
fn test_from_file_format_one_line() {
    assert!(Cube::from_file_format("WWWW").is_err());
}

#[test]
fn test_from_file_format_two_lines() {
    let invalid = r#"     WWWW
GGGG RRRR BBBB OOOO"#;
    assert!(Cube::from_file_format(invalid).is_err());
}

#[test]
fn test_from_file_format_whitespace_only() {
    let invalid = r#"     
      
     
"#;
    assert!(Cube::from_file_format(invalid).is_err());
}
