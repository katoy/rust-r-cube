use crate::cube::{Cube, Move};
use rustc_hash::FxHashMap;
use std::collections::VecDeque;
use std::sync::mpsc::Sender;
use std::sync::OnceLock;

/// デフォルトの最大探索深度
pub const DEFAULT_MAX_DEPTH: usize = 11;
const PROGRESS_UPDATE_INTERVAL: usize = 4;

/// BFS探索で使用する状態マップ: 状態 → (到達した手, 親の状態)
type StateMap = FxHashMap<Cube, (Move, Option<Cube>)>;

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
    let base = Cube::new();
    let mut states = Vec::new();
    let mut queue = VecDeque::new();
    let mut visited = FxHashMap::default();

    let base_norm = base.normalized();
    queue.push_back(base.clone());
    visited.insert(base_norm, ());
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
            if let std::collections::hash_map::Entry::Vacant(e) = visited.entry(next_norm) {
                e.insert(());
                states.push(next.clone());
                queue.push_back(next);
            }
        }
    }
    // 全ての完成状態のorientationを時計回りパターンにリセット
    states
        .iter()
        .map(|cube| cube.with_clockwise_orientations())
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
    let backward_depth = max_depth - forward_depth;
    let total_depth = forward_depth + backward_depth;

    // --- 順方向探索 ---
    // 深度dまでの状態数の概算: 12^d (12手 × 深度)
    let estimated_states = 12_usize.pow(forward_depth as u32).min(100_000);
    let mut forward_dist: StateMap =
        FxHashMap::with_capacity_and_hasher(estimated_states, Default::default());
    let mut forward_queue: StateQueue = VecDeque::with_capacity(estimated_states);

    let start_key = if ignore_orientation {
        start_cube.normalized()
    } else {
        start_cube.clone()
    };
    forward_queue.push_back(start_key.clone());
    forward_dist.insert(start_key, (Move::R, None)); // marker

    // 順方向BFS
    let mut current_depth = 0;
    while current_depth < forward_depth {
        let level_size = forward_queue.len();
        if level_size == 0 {
            break;
        }

        // 進捗送信（順方向探索） - 一定間隔で送信
        if let Some(ref tx) = progress_tx {
            if current_depth % PROGRESS_UPDATE_INTERVAL == 0 {
                let progress = (current_depth as f32) / (total_depth as f32);
                let _ = tx.send(progress);
            }
        }

        for _ in 0..level_size {
            let curr = forward_queue
                .pop_front()
                .expect("forward_queue should not be empty during BFS iteration");

            for &mv in &all_moves {
                // 枝刈り：直前の逆操作を回避
                if let Some(&(last_mv, ref parent)) = forward_dist.get(&curr) {
                    if parent.is_some() && last_mv == mv.inverse() {
                        continue;
                    }
                }

                let mut next = curr.clone();
                next.apply_move(mv);
                let next_key = if ignore_orientation {
                    next.normalized()
                } else {
                    next
                };

                use std::collections::hash_map::Entry;
                if let Entry::Vacant(e) = forward_dist.entry(next_key) {
                    let key_clone = e.key().clone();
                    e.insert((mv, Some(curr.clone())));
                    forward_queue.push_back(key_clone);
                }
            }
        }
        current_depth += 1;
    }

    // --- 逆方向探索 ---
    let estimated_backward_states = 12_usize.pow(backward_depth as u32).min(100_000);
    let mut backward_queue: StateQueue = VecDeque::with_capacity(estimated_backward_states);
    let mut backward_map: StateMap =
        FxHashMap::with_capacity_and_hasher(estimated_backward_states, Default::default());

    // 向き無視の場合も向きも揃える場合も、24通りの完成状態すべてを使用
    // ただし、キーの取り方が異なる
    for solved in get_solved_states() {
        let s_key = if ignore_orientation {
            solved.normalized()
        } else {
            solved.clone()
        };
        if !backward_map.contains_key(&s_key) {
            if forward_dist.contains_key(&s_key) {
                if let Some(ref tx) = progress_tx {
                    let _ = tx.send(1.0);
                }
                return Solution {
                    moves: reconstruct_path_forward(&forward_dist, &s_key),
                    found: true,
                };
            }
            backward_map.insert(s_key.clone(), (Move::R, None));
            backward_queue.push_back(s_key);
        }
    }

    let mut current_depth = 0;
    while !backward_queue.is_empty() && current_depth <= backward_depth {
        let level_size = backward_queue.len();

        // 進捗送信（逆方向探索） - 一定間隔で送信
        if let Some(ref tx) = progress_tx {
            if current_depth % PROGRESS_UPDATE_INTERVAL == 0 {
                let progress = (forward_depth + current_depth) as f32 / (total_depth as f32);
                let _ = tx.send(progress);
            }
        }

        for _ in 0..level_size {
            let curr = backward_queue
                .pop_front()
                .expect("backward_queue should not be empty during BFS iteration");

            // 衝突判定
            if forward_dist.contains_key(&curr) {
                let mut moves = reconstruct_path_forward(&forward_dist, &curr);
                let rev_moves = reconstruct_path_backward(&backward_map, &curr);
                moves.extend(rev_moves);
                if let Some(ref tx) = progress_tx {
                    let _ = tx.send(1.0);
                }
                return Solution { moves, found: true };
            }

            if current_depth == backward_depth {
                continue;
            }

            for &mv in &all_moves {
                if let Some(&(last_mv, ref parent)) = backward_map.get(&curr) {
                    if parent.is_some() && last_mv == mv.inverse() {
                        continue;
                    }
                }

                let mut next = curr.clone();
                next.apply_move(mv);
                let next_key = if ignore_orientation {
                    next.normalized()
                } else {
                    next
                };

                use std::collections::hash_map::Entry;
                if let Entry::Vacant(e) = backward_map.entry(next_key) {
                    let key_clone = e.key().clone();
                    e.insert((mv, Some(curr.clone())));
                    backward_queue.push_back(key_clone);
                }
            }
        }
        current_depth += 1;
    }

    if let Some(ref tx) = progress_tx {
        let _ = tx.send(1.0);
    }

    Solution {
        moves: vec![],
        found: false,
    }
}

fn reconstruct_path_forward(dist: &StateMap, target: &Cube) -> Vec<Move> {
    let mut path = Vec::new();
    let mut curr = target;
    while let Some(&(mv, ref parent_opt)) = dist.get(curr) {
        if let Some(ref p) = *parent_opt {
            path.push(mv);
            curr = p;
        } else {
            break;
        }
    }
    path.reverse();
    path
}

fn reconstruct_path_backward(dist: &StateMap, target: &Cube) -> Vec<Move> {
    let mut path = Vec::new();
    let mut curr = target;
    while let Some(&(mv, ref parent_opt)) = dist.get(curr) {
        if let Some(ref p) = *parent_opt {
            path.push(mv.inverse());
            curr = p;
        } else {
            break;
        }
    }
    path
}
