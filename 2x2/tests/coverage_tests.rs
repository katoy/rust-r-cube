use rubiks_cube_2x2::cube::{Color, Cube, Move};
use rubiks_cube_2x2::solver::{self, Solution};
use std::sync::mpsc;

// ============================================================
// cube/mod.rs カバレッジ
// ============================================================

#[test]
fn test_from_colors_invalid_color_count() {
    // validate_colors がエラーになる色配列で from_colors を呼ぶ
    let mut colors = [Color::White; 24];
    colors[0] = Color::Red; // White=3, Red=5 → 検証エラー
    assert!(Cube::from_colors(&colors).is_err());
}

// ============================================================
// cube/io.rs カバレッジ
// ============================================================

#[test]
fn test_from_file_format_invalid_char() {
    // 不正な文字（X）を含む入力
    let invalid = "     WWWX\nGGGG RRRR BBBB OOOO\n     YYYY\n";
    assert!(Cube::from_file_format(invalid).is_err());
}

// ============================================================
// cube/validation.rs カバレッジ: check_corner_parity のエラーパス
// ============================================================

#[test]
fn test_check_corner_same_color_in_corner() {
    // UFL[2,9,16] を Red, Green, Red にする（同じ色が2つ）
    // sticker[2]=Red (was White), sticker[17]=White (was Red) で色数を保持
    let mut cube = Cube::new();
    cube.stickers[2].color = Color::Red;
    cube.stickers[17].color = Color::White;
    assert!(cube.is_valid_state().is_err());
}

#[test]
fn test_check_corner_opposite_colors_in_corner() {
    // UFL[2,9,16] を White, Yellow, Red にする（対面色 White-Yellow）
    // sticker[9]=Yellow (was Green), sticker[4]=Green (was Yellow) で色数を保持
    let mut cube = Cube::new();
    cube.stickers[9].color = Color::Yellow;
    cube.stickers[4].color = Color::Green;
    assert!(cube.is_valid_state().is_err());
}

#[test]
fn test_check_corner_duplicate_pieces() {
    // UFL[2,9,16]={White,Blue,Red} と UFR[3,17,12]={White,Red,Blue} を同一にする
    // sticker[9]=Blue (was Green), sticker[13]=Green (was Blue) で色数を保持
    let mut cube = Cube::new();
    cube.stickers[9].color = Color::Blue;
    cube.stickers[13].color = Color::Green;
    assert!(cube.is_valid_state().is_err());
}

#[test]
fn test_check_corner_no_white_yellow() {
    // UFL[2,9,16] を Orange, Green, Red にする（White/Yellow なし）
    // sticker[2]=Orange (was White), sticker[23]=White (was Orange) で色数を保持
    let mut cube = Cube::new();
    cube.stickers[2].color = Color::Orange;
    cube.stickers[23].color = Color::White;
    assert!(cube.is_valid_state().is_err());
}

// ============================================================
// cube/mod.rs カバレッジ: is_solved_with_orientation の !is_solved() パス
// ============================================================

#[test]
fn test_is_solved_with_orientation_when_not_solved() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R); // 色が揃わない状態
                              // is_solved() = false → is_solved_with_orientation() も即 false を返す
    assert!(!cube.is_solved_with_orientation());
}

// ============================================================
// cube/mod.rs カバレッジ: restore_orientation_instantly の成功パスとエラーパス
// ============================================================

#[test]
fn test_from_colors_valid_solved_cube() {
    // 解決済みキューブの色配列から from_colors → restore_orientation_instantly 成功パス
    let cube = Cube::new();
    let colors: [Color; 24] = std::array::from_fn(|i| cube.get_sticker(i).color);
    let result = Cube::from_colors(&colors);
    assert!(result.is_ok());
}

#[test]
fn test_from_colors_twist_parity_error() {
    // UFL コーナーを1ねじった色配列で from_colors → restore_orientation_instantly が
    // is_valid_state でエラー → mod.rs:160 の ? エラーパスを通る
    let cube = Cube::new();
    let mut colors: [Color; 24] = std::array::from_fn(|i| cube.get_sticker(i).color);
    // [White,Green,Red] → [Red,White,Green] (時計回りに1ねじり)
    colors[2] = Color::Red; // was White
    colors[9] = Color::White; // was Green
    colors[16] = Color::Green; // was Red
    let result = Cube::from_colors(&colors);
    assert!(result.is_err());
}

// ============================================================
// cube/validation.rs カバレッジ: twist パリティエラー
// ============================================================

#[test]
fn test_check_corner_twist_parity_error() {
    // UFL[2,9,16] を時計回りに1ねじる: [White,Green,Red]→[Red,White,Green]
    // 色数は保持, コーナーの重複なし, 対面色なし, だが twist 合計 = 1 ≢ 0 (mod 3)
    let mut cube = Cube::new();
    cube.stickers[2].color = Color::Red; // was White
    cube.stickers[9].color = Color::White; // was Green
    cube.stickers[16].color = Color::Green; // was Red
    assert!(cube.is_valid_state().is_err());
}

// ============================================================
// solver.rs カバレッジ: 進捗インターバルのゼロ以外パス（forward + backward）
// ============================================================

#[test]
fn test_solve_with_progress_nonzero_depth_interval() {
    // 1手スクランブル, max_depth=3 → forward_depth=2 で depth 1 を探索
    // depth 1 % 4 != 0 → forward progress を送らないパスを通る
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    let (tx, _rx) = mpsc::channel::<f32>();
    let sol = solver::solve_with_progress(&cube, 3, true, Some(tx));
    assert!(sol.found);
}

#[test]
fn test_solve_with_progress_backward_depth_interval() {
    // 3手スクランブル, max_depth=4 → backward_depth=2 で backward depth 1 を探索
    // backward depth 1 % 4 != 0 → backward progress を送らないパスを通る
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);
    cube.apply_move(Move::R);
    let (tx, _rx) = mpsc::channel::<f32>();
    let sol = solver::solve_with_progress(&cube, 4, true, Some(tx));
    assert!(sol.found);
}

// from_file_formatのエラーケース追加テスト
#[test]
fn test_from_file_format_too_few_lines() {
    let invalid = "     WWWW\nGGGG RRRR BBBB OOOO\n";
    assert!(Cube::from_file_format(invalid).is_err());
}

#[test]
fn test_from_file_format_line1_too_short() {
    let invalid = "     WWW\nGGGG RRRR BBBB OOOO\n     YYYY\n";
    assert!(Cube::from_file_format(invalid).is_err());
}

#[test]
fn test_from_file_format_line2_too_short() {
    let invalid = "     WWWW\nGGGG RRRR BBBB OOO\n     YYYY\n";
    assert!(Cube::from_file_format(invalid).is_err());
}

#[test]
fn test_from_file_format_line3_too_short() {
    let invalid = "     WWWW\nGGGG RRRR BBBB OOOO\n     YYY\n";
    assert!(Cube::from_file_format(invalid).is_err());
}

#[test]
fn test_from_file_format_line2_invalid_char() {
    let invalid = "     WWWW\nGGGX RRRR BBBB OOOO\n     YYYY\n";
    assert!(Cube::from_file_format(invalid).is_err());
}

#[test]
fn test_from_file_format_line3_invalid_char() {
    let invalid = "     WWWW\nGGGG RRRR BBBB OOOO\n     YYYX\n";
    assert!(Cube::from_file_format(invalid).is_err());
}

#[test]
fn test_validate_colors_missing_color() {
    // すべての色が欠けているケース
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
}

#[test]
fn test_restore_orientation_instantly_invalid_colors() {
    let mut cube = Cube::new();
    // 色を不正にする (Whiteを1つGrayにする)
    cube.stickers[0].color = Color::Gray;
    // restore_orientation_instantly 内の validate_colors でエラーになるはず
    assert!(cube.restore_orientation_instantly().is_err());
}

#[test]
fn test_restore_orientation_instantly_impossible_corner() {
    let mut cube = Cube::new();
    // UFL[2,9,16] を対面色 (White, Yellow, Red) にする。
    // 色数を維持するために、他所にある Yellow と 9番(Green) を入れ替える。
    // 4番(DFL)はYellow。
    let c9 = cube.stickers[9].color;
    let c4 = cube.stickers[4].color;
    cube.stickers[9].color = c4;
    cube.stickers[4].color = c9;

    // これにより validate_colors はパスするが、
    // UFLコーナーが [White, Yellow, Red] という不正な組み合わせになり、
    // restore_orientation_instantly 内の !found (line 246) に到達する。
    assert!(cube.restore_orientation_instantly().is_err());
}

#[test]
fn test_check_corner_parity_all_twists() {
    // 全てのツイスト (0, 1, 2) が position によって見つかることを確実にする
    let cube = Cube::new();
    // solved cube では全コーナー 0
    assert!(cube.is_valid_state().is_ok());

    // 1手回すと一部のコーナーがねじれる (1 or 2)
    let mut twisted = Cube::new();
    twisted.apply_move(Move::R);
    // 物理的に正しい移動なので is_valid_state は OK だが、内部的には 1 or 2 のツイストが発生している
    assert!(twisted.is_valid_state().is_ok());
}

#[test]
fn test_solve_with_progress_deep() {
    // 確実に depth 4 以上まで探索させるために、神の数(11)のスクランブルを使用
    let god_scramble = "    WGWG\nGRWY BYBR ROBO YOBG\n     OYRW\n";
    let cube = Cube::from_file_format(god_scramble).unwrap();

    let (tx, rx) = mpsc::channel();
    // max_depth=11 なら forward_depth=6, backward_depth=5
    // current_depth 0 と 4 で送信されるはず
    let _sol = solver::solve_with_progress(&cube, 11, true, Some(tx));

    let progress: Vec<f32> = rx.into_iter().collect();
    assert!(progress.len() >= 2);
}

#[test]
fn test_cube_default_equals_new() {
    let cube1 = Cube::default();
    let cube2 = Cube::new();
    assert_eq!(cube1, cube2);
}

#[test]
fn test_color_gray_not_in_solved_cube() {
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
    let invalid = "     WWWW\nGGGG RRRR BBBB OOOO";
    assert!(Cube::from_file_format(invalid).is_err());
}

#[test]
fn test_from_file_format_whitespace_only() {
    let invalid = "     \n      \n     \n";
    assert!(Cube::from_file_format(invalid).is_err());
}

#[test]
fn test_apply_orientation_solution_error() {
    let mut cube = Cube::new();
    // 物理的に不可能な状態（捻じれパリティエラー）を作り出す
    let c0 = cube.stickers[2];
    let c1 = cube.stickers[9];
    let c2 = cube.stickers[16];
    cube.stickers[2] = c1;
    cube.stickers[9] = c2;
    cube.stickers[16] = c0;

    let solution = Solution {
        moves: vec![],
        found: true,
    };
    let result = cube.apply_orientation_solution(&solution);
    assert!(result.is_err());
}

#[test]
fn test_to_file_format_with_gray() {
    let mut cube = Cube::new();
    cube.set_sticker_color(0, Color::Gray);
    let s = cube.to_file_format();
    assert!(s.contains('.'));
}

#[test]
fn test_coverage_gap_cube_default() {
    let cube = <Cube as Default>::default();
    assert!(cube.is_solved());
}

#[test]
fn test_coverage_gap_solver_early_breaks() {
    let cube = Cube::new();
    let sol = solver::solve(&cube, 11, true);
    assert!(sol.found);
    assert_eq!(sol.moves.len(), 0);
}

#[test]
fn test_coverage_gap_solver_forward_queue_empty() {
    let cube = Cube::new();
    let mut scrambled = cube.clone();
    scrambled.apply_move(Move::R);
    let sol = solver::solve(&scrambled, 1, true);
    assert!(sol.found);
}

#[test]
fn test_coverage_gap_solver_backward_visited_collision() {
    let cube = Cube::new();
    let sol = solver::solve(&cube, 11, false);
    assert!(sol.found);
}

#[test]
fn test_coverage_gap_solver_not_found_with_progress() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);

    let (tx, rx) = mpsc::channel();
    let solution = solver::solve_with_progress(&cube, 1, false, Some(tx));
    assert!(!solution.found);

    let progress: Vec<f32> = rx.into_iter().collect();
    assert!(progress.contains(&1.0));
}

#[test]
fn test_check_corner_parity_invalid_corner_color_gray() {
    let mut cube = Cube::new();
    // UFL [2, 9, 16] を対面色にならず、かつ白・黄を含まない組み合わせ [Red, Green, Gray] にする。
    cube.stickers[2].color = Color::Red;
    cube.stickers[9].color = Color::Green;
    cube.stickers[16].color = Color::Gray;

    // validation::check_corner_parity を直接呼び出すことで、
    // 前段の is_valid_state() 内の validate_colors() (色数チェック) でエラーにならずに、
    // 不正なコーナー色の判定（pos_opt = None）へ到達させます。
    let result = rubiks_cube_2x2::cube::validation::check_corner_parity(&cube);
    assert!(result.is_err());
}

#[test]
fn test_coverage_gap_solver_forward_queue_empty_real() {
    // ほとんどの色が Gray で、1つだけが Red のキューブを作成する。
    // これにより is_solved() = false になり、かつ向き無視 (ignore_orientation = true) での
    // 状態空間のサイズが最大 24 個（Redステッカーの位置）に制限されます。
    let mut cube = Cube::new();
    for sticker in &mut cube.stickers {
        sticker.color = Color::Gray;
        sticker.orientation = 0;
    }
    cube.stickers[0].color = Color::Red; // 1つだけRedにする

    // 向き無視 (true), 最大深度 10 (forward_depth = 5) で探索。
    // 状態空間が非常に小さいため、すぐに全探索が完了してキューが空になり、
    // queue.is_empty() による break が確実に走ります。
    let sol = solver::solve(&cube, 10, true);
    assert!(!sol.found);
}
