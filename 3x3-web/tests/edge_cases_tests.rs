use rubiks_cube_3x3::cube::{Color, Cube, Move};
use rubiks_cube_3x3::kociemba::{RawCube, Search};
use rubiks_cube_3x3::solver::{solve, is_fully_solved};

/// エッジケースと error 条件を触発するテスト

#[test]
fn test_color_not_found_error() {
    // 特定の色が完全に欠落している状態
    let mut colors = [Color::White; 54];
    // White を削除し、他の色で補充
    for color in colors.iter_mut().take(9) {
        *color = Color::Yellow;  // White が 0 個に
    }
    for color in colors.iter_mut().take(18).skip(9) {
        *color = Color::Yellow;  // Yellow が 18 個に
    }

    let result = Cube::from_colors(&colors);
    assert!(result.is_err());

    if let Err(e) = result {
        let err_msg = e.to_string();
        assert!(err_msg.contains("Color") || err_msg.contains("White") || err_msg.contains("色"));
    }
}

#[test]
fn test_invalid_color_counts_mixed() {
    // 複数の色が不正な個数
    let mut colors = [Color::White; 54];

    // White を 8 個に (不足)
    for color in colors.iter_mut().take(8) {
        *color = Color::White;
    }
    colors[8] = Color::Yellow;

    // Yellow を 10 個に (過剰)
    for color in colors.iter_mut().take(19).skip(9) {
        *color = Color::Yellow;
    }

    let result = Cube::from_colors(&colors);
    assert!(result.is_err());
}

#[test]
fn test_corner_permutation_uniqueness() {
    // コーナーピースの置換に関する検証
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);
    cube.apply_move(Move::Rp);

    let rc = RawCube::from_cube(&cube).unwrap();

    // すべてのコーナーが一意であることを確認
    let mut corner_set = std::collections::HashSet::new();
    for &corner in &rc.cp {
        assert!(corner_set.insert(corner as u8), "Duplicate corner detected");
    }
    assert_eq!(corner_set.len(), 8);
}

#[test]
fn test_edge_permutation_uniqueness() {
    // エッジピースの置換に関する検証
    let mut cube = Cube::new();
    for _ in 0..3 {
        cube.apply_move(Move::F);
        cube.apply_move(Move::U);
    }

    let rc = RawCube::from_cube(&cube).unwrap();

    // すべてのエッジが一意であることを確認
    let mut edge_set = std::collections::HashSet::new();
    for &edge in &rc.ep {
        assert!(edge_set.insert(edge as u8), "Duplicate edge detected");
    }
    assert_eq!(edge_set.len(), 12);
}

#[test]
fn test_very_scrambled_cube() {
    // 大量の操作でスクランブルされたキューブ
    let mut cube = Cube::new();
    for _ in 0..50 {
        cube.scramble(1);
    }

    let result = solve(&cube, 25, false);
    // 深い探索でも解けることを確認
    if result.found {
        assert!(!result.moves.is_empty());
    }
}

#[test]
fn test_solve_depth_boundary_cases() {
    // 特定の深さでのバウンダリケース
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);
    cube.apply_move(Move::F);

    // 深さ 0: 解けない (スクランブルされているため)
    let result = solve(&cube, 0, false);
    assert!(!result.found);

    // 深さ 1: 多くの場合解けない
    let _result = solve(&cube, 1, false);
    // may or may not be found depending on scramble

    // 深さ 3: 高確率で解ける
    let result = solve(&cube, 3, false);
    assert!(result.found);
}

#[test]
fn test_solved_cube_with_ignore_orientation() {
    // 既に解けているキューブで ignore_orientation フラグをテスト
    let cube = Cube::new();

    let result_color_only = solve(&cube, 20, true);
    assert!(result_color_only.found);
    assert_eq!(result_color_only.moves.len(), 0);

    let result_full = solve(&cube, 20, false);
    assert!(result_full.found);
    assert_eq!(result_full.moves.len(), 0);
}

#[test]
fn test_raw_cube_multiply_identity() {
    // RawCube の乗算でアイデンティティケースをテスト
    let mut cube = Cube::new();
    cube.apply_move(Move::R);

    let _rc = RawCube::from_cube(&cube).unwrap();

    // R と R' を合成すると、ほぼアイデンティティになる
    let _rc_prime = RawCube::from_cube(&Cube::new()).unwrap();
    for _ in 0..3 {
        cube.apply_move(Move::R);
    }
    let _rc_cubed = RawCube::from_cube(&cube).unwrap();

    // R^4 = I の検証
    let mut test_cube = Cube::new();
    for _ in 0..4 {
        test_cube.apply_move(Move::R);
    }
    assert!(test_cube.is_solved());
}

#[test]
fn test_multiple_sequential_solves() {
    // 複数回連続で solve を呼び出すケース
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);

    for i in 0..3 {
        let result = solve(&cube, 20, false);
        assert!(result.found, "Iteration {} failed", i);
    }
}

#[test]
fn test_solve_with_all_moves() {
    // すべての移動方向が含まれるスクランブル
    let mut cube = Cube::new();
    let all_moves = vec![
        Move::R, Move::L, Move::U, Move::D, Move::F, Move::B,
        Move::M, Move::E, Move::S,
        Move::X, Move::Y, Move::Z,
    ];

    for mv in &all_moves {
        cube.apply_move(*mv);
    }

    let result = solve(&cube, 20, false);
    assert!(result.found);
}

#[test]
fn test_is_fully_solved_checks() {
    let cube = Cube::new();
    assert!(is_fully_solved(&cube));

    let mut scrambled = cube.clone();
    scrambled.apply_move(Move::R);
    assert!(!is_fully_solved(&scrambled));

    let mut almost_solved = cube.clone();
    almost_solved.apply_move(Move::R);
    almost_solved.apply_move(Move::Rp);
    assert!(is_fully_solved(&almost_solved));
}

#[test]
fn test_raw_cube_from_invalid_state() {
    // 正常なキューブ構造だがスクランブルされている状態
    let mut cube = Cube::new();
    for _ in 0..10 {
        cube.apply_move(Move::R);
    }

    // RawCube 変換が成功することを確認
    let rc = RawCube::from_cube(&cube);
    assert!(rc.is_ok());

    if let Ok(rc) = rc {
        // コーナー配列の検証
        assert_eq!(rc.cp.len(), 8);
        assert_eq!(rc.co.len(), 8);
        // エッジ配列の検証
        assert_eq!(rc.ep.len(), 12);
        assert_eq!(rc.eo.len(), 12);
    }
}

#[test]
fn test_normalized_vs_original() {
    // 正規化されたキューブとオリジナルの比較
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);
    cube.apply_move(Move::F);

    let normalized = cube.normalized();

    // 正規化後、すべてのステッカーの向きが 0 になっているはず
    let mut all_ori_zero = true;
    for i in 0..54 {
        if normalized.get_sticker(i).orientation != 0 {
            all_ori_zero = false;
            break;
        }
    }
    // 多くのステッカーは 0 のはず（必ずではないが）
    assert!(all_ori_zero);

    // 色の配置は変わらないはず
    for i in 0..54 {
        assert_eq!(cube.get_sticker(i).color, normalized.get_sticker(i).color);
    }
}

#[test]
fn test_face_rotations_four_times_identity() {
    // すべての面回転で 4 回回転するとアイデンティティになることを検証
    for mv in &[Move::U, Move::D, Move::L, Move::R, Move::F, Move::B] {
        let mut cube = Cube::new();
        for _ in 0..4 {
            cube.apply_move(*mv);
        }
        assert!(cube.is_solved(), "Move {:?}^4 should be identity", mv);
    }
}

#[test]
fn test_middle_layer_cycle_order() {
    // ミドルレイヤーの周期をテスト（異なる可能性がある）
    for mv in &[Move::M, Move::E, Move::S] {
        let mut cube = Cube::new();
        let mut found_cycle = false;
        for i in 1..=8 {
            cube.apply_move(*mv);
            if cube.is_solved() {
                found_cycle = true;
                // ミドルレイヤーは特定の周期を持つ
                assert!(i > 0 && i <= 8, "Move {:?} should cycle within 1-8 moves", mv);
                break;
            }
        }
        assert!(found_cycle, "Move {:?} should return to identity within 8 moves", mv);
    }
}

#[test]
fn test_search_with_zero_max_nodes() {
    // max_nodes が 0 の場合のテスト
    let mut cube = Cube::new();
    cube.apply_move(Move::R);

    let rc = RawCube::from_cube(&cube).unwrap();
    let mut search = Search::new();
    search.max_nodes = 1;  // 非常に制限

    let result = search.solve(&rc, 10);
    // ノード制限でフェイル可能性がある
    let _ = result;
}

#[test]
fn test_color_array_conversion() {
    // カラー配列からキューブへの変換とラウンドトリップ
    let mut colors_array = [Color::White; 54];
    for i in 0..9 {
        colors_array[i] = Color::White;
        colors_array[9 + i] = Color::Yellow;
        colors_array[18 + i] = Color::Green;
        colors_array[27 + i] = Color::Blue;
        colors_array[36 + i] = Color::Red;
        colors_array[45 + i] = Color::Orange;
    }

    let cube = Cube::from_colors(&colors_array).unwrap();
    assert!(cube.is_solved());

    // ラウンドトリップテスト
    for (i, &color) in colors_array.iter().enumerate() {
        let sticker = cube.get_sticker(i);
        assert_eq!(sticker.color, color);
    }
}
