use crate::cube::{Cube, Face, Move};
use crate::kociemba::{RawCube, Search};
use glam::Vec3;
use std::sync::OnceLock;

pub const DEFAULT_MAX_DEPTH: usize = 128;

// ランダム探索パラメータ
const RANDOM_TRIALS: usize = 3000;
const RANDOM_SEED: usize = 999888777;
const LCG_MULTIPLIER: usize = 1103515245;
const LCG_INCREMENT: usize = 12345;
const PROGRESS_WEIGHT: f32 = 0.9;
const MAX_SETUP_MOVES: usize = 6;
const TOTAL_BASIC_MOVES: usize = 18;
const TOTAL_ROTATIONS: usize = 24;

#[derive(Debug, Clone)]
pub struct Solution {
    pub moves: Vec<Move>,
    pub found: bool,
    pub message: String,
}

#[cfg(any(target_arch = "wasm32", test))]
pub struct SolverState {
    raw_cube: Result<RawCube, String>,
    initial_cube: Cube,
    max_depth: usize,
    ignore_orientation: bool,
    solution: Option<Solution>,
    finished: bool,
}

static SOLVED_ORIS: OnceLock<Vec<Vec<u8>>> = OnceLock::new();
static SOLVED_STATES: OnceLock<Vec<Cube>> = OnceLock::new();

pub fn get_solved_oris() -> &'static [Vec<u8>] {
    SOLVED_ORIS.get_or_init(generate_all_solved_oris)
}

pub fn get_solved_states() -> &'static [Cube] {
    SOLVED_STATES.get_or_init(generate_all_solved_states)
}

fn generate_all_solved_states() -> Vec<Cube> {
    use rustc_hash::FxHashSet;
    use std::collections::VecDeque;

    let base = Cube::new();
    let mut states = Vec::new();
    let mut queue = VecDeque::new();
    let mut visited: FxHashSet<Cube> = FxHashSet::default();

    queue.push_back(base.clone());
    visited.insert(base.clone());
    states.push(base);

    let basic_rotations = [Move::X, Move::Y, Move::Z];

    while let Some(current) = queue.pop_front() {
        for &rot in &basic_rotations {
            let mut next = current.clone();
            next.apply_move(rot);
            if visited.insert(next.clone()) {
                states.push(next.clone());
                queue.push_back(next);
            }
        }
    }
    states
}

fn generate_all_solved_oris() -> Vec<Vec<u8>> {
    let states = get_solved_states();
    states.iter().map(get_orientations_vec).collect()
}

pub fn is_fully_solved(cube: &Cube) -> bool {
    if !cube.is_solved() {
        return false;
    }

    let current_oris = get_orientations_vec(cube);
    let solved_oris = get_solved_oris();
    let mut matched_pattern = None;
    for (i, target) in solved_oris.iter().enumerate() {
        if current_oris == *target {
            matched_pattern = Some(i);
            break;
        }
    }

    if std::env::var("SOLVER_DEBUG").is_ok() {
        if let Some(pattern_idx) = matched_pattern {
            println!(
                "DEBUG: is_fully_solved: MATCHED orientation pattern {}! RelativeOris={:?}",
                pattern_idx, current_oris
            );
        } else {
            println!(
                "DEBUG: is_fully_solved: Color=Solved, RelativeOris={:?}",
                current_oris
            );
            // 全パターンを詳細比較
            for (i, target) in solved_oris.iter().enumerate() {
                if i % 8 == 0 {
                    print!("  ");
                }
                print!("P{}:{} ", i, current_oris == *target);
                if i % 8 == 7 {
                    println!();
                }
            }
        }
    }

    matched_pattern.is_some()
}

pub fn is_orientation_solvable(cube: &Cube) -> bool {
    if !cube.is_solved() {
        return false;
    }
    let oris = get_orientations_vec(cube);
    let total_ori: u32 = oris.iter().map(|&o| o as u32).sum();
    total_ori.is_multiple_of(2)
}

pub fn solve_with_progress(
    start_cube: &Cube,
    max_depth: usize,
    ignore_orientation: bool,
    progress_tx: Option<std::sync::mpsc::Sender<f32>>,
) -> Solution {
    solve_internal(start_cube, max_depth, ignore_orientation, progress_tx)
}

pub fn solve(start_cube: &Cube, max_depth: usize, ignore_orientation: bool) -> Solution {
    // 物理的妥当性チェック
    if let Err(e) = start_cube.is_valid_state() {
        return Solution {
            moves: vec![],
            found: false,
            message: format!("無効なキューブ状態: {}", e),
        };
    }

    solve_internal(start_cube, max_depth, ignore_orientation, None)
}

/// プログレス報告を抽象化するヘルパー
struct ProgressReporter {
    tx: Option<std::sync::Mutex<std::sync::mpsc::Sender<f32>>>,
}

impl ProgressReporter {
    fn new(tx: Option<std::sync::mpsc::Sender<f32>>) -> Self {
        Self {
            tx: tx.map(std::sync::Mutex::new),
        }
    }

    fn report(&self, progress: f32) {
        if let Some(ref tx_mutex) = self.tx {
            if let Ok(tx) = tx_mutex.lock() {
                let _ = tx.send(progress);
            }
        }
    }
}

pub enum TrySolveResult {
    Perfect(Solution),
    ColorOnly(Solution),
}

/// setup_moves と rotation を適用してソルバーを実行し、解の検証と向き修正を行う
pub fn try_solve_with_rotation(
    start_cube: &Cube,
    setup_moves: &[Move],
    rotation: &[Move],
    max_depth: usize,
    ignore_orientation: bool,
    search: &mut Search,
) -> Option<TrySolveResult> {
    // setup_moves + rotation を適用したキューブを作成
    let mut test_cube = start_cube.clone();
    for &m in setup_moves {
        test_cube.apply_move(m);
    }
    for &m in rotation {
        test_cube.apply_move(m);
    }

    // RawCube に変換
    let rc = RawCube::from_cube(&test_cube).ok()?;

    // 必要な手数を計算
    let needed = setup_moves.len() + rotation.len();
    if needed >= max_depth {
        return None;
    }

    // Kociemba ソルバーで解を探索
    let m_fix = search.solve(&rc, max_depth.saturating_sub(needed))?;

    // 全ての手順を結合
    let mut moves = Vec::new();
    moves.extend_from_slice(setup_moves);
    moves.extend_from_slice(rotation);
    moves.extend(m_fix);

    // 解を適用して検証
    let mut check_cube = start_cube.clone();
    for &m in &moves {
        check_cube.apply_move(m);
    }

    // 完全に解けている場合 (方位も含めて)
    if is_fully_solved(&check_cube) && moves.len() <= max_depth {
        return Some(TrySolveResult::Perfect(Solution {
            moves: moves.clone(),
            found: true,
            message: "方位も含めて完全に解決しました。".to_string(),
        }));
    }

    // 向きを無視する場合に限り、色が揃っていれば即座に返す
    if ignore_orientation && check_cube.is_solved() && moves.len() <= max_depth {
        return Some(TrySolveResult::Perfect(Solution {
            moves: moves.clone(),
            found: true,
            message: "色の揃った解（向きは無視）を見つけました。".to_string(),
        }));
    }

    // 色が揃っている場合、向き修正を試みる
    if check_cube.is_solved() {
        if !is_orientation_solvable(&check_cube) {
            let oris = get_orientations_vec(&check_cube);
            let sum: u32 = oris.iter().map(|&o| o as u32).sum();
            if std::env::var("SOLVER_DEBUG").is_ok() {
                println!("DEBUG: try_solve_with_rotation: Odd parity detected (sum={}). Physically impossible.", sum);
            }
            return Some(TrySolveResult::ColorOnly(Solution {
                moves: moves.clone(),
                found: ignore_orientation, // 向きを無視する場合のみ "成功" とみなす
                message: if ignore_orientation {
                    format!("色は解決しましたが、方位パリティが異常(sum={})なため、方位の解決は不可能です。", sum)
                } else {
                    format!("方位パリティが異常(sum={})なため、解決できません。物理的に不可能な状態です。", sum)
                },
            }));
        }

        if std::env::var("SOLVER_DEBUG").is_ok() {
            println!(
                "DEBUG: try_solve_with_rotation: color solved. oris={:?}",
                get_orientations_vec(&check_cube)
            );
        }
        let fixes = apply_supercube_fixes(&check_cube, search);
        if std::env::var("SOLVER_DEBUG").is_ok() {
            println!(
                "DEBUG: try_solve_with_rotation: apply_supercube_fixes returned {} moves.",
                fixes.len()
            );
        }
        let mut final_moves = moves.clone();
        final_moves.extend(fixes.clone());

        let mut final_cube = check_cube.clone();
        for &m in &fixes {
            final_cube.apply_move(m);
        }

        if std::env::var("SOLVER_DEBUG").is_ok() {
            println!(
                "DEBUG: try_solve_with_rotation: after fixes, oris={:?}, is_solved={}",
                get_orientations_vec(&final_cube),
                final_cube.is_solved()
            );
        }

        if is_fully_solved(&final_cube) {
            if final_moves.len() <= max_depth {
                return Some(TrySolveResult::Perfect(Solution {
                    moves: final_moves,
                    found: true,
                    message: "色解決後にセンターの向きを修正しました。".to_string(),
                }));
            } else {
                return Some(TrySolveResult::ColorOnly(Solution {
                    moves: final_moves,
                    found: ignore_orientation,
                    message: "色は揃いましたが、向きの修正を含めると探索深度を超えます。"
                        .to_string(),
                }));
            }
        }
    }
    // ignore_orientation が false の場合、色が揃っていない状態は color_only_solution として保存しない
    // (color_only_solution は色が揃っているが向きが揃っていない状態を保存するために使用される)
    // また、色が揃っているが向きが揃わない状態（パリティエラー）も、ignore_orientation=false の場合は
    // color_only_solution には保存しない。
    // このブロックは、check_cube.is_solved() が false の場合にのみ到達する。
    // そのため、color_only_solution には、色が揃った状態のみを保存する。
    // したがって、この else if ブロックは削除する。

    None
}

fn solve_internal(
    start_cube: &Cube,
    max_depth: usize,
    ignore_orientation: bool,
    progress_tx: Option<std::sync::mpsc::Sender<f32>>,
) -> Solution {
    let progress = ProgressReporter::new(progress_tx);
    progress.report(0.0);

    if is_fully_solved(start_cube) {
        progress.report(1.0);
        return Solution {
            moves: vec![],
            found: true,
            message: "既に完全に解決されています。".to_string(),
        };
    }
    // 向きを無視する場合に限り、色が揃っていれば即座に返す
    if ignore_orientation && start_cube.is_solved() {
        progress.report(1.0);
        return Solution {
            moves: vec![],
            found: true,
            message: "既に色が揃っています（向きは無視）。".to_string(),
        };
    }

    let mut color_only_solution: Option<Solution> = None;

    // 色が揃っているが向きが不完全な場合、即座に方位修正を試みる (探索をバイパス)
    if start_cube.is_solved() {
        let mut search = Search::new();
        if let Some(res) = try_solve_with_rotation(
            start_cube,
            &[], // setup_moves
            &[], // rot
            max_depth,
            false, // ignore_orientation=false
            &mut search,
        ) {
            match res {
                TrySolveResult::Perfect(sol) => {
                    progress.report(1.0);
                    return sol;
                }
                TrySolveResult::ColorOnly(sol) => {
                    color_only_solution = Some(sol);
                }
            }
        }
    }

    let mut search = Search::new();
    let rotations = get_all_rotations();

    // 1. 直近方位試行
    for rot in &rotations {
        if let Some(res) = try_solve_with_rotation(
            start_cube,
            &[],
            rot,
            max_depth,
            ignore_orientation,
            &mut search,
        ) {
            match res {
                TrySolveResult::Perfect(sol) => {
                    progress.report(1.0);
                    return sol;
                }
                TrySolveResult::ColorOnly(sol) => {
                    if color_only_solution.is_none() {
                        color_only_solution = Some(sol);
                    }
                }
            }
        }
    }

    // ignore_orientation が true の場合、色が揃った解があればそれを優先
    if ignore_orientation {
        if let Some(sol) = color_only_solution.clone() {
            return sol;
        }
    }

    // 2. 超軽量・高密度試行 (2000試行)
    let all_moves = Move::all_moves();
    use crate::kociemba::DEFAULT_MAX_NODES;

    #[cfg(not(target_arch = "wasm32"))]
    {
        use rayon::prelude::*;
        use std::sync::Mutex;

        let color_only_mutex = Mutex::new(color_only_solution);

        let perfect_sol = (0..RANDOM_TRIALS)
            .into_par_iter()
            .find_map_any(|trial_iter| {
                let mut local_search = Search::new();
                // 探索が難航している場合、早期にノード制限を引き上げる
                if trial_iter > 100 {
                    local_search.max_nodes = DEFAULT_MAX_NODES * 5;
                }
                if trial_iter > 500 {
                    local_search.max_nodes = DEFAULT_MAX_NODES * 10;
                }
                if trial_iter > 1000 {
                    local_search.max_nodes = DEFAULT_MAX_NODES * 25;
                }

                // スレッドごとに異なるシードを作成
                let mut local_seed = RANDOM_SEED.wrapping_add(trial_iter * 12345);
                let next_rn_local = |s: &mut usize| -> usize {
                    *s = s.wrapping_mul(LCG_MULTIPLIER).wrapping_add(LCG_INCREMENT);
                    (*s / 65536) % 32768
                };

                let n_random = (next_rn_local(&mut local_seed) % MAX_SETUP_MOVES) + 1;
                let mut setup_moves = vec![];
                for _ in 0..n_random {
                    let m = all_moves[next_rn_local(&mut local_seed) % TOTAL_BASIC_MOVES];
                    setup_moves.push(m);
                }
                let rot = &rotations[next_rn_local(&mut local_seed) % TOTAL_ROTATIONS];

                if let Some(res) = try_solve_with_rotation(
                    start_cube,
                    &setup_moves,
                    rot,
                    max_depth,
                    ignore_orientation,
                    &mut local_search,
                ) {
                    match res {
                        TrySolveResult::Perfect(sol) => return Some(sol),
                        TrySolveResult::ColorOnly(sol) => {
                            let mut guard = color_only_mutex.lock().unwrap();
                            if guard.is_none() {
                                *guard = Some(sol);
                            }
                        }
                    }
                }
                // 100回ごとにプログレス報告 (適当な頻度)
                if trial_iter % 100 == 0 {
                    progress.report(trial_iter as f32 / RANDOM_TRIALS as f32 * PROGRESS_WEIGHT);
                }
                None
            });

        if let Some(sol) = perfect_sol {
            progress.report(1.0);
            return sol;
        }
        color_only_solution = color_only_mutex.into_inner().unwrap();
    }

    #[cfg(target_arch = "wasm32")]
    {
        let mut seed: usize = RANDOM_SEED;
        let mut next_rn = |s: &mut usize| -> usize {
            *s = s.wrapping_mul(LCG_MULTIPLIER).wrapping_add(LCG_INCREMENT);
            (*s / 65536) % 32768
        };

        for trial_iter in 0..RANDOM_TRIALS {
            // 探索が難航している場合、早期にノード制限を引き上げる
            if trial_iter > 100 {
                search.max_nodes = DEFAULT_MAX_NODES * 5;
            }
            if trial_iter > 500 {
                search.max_nodes = DEFAULT_MAX_NODES * 10;
            }
            if trial_iter > 1000 {
                search.max_nodes = DEFAULT_MAX_NODES * 25;
            }

            progress.report(trial_iter as f32 / RANDOM_TRIALS as f32 * PROGRESS_WEIGHT);
            let n_random = (next_rn(&mut seed) % MAX_SETUP_MOVES) + 1;
            let mut setup_moves = vec![];
            for _ in 0..n_random {
                let m = all_moves[next_rn(&mut seed) % TOTAL_BASIC_MOVES];
                setup_moves.push(m);
            }
            let rot = &rotations[next_rn(&mut seed) % TOTAL_ROTATIONS];

            if let Some(res) = try_solve_with_rotation(
                start_cube,
                &setup_moves,
                rot,
                max_depth,
                ignore_orientation,
                &mut search,
            ) {
                match res {
                    TrySolveResult::Perfect(sol) => {
                        progress.report(1.0);
                        return sol;
                    }
                    TrySolveResult::ColorOnly(sol) => {
                        if color_only_solution.is_none() {
                            color_only_solution = Some(sol);
                        }
                    }
                }
            }
        }
    }

    if std::env::var("SOLVER_DEBUG").is_ok() {
        println!(
            "DEBUG: solve_internal: no exact solution found. color_only={}",
            color_only_solution.is_some()
        );
    }

    if let Some(sol) = color_only_solution {
        progress.report(1.0);
        return sol;
    }

    progress.report(1.0);
    let mut msg = "解が見つかりませんでした。".to_string();
    if !is_orientation_solvable(start_cube) {
        let oris = get_orientations_vec(start_cube);
        let sum: u32 = oris.iter().map(|&o| o as u32).sum();
        msg += &format!(
            "方位パリティが異常(sum={})なため、現在の色配置のままでは解決不可能です。",
            sum
        );
    } else {
        msg += "物理的に不可能な状態か、探索深度（最大128手）を超えている可能性があります。";
    }

    Solution {
        moves: vec![],
        found: false,
        message: msg,
    }
}

pub fn get_orientations_vec(cube: &Cube) -> Vec<u8> {
    Face::all()
        .iter()
        .map(|f| cube.stickers[f.start_index() + 4].orientation)
        .collect()
}

pub fn undo_setup(mut setup: Vec<Move>) -> Vec<Move> {
    for m in &mut setup {
        *m = m.inverse();
    }
    setup.reverse();
    setup
}

pub fn get_all_rotations() -> Vec<Vec<Move>> {
    vec![
        vec![],
        vec![Move::X],
        vec![Move::X2],
        vec![Move::Xp],
        vec![Move::Y],
        vec![Move::Y2],
        vec![Move::Yp],
        vec![Move::Z],
        vec![Move::Z2],
        vec![Move::Zp],
        vec![Move::X, Move::Y],
        vec![Move::X, Move::Y2],
        vec![Move::X, Move::Yp],
        vec![Move::X, Move::Z],
        vec![Move::X, Move::Z2],
        vec![Move::X, Move::Zp],
        vec![Move::Xp, Move::Y],
        vec![Move::Xp, Move::Y2],
        vec![Move::Xp, Move::Yp],
        vec![Move::Xp, Move::Z],
        vec![Move::Xp, Move::Z2],
        vec![Move::Xp, Move::Zp],
        vec![Move::X2, Move::Y],
        vec![Move::X2, Move::Z],
    ]
}

fn get_target_oris(cube: &Cube) -> Vec<u8> {
    let states = get_solved_states();
    for (_i, s) in states.iter().enumerate() {
        let mut match_centers = true;
        for f in Face::all() {
            let sc = s.stickers[f.start_index() + 4].color;
            let cc = cube.stickers[f.start_index() + 4].color;
            if sc != cc {
                match_centers = false;
                break;
            }
        }
        if match_centers {
            if std::env::var("SOLVER_DEBUG").is_ok() {
                println!("DEBUG: get_target_oris: Matched solved state pattern {} based on center colors.", _i);
                let colors: Vec<_> = Face::all()
                    .iter()
                    .map(|f| s.stickers[f.start_index() + 4].color)
                    .collect();
                println!("DEBUG: get_target_oris: Pattern colors={:?}", colors);
            }
            return get_orientations_vec(s);
        }
    }
    if std::env::var("SOLVER_DEBUG").is_ok() {
        println!("DEBUG: get_target_oris: No match found! Falling back to Pattern 0.");
    }
    vec![0, 0, 0, 0, 0, 0]
}

pub fn apply_supercube_fixes(cube: &Cube, _search: &mut Search) -> Vec<Move> {
    let mut current_cube = cube.clone();
    let mut final_moves = Vec::new();
    let target_oris = get_target_oris(cube);

    for iter in 0..12 {
        let oris = get_orientations_vec(&current_cube);
        if std::env::var("SOLVER_DEBUG").is_ok() {
            println!(
                "DEBUG: apply_supercube_fixes: iter={}, oris={:?}, target={:?}",
                iter, oris, target_oris
            );
        }
        if oris == target_oris {
            break;
        }

        // 相対的なズレを計算 (0:なし, 1:CW, 2:180, 3:CCW)
        let mut rel_oris = [0u8; 6];
        for i in 0..6 {
            rel_oris[i] = (oris[i] as i8 - target_oris[i] as i8).rem_euclid(4) as u8;
        }

        let mut d180s = Vec::new();
        let mut d90s = Vec::new(); // (Face, rel_ori)
        for (i, &rel_o) in rel_oris.iter().enumerate() {
            let f = Face::from_index(i * 9);
            if rel_o == 2 {
                d180s.push(f);
            } else if rel_o != 0 {
                d90s.push((f, rel_o));
            }
        }

        let fix = if let Some(&f) = d180s.first() {
            get_fix_180(f)
        } else if d90s.len() >= 2 {
            let (f1, r1) = d90s[0];
            let (f2, r2) = d90s[1];

            if !is_opposite_face(f1, f2) {
                // 90度ペア修正
                if r1 == 1 && r2 == 3 {
                    get_fix_90_pair(f1, f2)
                } else if r1 == 3 && r2 == 1 {
                    get_fix_90_pair(f2, f1)
                } else if r1 == 1 && r2 == 1 {
                    get_fix_90_pair(f1, f2)
                } else {
                    get_fix_90_pair(f2, f1)
                }
            } else {
                // 反対側の面同士の場合、中継面（バッファ）を使用
                let buffer = get_buffer_face(f1, f2);
                if r1 == 1 {
                    get_fix_90_pair(f1, buffer)
                } else {
                    get_fix_90_pair(buffer, f1)
                }
            }
        } else {
            if std::env::var("SOLVER_DEBUG").is_ok() {
                println!(
                    "DEBUG: apply_supercube_fixes: breaking at iter {} with d90s.len={}",
                    iter,
                    d90s.len()
                );
            }
            break;
        };

        if std::env::var("SOLVER_DEBUG").is_ok() {
            let oris_before = get_orientations_vec(&current_cube);
            for &m in &fix {
                current_cube.apply_move(m);
            }
            println!(
                "DEBUG: apply_supercube_fixes: applied fix of len {}. Oris: {:?} -> {:?}",
                fix.len(),
                oris_before,
                get_orientations_vec(&current_cube)
            );
        } else {
            for &m in &fix {
                current_cube.apply_move(m);
            }
        }
        final_moves.extend(fix);
    }
    final_moves
}

pub fn is_opposite_face(f1: Face, f2: Face) -> bool {
    matches!(
        (f1, f2),
        (Face::Up, Face::Down)
            | (Face::Down, Face::Up)
            | (Face::Front, Face::Back)
            | (Face::Back, Face::Front)
            | (Face::Right, Face::Left)
            | (Face::Left, Face::Right)
    )
}

pub fn get_buffer_face(f1: Face, f2: Face) -> Face {
    for &f in &[
        Face::Up,
        Face::Down,
        Face::Front,
        Face::Back,
        Face::Right,
        Face::Left,
    ] {
        if !is_opposite_face(f1, f) && !is_opposite_face(f2, f) && f != f1 && f != f2 {
            return f;
        }
    }
    Face::Up
}

fn get_fix_180(face: Face) -> Vec<Move> {
    let rot = get_setup_to_up(face);
    let mut moves = rot.clone();
    // (U R L U2 R' L' U R L U2 R' L') is a verfied color-preserving 180-rot for center.
    // In our tests: (U R L U2 Rp Lp) * 2 worked.
    let seq = vec![
        Move::U,
        Move::R,
        Move::L,
        Move::U2,
        Move::Rp,
        Move::Lp,
        Move::U,
        Move::R,
        Move::L,
        Move::U2,
        Move::Rp,
        Move::Lp,
    ];
    moves.extend(seq);
    moves.extend(undo_setup(rot));
    moves
}

fn get_fix_90_pair(f_cw: Face, f_ccw: Face) -> Vec<Move> {
    // Verified color-preserving 90-degree pair rotation:
    // Mp E M U Mp Ep M Up (Up CCW, Right CW)
    // To solve f_cw=1 and f_ccw=3, we apply 3 to f_cw and 1 to f_ccw.
    // So setup f_cw -> Up, f_ccw -> Right.
    let rot = get_setup_to_up_right(f_cw, f_ccw);
    let mut moves = rot.clone();
    let seq = vec![
        Move::Mp,
        Move::E,
        Move::M,
        Move::U,
        Move::Mp,
        Move::Ep,
        Move::M,
        Move::Up,
    ];
    moves.extend(seq);
    moves.extend(undo_setup(rot));
    moves
}

pub fn get_setup_to_up(face: Face) -> Vec<Move> {
    for rot in get_all_rotations() {
        // その回転で Piece originally at Y=1 がどこに動くかを探す
        // （実際には Face SLOT がどこに映るかを知りたい）
        // SLOT "face" を SLOT "Up" に持ってくる回転を探す。
        let result_face = apply_rot_to_face(face, &rot);
        if result_face == Face::Up {
            return rot;
        }
    }
    vec![]
}

pub fn get_setup_to_up_right(f_up: Face, f_right: Face) -> Vec<Move> {
    for rot in get_all_rotations() {
        if apply_rot_to_face(f_up, &rot) == Face::Up
            && apply_rot_to_face(f_right, &rot) == Face::Right
        {
            return rot;
        }
    }
    vec![]
}

pub fn apply_rot_to_face(face: Face, rot: &[Move]) -> Face {
    let mut normal = match face {
        Face::Up => Vec3::Y,
        Face::Down => -Vec3::Y,
        Face::Left => -Vec3::X,
        Face::Right => Vec3::X,
        Face::Front => Vec3::Z,
        Face::Back => -Vec3::Z,
    };
    for &m in rot {
        let (axis, _, angle) = move_to_geometric_params_for_rot(m);
        let mat = glam::Mat4::from_axis_angle(axis, angle);
        normal = mat.transform_vector3(normal);
    }
    // 法線に最も近い Face を返す
    Face::all()
        .iter()
        .copied()
        .find(|&f| {
            let fnorm = match f {
                Face::Up => Vec3::Y,
                Face::Down => -Vec3::Y,
                Face::Left => -Vec3::X,
                Face::Right => Vec3::X,
                Face::Front => Vec3::Z,
                Face::Back => -Vec3::Z,
            };
            (normal - fnorm).length() < 0.1
        })
        .unwrap_or(Face::Up)
}

fn move_to_geometric_params_for_rot(mv: Move) -> (Vec3, i8, f32) {
    let pi_2 = std::f32::consts::FRAC_PI_2;
    match mv {
        Move::X => (Vec3::X, 0, -pi_2),
        Move::Xp => (Vec3::X, 0, pi_2),
        Move::X2 => (Vec3::X, 0, std::f32::consts::PI),
        Move::Y => (Vec3::Y, 0, -pi_2),
        Move::Yp => (Vec3::Y, 0, pi_2),
        Move::Y2 => (Vec3::Y, 0, std::f32::consts::PI),
        Move::Z => (Vec3::Z, 0, -pi_2),
        Move::Zp => (Vec3::Z, 0, pi_2),
        Move::Z2 => (Vec3::Z, 0, std::f32::consts::PI),
        _ => (Vec3::Y, 0, 0.0),
    }
}

#[cfg(any(target_arch = "wasm32", test))]
impl SolverState {
    pub fn new(start_cube: &Cube, max_depth: usize, ignore_orientation: bool) -> Self {
        let rc = RawCube::from_cube(start_cube);
        Self {
            raw_cube: rc,
            initial_cube: start_cube.clone(),
            max_depth,
            ignore_orientation,
            solution: None,
            finished: false,
        }
    }
    pub fn error(&self) -> Option<String> {
        match &self.raw_cube {
            Ok(_) => None,
            Err(e) => Some(e.clone()),
        }
    }
    pub fn process_chunk(&mut self, _: usize) -> (usize, bool) {
        if self.finished {
            return (0, true);
        }
        let result = solve_internal(
            &self.initial_cube,
            self.max_depth,
            self.ignore_orientation,
            None,
        );
        self.solution = Some(result);
        self.finished = true;
        (1, true)
    }
    pub fn get_solution(&self) -> Option<Solution> {
        self.solution.clone()
    }
    pub fn estimate_progress(&self) -> f32 {
        if self.finished {
            1.0
        } else {
            0.5
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cube::Color;

    // ==================== SolverState Tests ====================

    #[test]
    fn test_solver_state_new_valid() {
        let cube = Cube::new();
        let state = SolverState::new(&cube, 20, false);
        assert!(state.error().is_none());
    }

    #[test]
    fn test_solver_state_new_scrambled() {
        let mut cube = Cube::new();
        cube.apply_move(Move::U);
        cube.apply_move(Move::R);
        cube.apply_move(Move::F);

        let state = SolverState::new(&cube, 30, false);
        assert!(state.error().is_none());
    }

    #[test]
    fn test_solver_state_process_chunk_solved() {
        let cube = Cube::new();
        let mut state = SolverState::new(&cube, 20, false);

        let (progress, finished) = state.process_chunk(10);
        assert!(finished);
        assert_eq!(progress, 1);

        let solution = state.get_solution();
        assert!(solution.is_some());
        if let Some(sol) = solution {
            assert!(sol.found);
            assert_eq!(sol.moves.len(), 0);
        }
    }

    #[test]
    fn test_solver_state_process_chunk_scrambled() {
        let mut cube = Cube::new();
        cube.apply_move(Move::U);
        cube.apply_move(Move::R);
        cube.apply_move(Move::F);

        let mut state = SolverState::new(&cube, 30, false);

        let (progress, finished) = state.process_chunk(100);
        assert!(finished);
        assert_eq!(progress, 1);

        let solution = state.get_solution();
        assert!(solution.is_some());
    }

    #[test]
    fn test_solver_state_estimate_progress_before() {
        let cube = Cube::new();
        let state = SolverState::new(&cube, 20, false);

        let progress = state.estimate_progress();
        assert_eq!(progress, 0.5);
    }

    #[test]
    fn test_solver_state_estimate_progress_after() {
        let cube = Cube::new();
        let mut state = SolverState::new(&cube, 20, false);

        state.process_chunk(10);
        let progress = state.estimate_progress();
        assert_eq!(progress, 1.0);
    }

    #[test]
    fn test_solver_state_multiple_calls() {
        let mut cube = Cube::new();
        cube.apply_move(Move::U);

        let mut state = SolverState::new(&cube, 20, false);

        let (_, finished1) = state.process_chunk(10);
        assert!(finished1);

        let (progress2, finished2) = state.process_chunk(10);
        assert!(finished2);
        assert_eq!(progress2, 0);
    }

    #[test]
    fn test_solver_state_get_solution_before() {
        let cube = Cube::new();
        let state = SolverState::new(&cube, 20, false);

        let solution = state.get_solution();
        assert!(solution.is_none());
    }

    #[test]
    fn test_solver_state_ignore_orientation() {
        let mut cube = Cube::new();
        cube.apply_move(Move::U);
        cube.apply_move(Move::Ep);

        let mut state = SolverState::new(&cube, 30, true);

        state.process_chunk(100);
        let solution = state.get_solution();
        assert!(solution.is_some());
    }

    #[test]
    fn test_solver_state_complex_scramble() {
        let mut cube = Cube::new();
        cube.scramble(10);

        let mut state = SolverState::new(&cube, 50, false);

        let (_, finished) = state.process_chunk(1000);
        assert!(finished);

        let solution = state.get_solution();
        assert!(solution.is_some());
    }

    // ==================== Special Conditions Tests ====================

    #[test]
    fn test_solve_color_solved_ignore_orientation() {
        let mut cube = Cube::new();
        cube.stickers[Face::Up.start_index() + 4].orientation = 1;

        let solution = solve(&cube, 20, true);
        assert!(solution.found);
        assert!(solution.message.contains("色が揃っています") || solution.message.contains("解決"));
    }

    #[test]
    fn test_debug_log_random_success() {
        std::env::set_var("SOLVER_DEBUG", "1");

        let mut cube = Cube::new();
        cube.apply_move(Move::U);
        cube.apply_move(Move::R);
        cube.apply_move(Move::F);

        let _ = solve(&cube, 30, false);

        std::env::remove_var("SOLVER_DEBUG");
    }

    #[test]
    fn test_debug_log_no_solution() {
        std::env::set_var("SOLVER_DEBUG", "1");

        let mut cube = Cube::new();
        for _ in 0..25 {
            cube.apply_move(Move::U);
            cube.apply_move(Move::R);
            cube.apply_move(Move::F);
        }

        let solution = solve(&cube, 1, false);
        assert!(!solution.found);

        std::env::remove_var("SOLVER_DEBUG");
    }

    #[test]
    fn test_error_message_depth_exceeded() {
        let mut cube = Cube::new();
        for _ in 0..30 {
            cube.apply_move(Move::U);
            cube.apply_move(Move::R);
            cube.apply_move(Move::F);
        }

        let solution = solve(&cube, 1, false);
        assert!(!solution.found);
        assert!(
            solution.message.contains("解が見つかりません")
                || solution.message.contains("探索深度")
        );
    }

    #[test]
    fn test_solver_state_error_coverage() {
        std::env::set_var("SOLVER_DEBUG", "1");
        let mut cube = Cube::new();
        // Break cube distribution
        cube.stickers[0].color = Color::Yellow;
        let state = SolverState::new(&cube, 20, false);
        assert!(state.error().is_some());
        let _ = state.estimate_progress();
        let _ = state.get_solution();
        std::env::remove_var("SOLVER_DEBUG");
    }

    #[test]
    fn test_solver_state_process_chunk_after_finished() {
        let cube = Cube::new();
        let mut state = SolverState::new(&cube, 20, false);
        state.process_chunk(1);
        assert!(state.finished);
        let (progress, finished) = state.process_chunk(1);
        assert_eq!(progress, 0);
        assert!(finished);
    }
}
