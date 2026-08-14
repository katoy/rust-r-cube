pub use crate::cube::{Cube, Face, Move};
use crate::kociemba::{RawCube, Search};
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

pub mod fix;
pub use fix::{
    apply_rot_to_face, apply_supercube_fixes, get_buffer_face, get_setup_to_up, is_opposite_face,
};

pub struct SolverState {
    pub raw_cube: Result<RawCube, String>,
    pub initial_cube: Cube,
    pub max_depth: usize,
    pub ignore_orientation: bool,
    pub solution: Option<Solution>,
    pub finished: bool,
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
            let mut next: Cube = current.clone();
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
    // (color_only_solution は色が揃っているが向きが揃っていない状態を保存するために使用される)
    // また、色が揃っているが向きが揃わない状態（パリティエラー）も、ignore_orientation=false の場合は
    // color_only_solution には保存しない。

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

    // 1. 直近方位試行（探索を伴わない純粋な回転と方位修正）
    if let Some(res) = attempt_direct_solve(start_cube, max_depth, ignore_orientation) {
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

    // ignore_orientation が true の場合、色が揃った解があればそれを優先
    if ignore_orientation {
        if let Some(sol) = color_only_solution.clone() {
            progress.report(1.0);
            return sol;
        }
    }

    // 2. 探索的解決 (ランダムなセットアップを用いた探索)
    if let Some(sol) = attempt_search(
        start_cube,
        max_depth,
        ignore_orientation,
        &progress,
        color_only_solution.clone(),
    ) {
        progress.report(1.0);
        return sol;
    }

    // 解決不能な原因の特定とエラーメッセージの返却
    fail_solution(start_cube, &progress)
}

fn attempt_direct_solve(
    cube: &Cube,
    max_depth: usize,
    ignore_orientation: bool,
) -> Option<TrySolveResult> {
    let mut search = Search::new();

    // 色が既に揃っている場合
    if cube.is_solved() {
        if let Some(res) = try_solve_with_rotation(cube, &[], &[], max_depth, false, &mut search) {
            return Some(res);
        }
    }

    // 24通りの回転を試行（Perfect を優先し、見つからない場合は最初の ColorOnly を返す）
    let rotations = get_all_rotations();
    let mut first_color_only = None;
    for rot in &rotations {
        if let Some(res) =
            try_solve_with_rotation(cube, &[], rot, max_depth, ignore_orientation, &mut search)
        {
            match &res {
                TrySolveResult::Perfect(_) => return Some(res),
                TrySolveResult::ColorOnly(_) => {
                    if first_color_only.is_none() {
                        first_color_only = Some(res);
                    }
                }
            }
        }
    }

    first_color_only
}

fn attempt_search(
    start_cube: &Cube,
    max_depth: usize,
    ignore_orientation: bool,
    progress: &ProgressReporter,
    mut color_only_solution: Option<Solution>,
) -> Option<Solution> {
    let mut _search = Search::new();
    let rotations = get_all_rotations();
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
                if trial_iter > 100 {
                    local_search.max_nodes = DEFAULT_MAX_NODES * 5;
                }
                if trial_iter > 500 {
                    local_search.max_nodes = DEFAULT_MAX_NODES * 10;
                }
                if trial_iter > 1000 {
                    local_search.max_nodes = DEFAULT_MAX_NODES * 25;
                }

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
                if trial_iter % 100 == 0 {
                    progress.report(trial_iter as f32 / RANDOM_TRIALS as f32 * PROGRESS_WEIGHT);
                }
                None
            });

        if let Some(sol) = perfect_sol {
            return Some(sol);
        }
        color_only_solution = color_only_mutex.into_inner().unwrap();
    }

    #[cfg(target_arch = "wasm32")]
    {
        let mut seed: usize = RANDOM_SEED;
        let next_rn = |s: &mut usize| -> usize {
            *s = s.wrapping_mul(LCG_MULTIPLIER).wrapping_add(LCG_INCREMENT);
            (*s / 65536) % 32768
        };

        for trial_iter in 0..RANDOM_TRIALS {
            if trial_iter > 100 {
                _search.max_nodes = DEFAULT_MAX_NODES * 5;
            }
            if trial_iter > 500 {
                _search.max_nodes = DEFAULT_MAX_NODES * 10;
            }
            if trial_iter > 1000 {
                _search.max_nodes = DEFAULT_MAX_NODES * 25;
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
                &mut _search,
            ) {
                match res {
                    TrySolveResult::Perfect(sol) => return Some(sol),
                    TrySolveResult::ColorOnly(sol) => {
                        if color_only_solution.is_none() {
                            color_only_solution = Some(sol);
                        }
                    }
                }
            }
        }
    }

    if let Some(sol) = color_only_solution {
        return Some(sol);
    }
    None
}

fn fail_solution(start_cube: &Cube, progress: &ProgressReporter) -> Solution {
    if std::env::var("SOLVER_DEBUG").is_ok() {
        println!("DEBUG: solve_internal: no solution found.");
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
        .map(|&f: &Face| cube.stickers[f.start_index() + 4].orientation)
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

// Functions moved to fix.rs:
// get_target_oris
// apply_supercube_fixes
// is_opposite_face
// get_buffer_face
// get_fix_180
// get_fix_90_pair
// get_setup_to_up
// get_setup_to_up_right
// apply_rot_to_face
// move_to_geometric_params_for_rot

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
