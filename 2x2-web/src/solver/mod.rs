pub mod coord;
pub mod search;
pub mod tables;

#[cfg(test)]
mod coord_tests;

use self::coord::RawCube;
use self::search::Search;
use crate::cube::{Color, Cube, Move};
use std::sync::mpsc::Sender;
use std::sync::OnceLock;

/// デフォルトの最大探索深度
pub const DEFAULT_MAX_DEPTH: usize = 14;

/// ソルバーの結果
#[derive(Debug, Clone)]
pub struct Solution {
    pub moves: Vec<Move>,
    pub found: bool,
}

/// インクリメンタルソルバー用の状態
pub struct SolverState {
    initial_cube: Cube,
    max_depth: usize,
    ignore_orientation: bool,
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

    // 2x2-web には X, Y, Z が無いので、
    // 基本回転 [(U, D'), (R, L'), (F, B')] を組み合わせて 24 通りの向きを生成
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
    states.into_iter().collect()
}

pub fn is_fully_solved(cube: &Cube) -> bool {
    get_solved_states().contains(cube)
}

pub fn solve_with_progress(
    start_cube: &Cube,
    max_depth: usize,
    ignore_orientation: bool,
    progress_tx: Option<Sender<f32>>,
) -> Solution {
    if let Some(ref tx) = progress_tx {
        let _ = tx.send(0.0);
    }
    let res = solve(start_cube, max_depth, ignore_orientation);
    if let Some(ref tx) = progress_tx {
        let _ = tx.send(1.0);
    }
    res
}

pub fn solve(start_cube: &Cube, max_depth: usize, ignore_orientation: bool) -> Solution {
    if ignore_orientation && start_cube.is_solved() {
        return Solution {
            moves: vec![],
            found: true,
        };
    }
    if !ignore_orientation && start_cube.is_solved_with_orientation() {
        return Solution {
            moves: vec![],
            found: true,
        };
    }

    let mut search = Search::new();

    // 24通りの向きを試し、それぞれで探索を行う。
    // 各向きにおいて、キューブの色を標準配色（Up=White, Down=Yellow, ...）に
    // リマップすることで、RawCube の移動法則と整合させる。
    let orientations = if ignore_orientation {
        get_all_orientations()
    } else {
        // 向きを考慮する場合、標準向き（White=Up）のみをターゲットにする
        vec![Orientation {
            rot_moves: vec![],
            face_map: [0, 1, 2, 3, 4, 5],
        }]
    };
    let mut best_moves: Option<Vec<Move>> = None;

    for ori in orientations {
        // 色のリマップ表を作成 (現在の色 -> 標準色)
        // face_map[i] は、現在のスロット i にある面の元の色 (0..5)
        let mut color_map = [Color::Gray; 6];
        for (i, &orig_color_idx) in ori.face_map.iter().enumerate() {
            color_map[orig_color_idx as usize] = Color::from_u8(i as u8);
        }

        // スタートキューブの色をリマップ
        let mut remapped_cube = start_cube.clone();
        for sticker in remapped_cube.stickers.iter_mut() {
            if sticker.color != Color::Gray {
                sticker.color = color_map[sticker.color as usize];
            }
        }

        // リマップ後のキューブは、この向きにおいて「色が揃うと標準配色になる」状態。
        // これを標準向き用の RawCube::from_cube に渡す。
        if let Ok(rc) = RawCube::from_cube(&remapped_cube, &[0, 1, 2, 3, 4, 5]) {
            if let Some(moves) = search.solve(&rc, max_depth) {
                // 見つかった解決手順を元のキューブの向きに翻訳
                let translated_moves: Vec<Move> = moves
                    .into_iter()
                    .map(|m| {
                        let face = (m as usize) / 3;
                        let offset = (m as usize) % 3;
                        let original_face = ori.face_map[face];
                        let all_moves = Move::all_moves();
                        all_moves[original_face as usize * 3 + offset]
                    })
                    .collect();

                let mut check_cube = start_cube.clone();
                for &tm in &translated_moves {
                    check_cube.apply_move(tm);
                }

                if ignore_orientation {
                    if !check_cube.is_solved() {
                        continue;
                    }
                } else {
                    // 向きを一致させる必要がある場合、適用後のキューブが完全な完成状態かチェック
                    if !check_cube.is_solved_with_orientation() {
                        continue;
                    }
                }

                // 特定の目標配色(face_map)に一致する必要がある場合のみ is_fully_solved を使う。
                // 通常は is_solved_with_orientation で十分。

                if best_moves.is_none()
                    || translated_moves.len() < best_moves.as_ref().unwrap().len()
                {
                    best_moves = Some(translated_moves);
                    if best_moves.as_ref().unwrap().is_empty() {
                        break;
                    }
                }
            }
        }
    }

    if let Some(moves) = best_moves {
        return Solution { moves, found: true };
    }

    Solution {
        moves: vec![],
        found: false,
    }
}

struct Orientation {
    #[allow(dead_code)]
    rot_moves: Vec<Move>,
    face_map: [u8; 6], // 現在の [U, D, L, R, F, B] 位置にある元の面インデックス
}

fn get_all_orientations() -> Vec<Orientation> {
    use rustc_hash::FxHashSet;
    use std::collections::VecDeque;

    let mut result = Vec::new();
    let mut visited = FxHashSet::default();
    let mut queue = VecDeque::new();

    let start_cube = Cube::new();
    queue.push_back((start_cube.clone(), Vec::<Move>::new()));
    visited.insert(start_cube);

    let generators = vec![
        vec![Move::U, Move::Dp], // Y
        vec![Move::R, Move::Lp], // X
        vec![Move::F, Move::Bp], // Z
    ];

    while let Some((curr_cube, moves)) = queue.pop_front() {
        // 現在の向きでの face_map を計算
        let mut face_map = [0u8; 6];
        let face_test_indices = [0, 4, 8, 12, 16, 20];
        for (i, &idx) in face_test_indices.iter().enumerate() {
            face_map[i] = curr_cube.stickers[idx].color as u8;
        }

        result.push(Orientation {
            rot_moves: moves.clone(),
            face_map,
        });

        for gen in &generators {
            let mut next_cube = curr_cube.clone();
            for &m in gen {
                next_cube.apply_move(m);
            }

            if visited.insert(next_cube.clone()) {
                let mut next_moves = moves.clone();
                next_moves.extend(gen);
                queue.push_back((next_cube, next_moves));
            }
        }
    }

    result
}

impl SolverState {
    pub fn new(start_cube: &Cube, max_depth: usize, ignore_orientation: bool) -> Self {
        Self {
            initial_cube: start_cube.clone(),
            max_depth,
            ignore_orientation,
            solution: None,
            finished: false,
        }
    }

    pub fn process_chunk(&mut self, _max_nodes: usize) -> (usize, bool) {
        if self.finished {
            return (0, true);
        }
        let sol = solve(&self.initial_cube, self.max_depth, self.ignore_orientation);
        self.solution = Some(sol);
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
impl Cube {
    fn apply_sequence(mut self, seq: &[Move]) -> Self {
        for &m in seq {
            self.apply_move(m);
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cube::{Cube, Move};

    #[test]
    fn test_move_translation_diagnostic() {
        let cube = Cube::new();
        // Rotate Y (U D')
        // Top CW (Seen from top): F->L, L->B, B->R, R->F
        let rot = vec![Move::U, Move::Dp];
        let mut rotated = cube.clone();
        for &m in &rot {
            rotated.apply_move(m);
        }

        // In rotated cube, turn face at pos 3 (Right)
        rotated.apply_move(Move::R);

        // The face at pos 3 in rotated was previously face 5 (Back)
        // Let's verify our get_all_orientations would find this.
        let orientations = get_all_orientations();
        let ori = orientations
            .iter()
            .find(|o| {
                let mut c = Cube::new();
                for &m in &o.rot_moves {
                    c.apply_move(m);
                }
                c == Cube::new().apply_sequence(&rot)
            })
            .expect("Should find Y rotation");

        assert_eq!(ori.face_map[3], 5, "Right pos should be original Back face");

        let translated_m = {
            let face = 3; // Right
            let offset = 0; // CW
            let original_face = ori.face_map[face];
            let all_moves = Move::all_moves();
            all_moves[original_face as usize * 3 + offset]
        };
        assert_eq!(translated_m, Move::B);

        // Verify: original applied with B, then rotated Y, should match rotated
        let mut original = Cube::new();
        original.apply_move(Move::B);
        let mut original_rotated = original.clone();
        for &m in &rot {
            original_rotated.apply_move(m);
        }

        assert_eq!(
            original_rotated, rotated,
            "Translated move B should match rotated move R"
        );
    }
}
