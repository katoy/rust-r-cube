use rubiks_cube_2x2::cube::{Color, Cube, Move};
use rubiks_cube_2x2::solver::coord::RawCube;
use rubiks_cube_2x2::solver::search::Search;
use rubiks_cube_2x2::solver::{is_fully_solved, solve, solve_with_progress, Solution, SolverState};
use std::sync::mpsc;

#[test]
fn test_solve_already_solved() {
    let cube = Cube::new();
    let solution = solve(&cube, 11, true);
    assert!(solution.found);
    assert_eq!(solution.moves.len(), 0);
}

#[test]
fn test_solve_one_move() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);

    let solution = solve(&cube, 11, true);
    assert!(solution.found);
    assert_eq!(solution.moves.len(), 1);
    assert_eq!(solution.moves[0], Move::R.inverse());
}

#[test]
fn test_solve_two_moves() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);

    let solution = solve(&cube, 11, true);
    assert!(solution.found);
    assert_eq!(solution.moves.len(), 2);

    let mut check_cube = cube.clone();
    for &mv in &solution.moves {
        check_cube.apply_move(mv);
    }
    assert!(check_cube.is_solved());
}

#[test]
fn test_solve_checker_pattern() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R2);
    cube.apply_move(Move::U2);
    cube.apply_move(Move::F2);

    let solution = solve(&cube, 11, true);
    assert!(solution.found);

    let mut check_cube = cube.clone();
    for &mv in &solution.moves {
        check_cube.apply_move(mv);
    }
    assert!(check_cube.is_solved());
}

#[test]
fn test_solve_deep_scramble() {
    let mut cube = Cube::new();
    let moves = vec![
        Move::R,
        Move::U,
        Move::F,
        Move::Rp,
        Move::Up,
        Move::Fp,
        Move::R,
    ];
    for &mv in &moves {
        cube.apply_move(mv);
    }

    let solution = solve(&cube, 11, true);
    assert!(solution.found);

    let mut check_cube = cube.clone();
    for &mv in &solution.moves {
        check_cube.apply_move(mv);
    }
    assert!(check_cube.is_solved());
}

#[test]
fn test_solve_depth_limit() {
    let mut cube = Cube::new();
    cube.scramble(10);
    let solution = solve(&cube, 0, true);
    assert!(!solution.found);
}

#[test]
fn test_solve_with_orientation_change() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    let solution = solve(&cube, 11, true);
    assert!(solution.found);
}

#[test]
fn test_solve_rotated_and_scrambled() {
    let mut cube = Cube::new();
    cube.apply_move(Move::U);
    cube.apply_move(Move::Dp);
    cube.apply_move(Move::R);

    let solution = solve(&cube, 11, true);
    assert!(solution.found);

    let mut check_cube = cube.clone();
    for &mv in &solution.moves {
        check_cube.apply_move(mv);
    }
    assert!(check_cube.is_solved());
}

#[test]
fn test_solution_struct() {
    let solution = Solution {
        moves: vec![Move::R, Move::U],
        found: true,
    };
    assert!(solution.found);
    assert_eq!(solution.moves.len(), 2);
    let solution2 = solution.clone();
    assert_eq!(solution.moves, solution2.moves);
    assert_eq!(solution.found, solution2.found);
}

#[test]
fn test_solve_fully_aligned() {
    let mut cube = Cube::new();
    cube.apply_move(Move::U);
    cube.apply_move(Move::Dp);

    assert!(cube.is_solved());
    assert!(is_fully_solved(&cube));

    let sol_align = solve(&cube, 11, false);
    assert!(sol_align.found);
    assert_eq!(sol_align.moves.len(), 0);

    cube.apply_move(Move::R);
    assert!(!cube.is_solved());
    assert!(!is_fully_solved(&cube));

    let sol_align2 = solve(&cube, 11, false);
    assert!(sol_align2.found);
    assert!(!sol_align2.moves.is_empty());
}

#[test]
fn test_solve_with_progress_details() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);

    let (tx, rx) = mpsc::channel();
    let solution = solve_with_progress(&cube, 11, false, Some(tx));

    assert!(solution.found);
    let progress_values: Vec<f32> = rx.into_iter().collect();
    assert!(progress_values.contains(&1.0));
    for &p in &progress_values {
        assert!((0.0..=1.0).contains(&p));
    }
}

#[test]
fn test_solve_unsolvable_at_depth() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);
    cube.apply_move(Move::F);
    cube.apply_move(Move::L);
    cube.apply_move(Move::B);

    let solution = solve(&cube, 2, false);
    assert!(!solution.found);
}

#[test]
fn test_solver_state_basic() {
    let cube = Cube::new();
    let mut state = SolverState::new(&cube, 11, true);
    assert!(state.get_solution().is_none());
    assert_eq!(state.estimate_progress(), 0.5);

    let (nodes, finished) = state.process_chunk(100);
    assert_eq!(nodes, 1);
    assert!(finished);
    assert!(state.get_solution().is_some());
    assert_eq!(state.estimate_progress(), 1.0);

    let (nodes2, finished2) = state.process_chunk(100);
    assert_eq!(nodes2, 0);
    assert!(finished2);
}

#[test]
fn test_solver_state_process_chunk_twice() {
    let cube = Cube::new();
    let mut state = SolverState::new(&cube, 1, false);
    state.process_chunk(100);
    let (count, finished2) = state.process_chunk(100);
    assert_eq!(count, 0);
    assert!(finished2);
}

#[test]
fn test_search_node_limit_reached() {
    let mut search = Search::new();
    search.max_nodes = 1;
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);
    let rc = RawCube::from_cube(&cube, &[0, 1, 2, 3, 4, 5]).unwrap();
    let result = search.solve(&rc, 2);
    assert!(result.is_none());
}

#[test]
fn test_solve_invalid_cube_colors_for_err_path() {
    let mut cube = Cube::new();
    cube.stickers[4].color = Color::White;
    let result = solve(&cube, 1, false);
    assert!(!result.found);
}

#[test]
fn test_solve_shorter_hidden_orientation() {
    let mut scrambled = Cube::new();
    scrambled.apply_move(Move::R);
    scrambled.apply_move(Move::Lp);
    scrambled.apply_move(Move::R);
    let result = solve(&scrambled, 4, true);
    assert!(result.found);
    assert_eq!(result.moves.len(), 1);
}

#[test]
fn test_move_cubes_basic() {
    for m_idx in 0..6 {
        let rc_move = RawCube::move_cube(m_idx);
        assert_ne!(rc_move.cp, RawCube::default().cp);
    }
}

#[test]
fn test_move_table_consistency() {
    use rubiks_cube_2x2::solver::tables::MoveTable;
    let mt = MoveTable::get();
    let mut cube = Cube::new();
    cube.apply_move(Move::U);
    let rc = RawCube::from_cube(&cube, &[0, 1, 2, 3, 4, 5]).unwrap();
    let cp = rc.get_cp();
    let next_cp = mt.cp[cp as usize][2]; // m=2 is Up
    assert_eq!(next_cp, 0);
}

#[test]
fn test_move_translation_diagnostic() {
    let cube = Cube::new();
    let mut rotated = cube.clone();
    rotated.apply_move(Move::U);
    rotated.apply_move(Move::Dp);
    rotated.apply_move(Move::R);

    // X, Y, Z は手動で回転を適用して同等の状態を作る
    let mut target = cube.clone();
    target.apply_move(Move::B);
    target.apply_move(Move::U);
    target.apply_move(Move::Dp);

    // Y rotation then R move should be same as B move then Y rotation
    assert_eq!(rotated, target);
}
#[test]
fn test_solver_extra_coverage() {
    // Transferred from src/solver/mod.rs
    let cube = Cube::new();
    let sol = solve(&cube, 11, true);
    assert!(sol.found);

    let state = SolverState::new(&cube, 11, true);
    assert!(state.get_solution().is_none());
    assert!(state.estimate_progress() < 1.0);

    // solve_with_progress
    let (tx, _rx) = std::sync::mpsc::channel();
    let _ = solve_with_progress(&cube, 1, true, Some(tx));
}

/// ignore_orientation=false のとき、色は揃っているがステッカーの向きが
/// 期待値と異なるキューブを渡すと、is_solved_with_orientation() が false を返すため
/// 解法の検証チェック（solver/mod.rs の continue パス）が通ることをカバーする。
#[test]
fn test_solve_orientation_mismatch_triggers_continue() {
    // 解法済みキューブを作成し、全ステッカーの orientation を 0 に強制設定する。
    // Cube::new() は CLOCKWISE_ORIENTATION_PATTERN = [1, 2, 0, 3] を使うが、
    // ここでは全て 0 にして「色は正しいが向きが違う」状態を作る。
    let mut cube = Cube::new();
    for sticker in &mut cube.stickers {
        sticker.orientation = 0;
    }

    // 色は全面揃っているが向きが違う
    assert!(cube.is_solved());
    assert!(!cube.is_solved_with_orientation());

    // ignore_orientation=false で解くと、ソルバーは色ベースで 0 手解を返すが
    // is_solved_with_orientation() が false なので continue し、解なしとなる。
    let solution = solve(&cube, 11, false);
    assert!(!solution.found);
}

/// solve_with_progress を progress_tx=None で呼ぶことで、
/// None 分岐（tx.send を行わないパス）をカバーする。
#[test]
fn test_solve_with_progress_none_tx() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    // None を渡すと進捗送信はスキップされるが、解法は正常に動作する
    let solution = solve_with_progress(&cube, 11, true, None);
    assert!(solution.found);
}

/// Gray ステッカーを含むキューブを solve に渡すと、
/// 色リマップ時に Gray をスキップするパス（solver/mod.rs の if sticker.color != Gray 分岐）を通る。
#[test]
fn test_solve_with_gray_stickers_skips_remap() {
    // 1枚だけ Gray にしたキューブ（is_solved() = false を保ちつつ Gray を含む）
    // sticker[0] は U 面のコーナーステッカー。Gray にすると UBL コーナーが無効になる。
    let mut cube = Cube::new();
    cube.stickers[0].color = Color::Gray;
    // U 面が [Gray, White, White, White] で is_solved() は false
    assert!(!cube.is_solved());
    // RawCube::from_cube が Gray を認識できないため全向きで失敗 → 解なし
    let solution = solve(&cube, 3, true);
    assert!(!solution.found);
}
