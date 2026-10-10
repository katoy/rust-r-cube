use super::coord::{move_cube_18, Corner, RawCube};
use super::cube;
use super::lbl::cancel_redundant_moves_with_phases;
use crate::PhaseInfo;
use std::collections::{HashSet, VecDeque};

/// Ortega法による 2x2x2 キューブ解法。
/// スピードキュービングで最も人気の高い3ステップ解法:
/// 1. 最初の1面 (First Face): 側面を揃えず、1面（下面）の色のみを素早く揃える
/// 2. 上面色揃え (OLL): 反対面（上面）の色を揃える (7パターンの公式)
/// 3. 両層同時配置 (PBL): 1層目と2層目の位置を1つのアルゴリズムで同時に解決 (PBL公式)
pub fn solve_ortega(rc: &RawCube) -> Result<(Vec<usize>, Vec<PhaseInfo>), String> {
    let mut total_moves = Vec::new();
    let mut phases = Vec::new();
    let mut current = *rc;

    // --- ステップ 1: 最初の1面 (First Face) ---
    let face1_moves = solve_first_face(&current)?;
    let f1_start = total_moves.len();
    current = cube::apply(&current, &face1_moves);
    total_moves.extend(&face1_moves);
    let f1_end = total_moves.len();
    phases.push(PhaseInfo {
        name: "ステップ 1: 最初の1面 (First Face)".to_string(),
        start: f1_start,
        end: f1_end,
    });

    // --- ステップ 2: 上面色揃え (OLL) ---
    let (oll_moves, oll_name) = solve_ortega_oll(&current)?;
    let oll_start = total_moves.len();
    current = cube::apply(&current, &oll_moves);
    total_moves.extend(&oll_moves);
    let oll_end = total_moves.len();
    phases.push(PhaseInfo {
        name: format!("ステップ 2: 反対面色揃え (OLL: {})", oll_name),
        start: oll_start,
        end: oll_end,
    });

    // --- ステップ 3: 両層同時配置 (PBL) ---
    let (pbl_moves, pbl_name) = solve_pbl(&current)?;
    let pbl_start = total_moves.len();
    total_moves.extend(&pbl_moves);
    let pbl_end = total_moves.len();
    phases.push(PhaseInfo {
        name: format!("ステップ 3: 両層同時配置 (PBL: {})", pbl_name),
        start: pbl_start,
        end: pbl_end,
    });

    // 冗長手のキャンセル
    let (optimized_moves, optimized_phases) =
        cancel_redundant_moves_with_phases(total_moves, phases);

    // 検証
    let final_cube = cube::apply(rc, &optimized_moves);
    if final_cube != RawCube::default() {
        return Err("Ortega法の解法検証に失敗しました。".to_string());
    }

    Ok((optimized_moves, optimized_phases))
}

/// 下面（D面）にある4つのコーナー（スロット4..8）がすべてD面向き（co == 0）かつD面ピースかを判定
#[inline]
pub fn is_first_face_solved(cube: &RawCube) -> bool {
    let d_pieces = [Corner::DFR, Corner::DLF, Corner::DBL, Corner::DRB];
    for slot in 4..8 {
        if cube.co[slot] != 0 || !d_pieces.contains(&cube.cp[slot]) {
            return false;
        }
    }
    true
}

/// 幅優先探索 (BFS) で最初の1面の色を揃える手順を探索 (深さ 0〜4 程度で即座に発見)
fn solve_first_face(cube: &RawCube) -> Result<Vec<usize>, String> {
    if is_first_face_solved(cube) {
        return Ok(Vec::new());
    }

    let mut q = VecDeque::new();
    let mut visited = HashSet::new();

    q.push_back((*cube, Vec::new()));
    visited.insert(cube.hash_first_face());

    while let Some((cur, moves)) = q.pop_front() {
        if moves.len() >= 6 {
            break;
        }

        let last_face = moves.last().map(|m| m / 3).unwrap_or(255);

        for face in 0..6 {
            if face == last_face {
                continue;
            }
            if (face == 3 && last_face == 0)
                || (face == 4 && last_face == 1)
                || (face == 5 && last_face == 2)
            {
                continue;
            }

            for turn in 0..3 {
                let m = face * 3 + turn;
                let next_cube = cur.multiply(move_cube_18(m));

                if is_first_face_solved(&next_cube) {
                    let mut res = moves.clone();
                    res.push(m);
                    return Ok(res);
                }

                if moves.len() + 1 < 5 {
                    let key = next_cube.hash_first_face();
                    if visited.insert(key) {
                        let mut next_moves = moves.clone();
                        next_moves.push(m);
                        q.push_back((next_cube, next_moves));
                    }
                }
            }
        }
    }

    Err("最初の1面の手順が見つかりませんでした。".into())
}

impl RawCube {
    #[inline]
    fn hash_first_face(&self) -> u64 {
        let mut h = 0u64;
        for i in 4..8 {
            h = (h << 3) | (self.cp[i] as u64);
            h = (h << 2) | (self.co[i] as u64);
        }
        h
    }
}

// --- ステップ 2: OLL ---

const ORTEGA_OLL_ALGS: &[(&str, &str)] = &[
    ("Sune (スーネ)", "R U R' U R U2 R'"),
    ("Anti-Sune (アンチスーネ)", "R U2 R' U' R U' R'"),
    ("H型 (4角反転)", "R2 U2 R U2 R2"),
    ("Pi型 (側面前後)", "F R U R' U' R U R' U' F'"),
    ("T型 (2角外向き)", "R U R' U' R' F R F'"),
    ("U型 (2角前向き)", "F R U R' U' F'"),
    ("L型 (ボウタイ)", "F R' F' R U R U' R'"),
];

fn solve_ortega_oll(cube: &RawCube) -> Result<(Vec<usize>, String), String> {
    if is_ortega_oll_solved(cube) {
        return Ok((Vec::new(), "完成 (スキップ)".to_string()));
    }

    let auf_choices: &[&[usize]] = &[&[], &[0], &[1], &[2]];

    for auf in auf_choices {
        let cube_after_auf = cube::apply(cube, auf);

        for (name, alg_str) in ORTEGA_OLL_ALGS {
            let alg_moves = cube::parse_moves(alg_str).unwrap();
            let result_cube = cube::apply(&cube_after_auf, &alg_moves);

            if is_ortega_oll_solved(&result_cube) && is_first_face_solved(&result_cube) {
                let mut combined = auf.to_vec();
                combined.extend(alg_moves);
                return Ok((combined, name.to_string()));
            }
        }
    }

    Err("Ortega OLL の適用パターンが見つかりませんでした。".into())
}

#[inline]
fn is_ortega_oll_solved(cube: &RawCube) -> bool {
    cube.co[0] == 0 && cube.co[1] == 0 && cube.co[2] == 0 && cube.co[3] == 0
}

// --- ステップ 3: PBL (Permutation of Both Layers) ---

const PBL_ALGORITHMS: &[(&str, &str)] = &[
    ("スキップ", ""),
    ("上下対角 (Double Diag)", "R2 F2 R2"),
    ("上下隣接 (Double Adj)", "R2 U' B2 U2 R2 U' R2"),
    (
        "上隣接・下揃い (J-perm)",
        "R U R' F' R U R' U' R' F R2 U' R'",
    ),
    (
        "上対角・下揃い (Y-perm)",
        "F R U' R' U' R U R' F' R U R' U' R' F R F'",
    ),
    ("上揃い・下隣接", "R D R' B' R D R' D' R' B R2 D' R'"),
    (
        "上揃い・下対角",
        "B R D' R' D' R D R' B' R D R' D' R' B R B'",
    ),
    ("上隣接・下対角", "R U' R F2 R' U R'"),
    ("上対角・下隣接", "R' D R' F2 R D' R"),
];

#[inline]
fn rotate_y(m: usize, times: usize) -> usize {
    let mut face = m / 3;
    let turn = m % 3;
    for _ in 0..times {
        face = match face {
            1 => 5,         // R -> B
            5 => 4,         // B -> L
            4 => 2,         // L -> F
            2 => 1,         // F -> R
            other => other, // U, D unchanged
        };
    }
    face * 3 + turn
}

fn solve_pbl(cube: &RawCube) -> Result<(Vec<usize>, String), String> {
    let u_aufs: &[&[usize]] = &[&[], &[0], &[1], &[2]];
    let d_aufs: &[&[usize]] = &[&[], &[9], &[10], &[11]]; // 9: D, 10: D2, 11: D'

    for pre_u in u_aufs {
        for pre_d in d_aufs {
            let mut pre_moves = pre_u.to_vec();
            pre_moves.extend_from_slice(pre_d);
            let cube_after_pre = cube::apply(cube, &pre_moves);

            for (name, alg_str) in PBL_ALGORITHMS {
                let base_moves = if alg_str.is_empty() {
                    Vec::new()
                } else {
                    cube::parse_moves(alg_str).unwrap()
                };

                for y_rot in 0..4 {
                    let alg_moves: Vec<usize> =
                        base_moves.iter().map(|&m| rotate_y(m, y_rot)).collect();
                    let cube_after_alg = cube::apply(&cube_after_pre, &alg_moves);

                    for post_u in u_aufs {
                        for post_d in d_aufs {
                            let mut post_moves = post_u.to_vec();
                            post_moves.extend_from_slice(post_d);
                            let final_cube = cube::apply(&cube_after_alg, &post_moves);

                            if final_cube == RawCube::default() {
                                let mut combined = pre_moves;
                                combined.extend(alg_moves);
                                combined.extend(post_moves);
                                return Ok((combined, name.to_string()));
                            }
                        }
                    }
                }
            }
        }
    }

    Err("PBL の適用パターンが見つかりませんでした。".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ortega_on_solved_cube() {
        let rc = RawCube::default();
        let (moves, _phases) = solve_ortega(&rc).expect("solved cube should be solvable by Ortega");
        assert!(moves.is_empty());
        assert_eq!(cube::apply(&rc, &moves), RawCube::default());
    }

    #[test]
    fn test_ortega_on_scrambled_cube() {
        for seed in 1..=100 {
            let sc = cube::scramble(seed);
            let rc = cube::apply(&RawCube::default(), &sc);
            let (moves, phases) = solve_ortega(&rc)
                .unwrap_or_else(|e| panic!("seed {} should be solvable: {}", seed, e));
            assert_eq!(cube::apply(&rc, &moves), RawCube::default());
            assert!(
                phases.len() <= 3 && !phases.is_empty(),
                "phases count={}",
                phases.len()
            );
        }
    }

    #[test]
    fn test_pbl_algorithms_preserve_layer_orientation() {
        for (name, alg) in PBL_ALGORITHMS {
            if alg.is_empty() {
                continue;
            }
            let moves = cube::parse_moves(alg).unwrap();
            let c = cube::apply(&RawCube::default(), &moves);
            assert_eq!(
                c.co, [0; 8],
                "PBL algorithm {} must preserve corner orientations",
                name
            );
        }
    }
}
