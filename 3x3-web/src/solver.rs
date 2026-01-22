use crate::cube::{Cube, Move};
use crate::kociemba::{RawCube, Search};
use std::sync::mpsc::Sender;
use std::sync::OnceLock;

/// デフォルトの最大探索深度 (Kociemba では通常合計20手程度)
pub const DEFAULT_MAX_DEPTH: usize = 24;

/// ソルバーの結果
#[derive(Debug, Clone)]
pub struct Solution {
    pub moves: Vec<Move>,
    pub found: bool,
}

/// Kociemba ソルバーの状態
#[cfg(any(target_arch = "wasm32", test))]
pub struct SolverState {
    search: Search,
    raw_cube: RawCube,
    solution: Option<Solution>,
    finished: bool,
}

static SOLVED_STATES: OnceLock<Vec<Cube>> = OnceLock::new();

/// 全24通りの向きの完成状態を取得（キャッシュ）
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

    let base_norm = base.normalized();
    queue.push_back(base.clone());
    visited.insert(base_norm);
    states.push(base);

    let rotations = vec![vec![Move::X], vec![Move::Y], vec![Move::Z]];

    while let Some(current) = queue.pop_front() {
        for rot_moves in &rotations {
            let mut next = current.clone();
            for &mv in rot_moves {
                next.apply_move(mv);
            }

            let next_norm = next.normalized();
            if visited.insert(next_norm) {
                states.push(next.clone());
                queue.push_back(next);
            }
        }
    }
    // 回転操作によって得られた物理的な向きを、
    // 方位によらず標準パターン [0; 9] にリセットする。
    states
        .iter()
        .map(Cube::with_clockwise_orientations)
        .collect()
}

/// キューブが（向きも含めて）完全に解けているか判定します。
///
/// 色だけでなく、ステッカーの向き（矢印の方向）も初期状態の24通りの
/// いずれかと一致しているかを確認します。
///
/// # 引数
///
/// - `cube`: 判定するキューブ
///
/// # 戻り値
///
/// - `true`: 完全に解けている（24通りの完成状態のいずれか）
/// - `false`: 解けていない
///
/// # 例
///
/// ```
/// use rubiks_cube_2x2::cube::Cube;
/// use rubiks_cube_2x2::solver::is_fully_solved;
///
/// let cube = Cube::new();
/// assert!(is_fully_solved(&cube));
/// ```
#[must_use]
pub fn is_fully_solved(cube: &Cube) -> bool {
    get_solved_states().contains(cube)
}

/// 双方向BFSを使用して最短解を探索します（進捗送信あり）。
///
/// GUI用の進捗通知機能付きバージョンです。探索の進捗状況を
/// チャネル経由で送信します。
///
/// # 引数
///
/// - `start_cube`: 開始状態のキューブ
/// - `max_depth`: 最大探索深度
/// - `ignore_orientation`: `true` の場合、色のみを考慮（向きは無視）
/// - `progress_tx`: 進捗通知用のSender（Noneの場合は通知なし）
///
/// # 戻り値
///
/// 解法の結果を含む `Solution` 構造体
///
/// # 例
///
/// ```
/// use rubiks_cube_2x2::cube::{Cube, Move};
/// use rubiks_cube_2x2::solver::solve_with_progress;
/// use std::sync::mpsc;
///
/// let mut cube = Cube::new();
/// cube.apply_move(Move::R);
///
/// let (tx, rx) = mpsc::channel();
/// let solution = solve_with_progress(&cube, 11, true, Some(tx));
/// assert!(solution.found);
/// ```
#[must_use]
pub fn solve_with_progress(
    start_cube: &Cube,
    max_depth: usize,
    ignore_orientation: bool,
    progress_tx: Option<Sender<f32>>,
) -> Solution {
    solve_internal(start_cube, max_depth, ignore_orientation, progress_tx)
}

/// 双方向BFSを使用して最短解を探索します。
///
/// キューブの現在の状態から完成状態への最短手順を探索します。
///
/// # 引数
///
/// - `start_cube`: 開始状態のキューブ
/// - `max_depth`: 最大探索深度（デフォルト: 11手）
/// - `ignore_orientation`: `true` の場合、色のみを考慮（向きは無視）
///
/// # 戻り値
///
/// 解法の結果を含む `Solution` 構造体
/// - `found`: 解が見つかったかどうか
/// - `moves`: 解法の手順（見つかった場合）
///
/// # 例
///
/// ```
/// use rubiks_cube_2x2::cube::{Cube, Move};
/// use rubiks_cube_2x2::solver::solve;
///
/// let mut cube = Cube::new();
/// cube.apply_move(Move::R);
/// cube.apply_move(Move::U);
///
/// let solution = solve(&cube, 11, true);
/// assert!(solution.found);
/// println!("解法: {} 手", solution.moves.len());
/// ```
#[must_use]
pub fn solve(start_cube: &Cube, max_depth: usize, ignore_orientation: bool) -> Solution {
    solve_internal(start_cube, max_depth, ignore_orientation, Option::None)
}

fn solve_internal(
    start_cube: &Cube,
    _max_depth: usize,
    _ignore_orientation: bool,
    progress_tx: Option<std::sync::mpsc::Sender<f32>>,
) -> Solution {
    if let Some(ref tx) = progress_tx {
        let _ = tx.send(0.1);
    }

    let rc = match RawCube::from_cube(start_cube) {
        Ok(rc) => rc,
        Err(e) => {
            eprintln!("RawCube::from_cube failed: {}", e);
            return Solution {
                moves: vec![],
                found: false,
            };
        }
    };

    if let Some(ref tx) = progress_tx {
        let _ = tx.send(0.3);
    }

    let mut search = Search::new();
    let moves = search.solve(&rc);

    if let Some(ref tx) = progress_tx {
        let _ = tx.send(1.0);
    }

    match moves {
        Option::Some(m) => Solution {
            moves: m,
            found: true,
        },
        Option::None => Solution {
            moves: vec![],
            found: false,
        },
    }
}

// --- 以下の古いBFS関連ロジックは削除 ---

// WASM環境およびテスト用: インクリメンタルソルバーの実装
#[cfg(any(target_arch = "wasm32", test))]
impl SolverState {
    pub fn new(start_cube: &Cube, _max_depth: usize, _ignore_orientation: bool) -> Self {
        let rc = RawCube::from_cube(start_cube).unwrap_or_default();
        Self {
            search: Search::new(),
            raw_cube: rc,
            solution: None,
            finished: false,
        }
    }

    pub fn process_chunk(&mut self, _max_nodes: usize) -> (usize, bool) {
        if self.finished {
            return (0, true);
        }
        // Kociemba は高速なので、1つのチャンクで一気に解決する（暫定）
        let moves = self.search.solve(&self.raw_cube);
        self.solution = Some(match moves {
            Option::Some(m) => Solution {
                moves: m,
                found: true,
            },
            Option::None => Solution {
                moves: vec![],
                found: false,
            },
        });
        self.finished = true;
        (1, true)
    }

    /// 解を取得
    pub fn get_solution(&self) -> Option<Solution> {
        self.solution.clone()
    }

    /// 進捗の推定（0.0 - 1.0）
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
    use crate::cube::{Cube, Move};

    #[test]
    fn test_solver_state_incremental() {
        let mut cube = Cube::new();
        cube.apply_move(Move::R);
        cube.apply_move(Move::U);

        let mut state = SolverState::new(&cube, 2, true);
        let (_, done) = state.process_chunk(10);

        assert!(done);
        let solution = state.get_solution().expect("Solution should be found");
        assert!(solution.found);
        assert!(!solution.moves.is_empty());
    }

    #[test]
    fn test_solver_state_already_solved() {
        let cube = Cube::new();
        let mut state = SolverState::new(&cube, 11, true);
        let (_, done) = state.process_chunk(1000);
        assert!(done);
        let solution = state.get_solution().unwrap();
        assert!(solution.found);
        assert_eq!(solution.moves.len(), 0);
    }

    #[test]
    fn test_solve_internal_forward_early_exit() {
        // 逆方向の初期化時に、既に順方向と衝突しているケース
        let mut cube = Cube::new();
        cube.apply_move(Move::R);

        // solve_internal は depth 1 で衝突を見つけるはず
        let (tx, rx) = std::sync::mpsc::channel();
        let solution = solve_internal(&cube, 11, true, Some(tx));
        assert!(solution.found);
        assert_eq!(solution.moves.len(), 1);

        // 進捗送信の確認 (1.0 が送られるはず)
        let progress: Vec<f32> = rx.into_iter().collect();
        assert!(progress.contains(&1.0));
    }

    #[test]
    fn test_solve_internal_unsolvable_low_depth() {
        let mut cube = Cube::new();
        // R U は HTM で 2手必要。depth 1 では解けないはず。
        cube.apply_move(Move::R);
        cube.apply_move(Move::U);
        let solution = solve_internal(&cube, 1, true, None);
        assert!(!solution.found);
    }

    #[test]
    fn test_get_solved_states_duplicates() {
        let states = get_solved_states();
        assert_eq!(states.len(), 24);
    }
}
