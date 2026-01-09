use rubiks_cube_2x2::cube::Cube;
use rubiks_cube_2x2::solver::solve_with_progress;

#[test]
fn test_real_cube_solve_and_verify() {
    // 実物のキューブ状態をシミュレート（有効なスクランブル）
    let mut cube = Cube::new();
    use rubiks_cube_2x2::cube::Move;

    // 複雑なスクランブルを適用
    let scramble = vec![Move::R, Move::U, Move::Fp, Move::D, Move::L, Move::Bp];
    for move_op in &scramble {
        cube.apply_move(*move_op);
    }

    // ファイルフォーマットに保存して読み込み（実物のキューブ入力をシミュレート）
    let format = cube.to_file_format();
    let mut cube_from_file = Cube::from_file_format(&format).expect("ファイルフォーマットエラー");

    // 解法を見つける（向き無視、最大深さ14）
    let (tx, _rx) = std::sync::mpsc::channel();
    let solution = solve_with_progress(&cube_from_file, 14, true, Some(tx));

    assert!(solution.found, "解が見つかりませんでした");
    assert!(!solution.moves.is_empty(), "解法が空です");

    // 解法を最後まで実行
    for move_op in &solution.moves {
        cube_from_file.apply_move(*move_op);
    }

    // 6面が揃っていることを確認（色のみ）
    // 各面が単色になっているか確認
    for face_start in [0, 4, 8, 12, 16, 20] {
        let first_color = cube_from_file.get_sticker(face_start).color;
        for offset in 1..4 {
            assert_eq!(
                cube_from_file.get_sticker(face_start + offset).color,
                first_color,
                "面 {} が揃っていません",
                face_start / 4
            );
        }
    }
}

#[test]
fn test_file_format_solve_roundtrip() {
    // スクランブル状態を作成
    let mut original = Cube::new();
    use rubiks_cube_2x2::cube::Move;
    original.apply_move(Move::R);
    original.apply_move(Move::U);
    original.apply_move(Move::R);
    original.apply_move(Move::F);

    // ファイルフォーマットに保存して読み込み
    let format = original.to_file_format();
    let mut cube = Cube::from_file_format(&format).expect("ファイルフォーマットエラー");

    // 解法を見つける
    let (tx, _rx) = std::sync::mpsc::channel();
    let solution = solve_with_progress(&cube, 11, true, Some(tx));

    assert!(solution.found, "解が見つかりませんでした");

    // 解法を適用
    for move_op in &solution.moves {
        cube.apply_move(*move_op);
    }

    // 完成状態になっていることを確認
    let solved = Cube::new();
    for i in 0..24 {
        assert_eq!(
            cube.get_sticker(i).color,
            solved.get_sticker(i).color,
            "ステッカー {} の色が一致しません",
            i
        );
    }
}
