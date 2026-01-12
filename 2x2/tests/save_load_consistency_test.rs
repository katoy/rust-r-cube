use rubiks_cube_2x2::cube::{Cube, Move, Face, Color};

#[test]
fn test_save_load_consistency() {
    // 1. スクランブルしたキューブを用意
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);
    cube.apply_move(Move::F);

    // 2. 文字列化
    let saved_str = cube.to_file_format();
    println!("Saved Format:\n{}", saved_str);

    // 3. 復元
    let loaded_cube = Cube::from_file_format(&saved_str).expect("Failed to load cube");

    // 4. 重複・欠落なく色が一致することを確認
    for i in 0..24 {
        assert_eq!(
            cube.get_sticker(i).color,
            loaded_cube.get_sticker(i).color,
            "Color mismatch at index {}", i
        );
    }
}

#[test]
fn test_save_load_orientation_restoration() {
    // スクランブルされた状態から保存・復元した際、
    // 向きが物理的に正しい（解決可能である）ことを確認
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::B2);
    cube.apply_move(Move::U);

    let saved_str = cube.to_file_format();
    let loaded_cube = Cube::from_file_format(&saved_str).expect("Failed to load");

    // 読み込まれたキューブが有効な状態（パリティエラーなし）であることを確認
    assert!(loaded_cube.is_valid_state().is_ok());

    // ソルバーで解けることを確認（向きも考慮）
    let solution = rubiks_cube_2x2::solver::solve(&loaded_cube, 11, false);
    assert!(solution.found, "Loaded cube should be solvable with orientation");
}

#[test]
fn test_save_load_orientation_persistence() {
    // 向きは保存されないことを確認（旧形式への差し戻し）
    let mut cube = Cube::new();
    for i in 0..24 {
        cube.stickers[i].orientation = 2;
    }
    
    let saved_str = cube.to_file_format();
    assert!(!saved_str.contains("W2"), "Should not contain orientation info");
    assert!(saved_str.contains("W"), "Should contain color info");
    
    let loaded_cube = Cube::from_file_format(&saved_str).expect("Failed to load");
    // 読み込み直後は向きがリセット（または from_colors 内の初期化）される
    // Cube::from_colors はデフォルトで with_clockwise_orientations() を呼ぶため [1,2,0,3] になる
    assert_ne!(loaded_cube.stickers[0].orientation, 2);
}

#[test]
fn test_parse_gray_state() {
    // Gray (未設定) 面を含む状態のパース (1文字形式)
    let input = "     ....\n.... .... .... ....\n     ....\n";
    let cube = Cube::from_file_format(input).expect("Failed to parse gray state");
    assert_eq!(cube.stickers[0].color, Color::Gray);
}

#[test]
fn test_legacy_format_compatibility() {
    // 旧形式（色のみ）の読み込み
    let legacy = "     WWWW\nGGGG RRRR BBBB OOOO\n     YYYY\n";
    let cube = Cube::from_file_format(legacy).expect("Failed to load legacy format");
    // 旧形式は clockwise テンプレート（index 0 は 1）として読み込まれる
    assert_eq!(cube.stickers[0].orientation, 1);
}
