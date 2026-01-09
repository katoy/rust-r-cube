use rubiks_cube_2x2::cube::Cube;
use rubiks_cube_2x2::solver::solve_with_progress;

#[test]
fn test_specific_cube_file_operations() {
    // ユーザー指定のキューブ状態（物理的に無効な可能性がある）
    let input_content = r#"     WWWW
OOOO GGGR RRBG BBRB
     YYYY
"#;

    // ステップ1: ファイルから読み込む（パリティチェック無効化のため成功する）
    let cube = Cube::from_file_format(input_content).expect("ファイルフォーマットエラー");

    // ステップ2: キューブの内容を保存する
    let output_content = cube.to_file_format();

    // ステップ3: 出力ファイルと入力ファイルの内容が一致することを確認
    // 空白の違いを無視して比較
    let input_normalized: String = input_content
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    let output_normalized: String = output_content
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    assert_eq!(
        input_normalized, output_normalized,
        "保存したファイル内容が入力と一致しません"
    );

    println!("✓ ファイル読み込み・保存・検証が成功しました");
    println!("  注意: このキューブ状態は物理的に無効な可能性があります");
}

#[test]
fn test_valid_cube_complete_workflow() {
    // 有効なスクランブル状態を作成
    let mut cube = Cube::new();
    use rubiks_cube_2x2::cube::Move;

    let scramble = vec![Move::R, Move::U, Move::Fp, Move::D, Move::L];
    for move_op in &scramble {
        cube.apply_move(*move_op);
    }

    // ステップ1: キューブの内容を保存する
    let saved_content = cube.to_file_format();

    // ステップ2: ファイルから読み込む
    let cube_from_file =
        Cube::from_file_format(&saved_content).expect("ファイルフォーマットエラー");

    // ステップ3: 再保存して内容が一致することを確認
    let resaved_content = cube_from_file.to_file_format();
    let saved_normalized: String = saved_content
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    let resaved_normalized: String = resaved_content
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    assert_eq!(
        saved_normalized, resaved_normalized,
        "再保存したファイル内容が元と一致しません"
    );

    // ステップ4: 解法を見つける
    let (tx, _rx) = std::sync::mpsc::channel();
    let mut cube_clone = cube_from_file.clone();
    let solution = solve_with_progress(&cube_from_file, 14, true, Some(tx));

    assert!(solution.found, "解が見つかりませんでした");
    assert!(!solution.moves.is_empty(), "解法が空です");

    // ステップ5: 解法を最後まで実行したら6面の色が揃うことを確認
    for move_op in &solution.moves {
        cube_clone.apply_move(*move_op);
    }

    // 6面が揃っていることを確認（各面が単色）
    for face_start in [0, 4, 8, 12, 16, 20] {
        let first_color = cube_clone.get_sticker(face_start).color;
        for offset in 1..4 {
            assert_eq!(
                cube_clone.get_sticker(face_start + offset).color,
                first_color,
                "面 {} が揃っていません",
                face_start / 4
            );
        }
    }

    println!("✓ 完全なワークフローテストが成功しました");
    println!("  -  ファイル保存＆再読み込み成功");
    println!("  - 解法発見成功（{}手）", solution.moves.len());
    println!("  - 解法適用後、6面完成を確認");
}

#[test]
fn test_file_roundtrip_preserves_colors() {
    // ファイル保存→読み込みのラウンドトリップで色が保持されることを確認
    let input1 = r#"     WWWW
GGGG RRRR BBBB OOOO
     YYYY
"#;

    let cube1 = Cube::from_file_format(input1).expect("読み込みエラー");
    let saved = cube1.to_file_format();
    let cube2 = Cube::from_file_format(&saved).expect("再読み込みエラー");

    // すべてのステッカーの色が一致することを確認
    for i in 0..24 {
        assert_eq!(
            cube1.get_sticker(i).color,
            cube2.get_sticker(i).color,
            "ステッカー {} の色がラウンドトリップ後に変わりました",
            i
        );
    }
}
