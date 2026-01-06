use crate::cube::{Cube, Move};
use rustc_hash::FxHashMap;
use std::collections::VecDeque;
use std::sync::OnceLock;

/// ソルバーの結果
#[derive(Debug, Clone)]
pub struct Solution {
    pub moves: Vec<Move>,
    pub found: bool,
}

static SOLVED_STATES: OnceLock<Vec<Cube>> = OnceLock::new();

/// 全24通りの向きの完成状態を取得（キャッシュ）
fn get_solved_states() -> &'static [Cube] {
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
    states
}

/// キューブが（向きも含めて）完全に解けているか判定
pub fn is_fully_solved(cube: &Cube) -> bool {
    get_solved_states().contains(cube)
}

/// 双方向BFSを使用して最短解を探索
pub fn solve(start_cube: &Cube, max_depth: usize, ignore_orientation: bool) -> Solution {
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

    // --- 順方向探索 ---
    // Cube -> (至った手, 親の状態)
    let mut forward_dist: FxHashMap<Cube, (Move, Option<Cube>)> = FxHashMap::default();
    let mut forward_queue: VecDeque<Cube> = VecDeque::new();

    let start_key = if ignore_orientation {
        start_cube.normalized()
    } else {
        start_cube.clone()
    };
    forward_queue.push_back(start_key.clone());
    forward_dist.insert(start_key.clone(), (Move::R, None)); // marker

    // 順方向BFS
    let mut current_depth = 0;
    while current_depth < forward_depth {
        let level_size = forward_queue.len();
        if level_size == 0 {
            break;
        }

        for _ in 0..level_size {
            let curr = forward_queue.pop_front().unwrap();

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

                if !forward_dist.contains_key(&next_key) {
                    forward_dist.insert(next_key.clone(), (mv, Some(curr.clone())));
                    forward_queue.push_back(next_key);
                }
            }
        }
        current_depth += 1;
    }

    // --- 逆方向探索 ---
    let mut backward_queue: VecDeque<Cube> = VecDeque::new();
    let mut backward_map: FxHashMap<Cube, (Move, Option<Cube>)> = FxHashMap::default();

    for solved in get_solved_states() {
        let s_key = if ignore_orientation {
            solved.normalized()
        } else {
            solved.clone()
        };
        if !backward_map.contains_key(&s_key) {
            if forward_dist.contains_key(&s_key) {
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
        for _ in 0..level_size {
            let curr = backward_queue.pop_front().unwrap();

            // 衝突判定
            if forward_dist.contains_key(&curr) {
                let mut moves = reconstruct_path_forward(&forward_dist, &curr);
                let rev_moves = reconstruct_path_backward(&backward_map, &curr);
                moves.extend(rev_moves);
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

                if !backward_map.contains_key(&next_key) {
                    backward_map.insert(next_key.clone(), (mv, Some(curr.clone())));
                    backward_queue.push_back(next_key);
                }
            }
        }
        current_depth += 1;
    }

    Solution {
        moves: vec![],
        found: false,
    }
}

fn reconstruct_path_forward(
    dist: &FxHashMap<Cube, (Move, Option<Cube>)>,
    target: &Cube,
) -> Vec<Move> {
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

fn reconstruct_path_backward(
    dist: &FxHashMap<Cube, (Move, Option<Cube>)>,
    target: &Cube,
) -> Vec<Move> {
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
