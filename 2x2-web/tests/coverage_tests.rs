use rubiks_cube_2x2::cube::{Color, Cube, Move};
use rubiks_cube_2x2::solver::{self, Solution};
use std::sync::mpsc;

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
fn test_validation_err_opposite_colors() {
    let mut cube = Cube::new();
    // コーナー UFL (2, 16, 9) に対面色 (White, Yellow) を持たせる
    // 色数を維持するため、16(Red) と 4(Yellow) を入れ替える
    let red = cube.stickers[16].color;
    let yellow = cube.stickers[4].color;
    cube.stickers[16].color = yellow;
    cube.stickers[4].color = red;
    assert!(cube.is_valid_state().is_err());
}

#[test]
fn test_validation_err_duplicate_corners() {
    let mut cube = Cube::new();
    // コーナー0とコーナー1を重複させる
    let c0_indices = [2, 16, 9];
    let c1_indices = [3, 12, 17];
    let c7_indices = [6, 23, 10];
    let c1_colors = [
        cube.stickers[c1_indices[0]].color,
        cube.stickers[c1_indices[1]].color,
        cube.stickers[c1_indices[2]].color,
    ];

    cube.stickers[c1_indices[0]].color = cube.stickers[c0_indices[0]].color;
    cube.stickers[c1_indices[1]].color = cube.stickers[c0_indices[1]].color;
    cube.stickers[c1_indices[2]].color = cube.stickers[c0_indices[2]].color;

    cube.stickers[c7_indices[0]].color = c1_colors[0];
    cube.stickers[c7_indices[1]].color = c1_colors[1];
    cube.stickers[c7_indices[2]].color = c1_colors[2];
    assert!(cube.is_valid_state().is_err());
}

#[test]
fn test_validation_err_invalid_color_corner() {
    let mut cube = Cube::new();
    // コーナー UFL から White を追い出し、Red を2つ持たせる
    let c2 = cube.stickers[2].color;
    let c17 = cube.stickers[17].color; // Red
    cube.stickers[2].color = c17;
    cube.stickers[17].color = c2;
    assert!(cube.is_valid_state().is_err());
}

#[test]
fn test_solver_state_basic() {
    let cube = Cube::new();
    let mut state = solver::SolverState::new(&cube, 11, true);
    assert!(state.get_solution().is_none());
    assert_eq!(state.estimate_progress(), 0.5);

    let (nodes, finished) = state.process_chunk(100);
    assert_eq!(nodes, 1);
    assert!(finished);
    assert!(state.get_solution().is_some());
    assert_eq!(state.estimate_progress(), 1.0);

    // すでに終了している場合の process_chunk
    let (nodes2, finished2) = state.process_chunk(100);
    assert_eq!(nodes2, 0);
    assert!(finished2);
}

#[test]
fn test_search_default_and_fully_solved() {
    use rubiks_cube_2x2::solver::is_fully_solved;
    use rubiks_cube_2x2::solver::search::Search;
    let _search = Search::default();

    let cube = Cube::new();
    assert!(is_fully_solved(&cube));

    let mut scrambled = cube.clone();
    scrambled.apply_move(Move::R);
    assert!(!is_fully_solved(&scrambled));

    // 全ての完成状態（24通り）をテスト
    for state in rubiks_cube_2x2::solver::get_solved_states() {
        assert!(is_fully_solved(state));
    }
}

#[test]
fn test_solve_with_progress_sender() {
    use std::sync::mpsc::channel;
    let cube = Cube::new();
    let (tx, rx) = channel();

    // すでに解けている場合でも進捗（0.0, 1.0）が送られるか
    let _ = rubiks_cube_2x2::solver::solve_with_progress(&cube, 1, false, Some(tx));

    let mut progress = Vec::new();
    while let Ok(val) = rx.try_recv() {
        progress.push(val);
    }
    assert!(progress.contains(&0.0));
    assert!(progress.contains(&1.0));
}

#[test]
fn test_solve_invalid_cube_colors_for_err_path() {
    let mut cube = Cube::new();
    // カラーカウントを壊す (White が 5個)
    cube.stickers[4].color = Color::White;

    //RawCube::from_cube が Err を返し、内部の solve 処理がスキップされるはず
    let result = rubiks_cube_2x2::solver::solve(&cube, 1, false);
    assert!(!result.found);
}

#[test]
fn test_solver_state_process_chunk_twice() {
    let cube = Cube::new();
    let mut state = rubiks_cube_2x2::solver::SolverState::new(&cube, 1, false);

    let (_, finished) = state.process_chunk(100);
    assert!(finished);

    // 2回目は早期リターンするはず
    let (count, finished2) = state.process_chunk(100);
    assert_eq!(count, 0);
    assert!(finished2);
    assert_eq!(state.estimate_progress(), 1.0);
}

#[test]
fn test_solve_shorter_hidden_orientation() {
    let cube = Cube::new();
    // X回転 (R L') してから R を回す
    // この状態は、標準向きからは [L R2] の 2手だが、
    // X回転した向きからは [R'] の 1手で解けるはず。
    let mut scrambled = cube.clone();
    scrambled.apply_move(Move::R);
    scrambled.apply_move(Move::Lp);
    scrambled.apply_move(Move::R);

    // ignore_orientation = true なら 1手の解が見つかるはず
    // この過程で「より短い解に更新する」パス (mod.rs:180) が通る
    let result = rubiks_cube_2x2::solver::solve(&scrambled, 4, true);
    assert!(result.found);
    assert_eq!(result.moves.len(), 1);
}

#[test]
fn test_search_node_limit_reached() {
    use rubiks_cube_2x2::solver::search::Search;
    let mut search = Search::new();
    search.max_nodes = 1; // 極端に小さくする
    let cube = Cube::new();
    let mut scrambled = cube.clone();
    scrambled.apply_move(Move::R);
    scrambled.apply_move(Move::U);
    let rc = rubiks_cube_2x2::solver::coord::RawCube::from_cube(&scrambled, &[0, 1, 2, 3, 4, 5])
        .unwrap();

    // 深さ1以上で探索すればノード制限に引っかかるはず
    let result = search.solve(&rc, 2);
    assert!(result.is_none());
}

#[test]
fn test_validation_check_corner_parity_direct() {
    use rubiks_cube_2x2::cube::validation::check_corner_parity;
    let mut cube = Cube::new();

    // 重複コーナーの誘発 (Corner 1 を Corner 0 のコピーにする)
    // Corner 0: [2, 16, 9] (UFL)
    // Corner 1: [3, 12, 17] (UFR)
    cube.stickers[3].color = cube.stickers[2].color;
    cube.stickers[12].color = cube.stickers[16].color;
    cube.stickers[17].color = cube.stickers[9].color;

    assert!(check_corner_parity(&cube).is_err());

    // 不正色コーナーの誘発 (White/Yellow がない)
    let mut cube2 = Cube::new();
    cube2.stickers[2].color = Color::Red;
    cube2.stickers[16].color = Color::Green;
    cube2.stickers[9].color = Color::Blue;
    assert!(check_corner_parity(&cube2).is_err());
}

#[test]
fn test_history_redo_count() {
    use rubiks_cube_2x2::history::History;
    let mut history = History::new();
    history.push(Move::R);
    history.undo();
    assert_eq!(history.undo_count(), 0);
    assert_eq!(history.redo_count(), 1);
}

#[test]
fn test_restore_orientation_invalid_piece() {
    let mut cube = Cube::new();
    // コーナー 0 (UFL) の色を、物理的に不可能な組み合わせ (White-White-Green) にする
    // カウントを維持するため、ステッカー 16 (本来 Red) を White にし、
    // ステッカー 1 (本来 White) を Red にします。
    cube.stickers[2].color = Color::White;
    cube.stickers[16].color = Color::White; // 重複 (本来 Red)
    cube.stickers[9].color = Color::Green;

    // 他で調整
    cube.stickers[1].color = Color::Red; // counts are preserved

    // restore_orientation_instantly はこの組み合わせを solved_states から見つけられず Err を返すはず
    let result = cube.restore_orientation_instantly();
    assert!(result.is_err());
}
