use crate::cube::{Cube, Move};
use rustc_hash::FxHashMap;
use std::collections::VecDeque;
use std::sync::mpsc::Sender;
use std::sync::OnceLock;

/// デフォルトの最大探索深度
pub const DEFAULT_MAX_DEPTH: usize = 11;
const ESTIMATED_STATES_MAX: usize = 1_000_000;

/// BFS探索で使用する状態マップ: 状態 → その状態に到達した操作（根はNone）
type StateMap = FxHashMap<Cube, Option<Move>>;

/// BFS探索で使用する状態キュー
type StateQueue = VecDeque<Cube>;

/// ソルバーの結果
#[derive(Debug, Clone)]
pub struct Solution {
    pub moves: Vec<Move>,
    pub found: bool,
}

static SOLVED_STATES: OnceLock<Vec<Cube>> = OnceLock::new();

/// 全24通りの向きの完成状態を取得（キャッシュ）
pub fn get_solved_states() -> &'static [Cube] {
    SOLVED_STATES.get_or_init(generate_all_solved_states)
}

fn generate_all_solved_states() -> Vec<Cube> {
    use rustc_hash::FxHashSet;
    let base = Cube::new();
    let mut states = Vec::new();
    let mut queue = VecDeque::new();
    let mut visited: FxHashSet<Cube> = FxHashSet::default();

    let base_norm = base.normalized();
    queue.push_back(base.clone());
    visited.insert(base_norm);
    states.push(base);

    let rotations = vec![
        vec![Move::U, Move::Dp],
        vec![Move::R, Move::Lp],
        vec![Move::F, Move::Bp],
    ];

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
    // 方位によらず標準パターン [1, 2, 0, 3] にリセットする。
    // (rotation.rs が物理的に整合したため、これら24状態はすべて解決状態として有効)
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
    solve_internal(start_cube, max_depth, ignore_orientation, None)
}

fn solve_internal(
    start_cube: &Cube,
    max_depth: usize,
    ignore_orientation: bool,
    progress_tx: Option<Sender<f32>>,
) -> Solution {
    println!(
        "高速化{}BFS探索開始: 最大深度={}",
        if ignore_orientation {
            "(向き無視) "
        } else {
            ""
        },
        max_depth
    );

    let is_goal = if ignore_orientation {
        start_cube.is_solved()
    } else {
        is_fully_solved(start_cube)
    };

    if is_goal {
        return Solution {
            moves: vec![],
            found: true,
        };
    }

    let all_moves = Move::all_moves();

    let forward_depth = max_depth.div_ceil(2);
    #[allow(clippy::cast_possible_truncation)]
    let estimated_states = all_moves
        .len()
        .pow(forward_depth as u32)
        .min(ESTIMATED_STATES_MAX);

    // 順方向と逆方向の探索データを定義
    let mut forward_queue = VecDeque::with_capacity(estimated_states);
    let mut forward_dist = FxHashMap::with_capacity_and_hasher(estimated_states, rustc_hash::FxBuildHasher);

    let mut backward_queue = VecDeque::with_capacity(estimated_states);
    let mut backward_dist = FxHashMap::with_capacity_and_hasher(estimated_states, rustc_hash::FxBuildHasher);

    // 初期状態のセット
    let start_key = if ignore_orientation {
        start_cube.normalized()
    } else {
        start_cube.clone()
    };
    forward_queue.push_back(start_key.clone());
    forward_dist.insert(start_key, None);

    for solved in get_solved_states() {
        let s_key = if ignore_orientation {
            solved.normalized()
        } else {
            solved.clone()
        };
        if !backward_dist.contains_key(&s_key) {
            backward_dist.insert(s_key.clone(), None);
            backward_queue.push_back(s_key);
        }
    }

    let mut forward_depth = 0;
    let mut backward_depth = 0;
    let total_depth = max_depth;

    while forward_depth + backward_depth < total_depth {
        if forward_queue.is_empty() || backward_queue.is_empty() {
            break;
        }

        // 進捗報告
        if let Some(ref tx) = progress_tx {
            #[allow(clippy::cast_precision_loss)]
            let progress = (forward_depth + backward_depth) as f32 / total_depth as f32;
            let _ = tx.send(progress);
        }

        // どちらを展開するか選択（サイズが小さい方）
        let expand_forward = if forward_queue.is_empty() {
            false
        } else if backward_queue.is_empty() {
            true
        } else {
            forward_queue.len() <= backward_queue.len()
        };

        if expand_forward {
            let next_entries = expand_layer_single(
                &mut forward_queue,
                &forward_dist,
                &all_moves,
                ignore_orientation,
            );

            for (next_key, val) in next_entries {
                if !forward_dist.contains_key(&next_key) {
                    forward_dist.insert(next_key.clone(), val);
                    
                    // 衝突判定
                    if backward_dist.contains_key(&next_key) {
                        if let Some(ref tx) = progress_tx {
                            let _ = tx.send(1.0);
                        }
                        let mut moves = reconstruct_path_forward(&forward_dist, &next_key, ignore_orientation);
                        let rev_moves = reconstruct_path_backward(&backward_dist, &next_key, ignore_orientation);
                        moves.extend(rev_moves);
                        return Solution { moves, found: true };
                    }
                    
                    forward_queue.push_back(next_key);
                }
            }
            forward_depth += 1;
        } else {
            let next_entries = expand_layer_single(
                &mut backward_queue,
                &backward_dist,
                &all_moves,
                ignore_orientation,
            );

            for (next_key, val) in next_entries {
                if !backward_dist.contains_key(&next_key) {
                    backward_dist.insert(next_key.clone(), val);
                    
                    // 衝突判定
                    if forward_dist.contains_key(&next_key) {
                        if let Some(ref tx) = progress_tx {
                            let _ = tx.send(1.0);
                        }
                        let mut moves = reconstruct_path_forward(&forward_dist, &next_key, ignore_orientation);
                        let rev_moves = reconstruct_path_backward(&backward_dist, &next_key, ignore_orientation);
                        moves.extend(rev_moves);
                        return Solution { moves, found: true };
                    }
                    
                    backward_queue.push_back(next_key);
                }
            }
            backward_depth += 1;
        }
    }

    if let Some(ref tx) = progress_tx {
        let _ = tx.send(1.0);
    }

    Solution {
        moves: vec![],
        found: false,
    }
}

/// BFSの一つの層を展開します。
fn expand_layer_single(
    queue: &mut StateQueue,
    dist: &StateMap,
    all_moves: &[Move],
    ignore_orientation: bool,
) -> Vec<(Cube, Option<Move>)> {
    use rayon::prelude::*;

    let current_nodes: Vec<Cube> = queue.drain(..).collect();

    // スレッド起動のオーバーヘッドを避けるため、ノード数が少ない場合はシングルスレッドで処理
    const PARALLEL_THRESHOLD: usize = 1000;
    if current_nodes.len() < PARALLEL_THRESHOLD {
        current_nodes
            .iter()
            .flat_map(|curr| {
                let last_mv = dist.get(curr).and_then(|&m| m);
                all_moves.iter().filter_map(move |&mv| {
                    if let Some(last) = last_mv {
                        if last == mv.inverse() {
                            return None;
                        }
                    }

                    let mut next = curr.clone();
                    next.apply_move(mv);
                    let next_key = if ignore_orientation {
                        next.normalized()
                    } else {
                        next
                    };

                    Some((next_key, Some(mv)))
                })
            })
            .collect()
    } else {
        current_nodes
            .par_iter()
            .flat_map_iter(|curr| {
                let last_mv = dist.get(curr).and_then(|&m| m);
                all_moves.iter().filter_map(move |&mv| {
                    if let Some(last) = last_mv {
                        if last == mv.inverse() {
                            return None;
                        }
                    }

                    let mut next = curr.clone();
                    next.apply_move(mv);
                    let next_key = if ignore_orientation {
                        next.normalized()
                    } else {
                        next
                    };

                    Some((next_key, Some(mv)))
                })
            })
            .collect()
    }
}

fn reconstruct_path_forward(dist: &StateMap, target: &Cube, ignore_orientation: bool) -> Vec<Move> {
    let mut path = Vec::new();
    let mut curr = target.clone();
    while let Some(&maybe_mv) = dist.get(&curr) {
        if let Some(mv) = maybe_mv {
            path.push(mv);
            let mut parent = curr;
            parent.apply_move(mv.inverse());
            curr = if ignore_orientation {
                parent.normalized()
            } else {
                parent
            };
        } else {
            break;
        }
    }
    path.reverse();
    path
}

fn reconstruct_path_backward(
    dist: &StateMap,
    target: &Cube,
    ignore_orientation: bool,
) -> Vec<Move> {
    let mut path = Vec::new();
    let mut curr = target.clone();
    while let Some(&maybe_mv) = dist.get(&curr) {
        if let Some(mv) = maybe_mv {
            path.push(mv.inverse());
            let mut parent = curr;
            parent.apply_move(mv.inverse());
            curr = if ignore_orientation {
                parent.normalized()
            } else {
                parent
            };
        } else {
            break;
        }
    }
    path
}
