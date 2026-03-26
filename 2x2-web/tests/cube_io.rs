use rubiks_cube_2x2::cube::{Color, Cube, Move};

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
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);
    cube.apply_move(Move::Fp);

    let format = cube.to_file_format();
    let restored = Cube::from_file_format(&format).unwrap();

    for i in 0..24 {
        assert_eq!(cube.get_sticker(i).color, restored.get_sticker(i).color);
    }
}

#[test]
fn test_to_file_format_structure() {
    let cube = Cube::new();
    let format = cube.to_file_format();
    let lines: Vec<&str> = format.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines[0].starts_with("     "));
    assert_eq!(lines[0].trim().len(), 4);
}

#[test]
fn test_save_load_consistency() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);
    cube.apply_move(Move::F);

    let saved_str = cube.to_file_format();
    let loaded_cube = Cube::from_file_format(&saved_str).expect("Failed to load cube");

    for i in 0..24 {
        assert_eq!(cube.get_sticker(i).color, loaded_cube.get_sticker(i).color);
    }
}

#[test]
fn test_save_load_orientation_restoration() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::B2);
    cube.apply_move(Move::U);

    let saved_str = cube.to_file_format();
    let loaded_cube = Cube::from_file_format(&saved_str).expect("Failed to load");

    assert!(loaded_cube.is_valid_state().is_ok());
    let solution = rubiks_cube_2x2::solver::solve(&loaded_cube, 11, false);
    assert!(solution.found);
}

#[test]
fn test_parse_gray_state() {
    let input = "     ....\n.... .... .... ....\n     ....\n";
    let cube = Cube::from_file_format(input).expect("Failed to parse gray state");
    assert_eq!(cube.stickers[0].color, Color::Gray);
}

#[test]
fn test_legacy_format_compatibility() {
    let legacy = "     WWWW\nGGGG RRRR BBBB OOOO\n     YYYY\n";
    let cube = Cube::from_file_format(legacy).expect("Failed to load legacy format");
    // [1,2,0,3] パターンの index 0 は 1
    assert_eq!(cube.stickers[0].orientation, 1);
}

#[test]
fn test_io_from_file_format_errors() {
    // 行数不足
    assert!(Cube::from_file_format("WWWW\nGGGG").is_err());

    // 1行目のパーツ数不正
    assert!(Cube::from_file_format("WWW\nGGGG RRRR BBBB OOOO\nYYYY").is_err());

    // 2行目のパーツ数不正
    assert!(Cube::from_file_format("WWWW\nGGGG RRRR BBBB OOO\nYYYY").is_err());

    // 3行目のパーツ数不正
    assert!(Cube::from_file_format("WWWW\nGGGG RRRR BBBB OOOO\nYYY").is_err());

    // 2行目に無効な文字
    assert!(Cube::from_file_format("WWWW\nGGGG RRRR BBBB OOOZ\nYYYY").is_err());

    // 1行目に無効な文字（parse_colors(lines[0])? のエラーパスをカバー）
    assert!(Cube::from_file_format("WWWZ\nGGGG RRRR BBBB OOOO\nYYYY").is_err());

    // 3行目に無効な文字（parse_colors(lines[2])? のエラーパスをカバー）
    assert!(Cube::from_file_format("WWWW\nGGGG RRRR BBBB OOOO\nYYYZ").is_err());
}

#[test]
fn test_to_file_format_with_gray() {
    let mut cube = Cube::new();
    cube.stickers[0].color = Color::Gray;
    let s = cube.to_file_format();
    assert!(s.contains('.'));
}
