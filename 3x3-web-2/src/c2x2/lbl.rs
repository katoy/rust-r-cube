use super::coord::{get_group_b_idx, move_cube_18, Corner, RawCube};
use super::cube;
use super::tables::PruningTable;
use crate::PhaseInfo;
use std::sync::OnceLock;

/// LBL法 (Layer-by-Layer) による 2x2x2 キューブ解法。
/// 3つの明確な教育的ステップでキューブを解きます:
/// 1. 完全1層 (First Layer): 下面（白面）および側面の4コーナーを完全に揃える
/// 2. 上面色揃え (OLL): 上面（黄色）の向きを揃える (7パターンの公式)
/// 3. 上面位置揃え (PLL): 上面コーナーの位置を揃えて完全完成 (T-perm / Y-perm)
pub fn solve_lbl(rc: &RawCube) -> Result<(Vec<usize>, Vec<PhaseInfo>), String> {
    let mut total_moves = Vec::new();
    let mut phases = Vec::new();
    let mut current = *rc;

    // --- ステップ 1: 完全1層 (First Layer) ---
    let layer1_moves = solve_first_layer(&current)?;
    let l1_start = total_moves.len();
    current = cube::apply(&current, &layer1_moves);
    total_moves.extend(&layer1_moves);
    let l1_end = total_moves.len();
    phases.push(PhaseInfo {
        name: "ステップ 1: 完全1層 (First Layer)".to_string(),
        start: l1_start,
        end: l1_end,
    });

    // --- ステップ 2: 上面色揃え (OLL) ---
    let (oll_moves, oll_name) = solve_oll(&current)?;
    let oll_start = total_moves.len();
    current = cube::apply(&current, &oll_moves);
    total_moves.extend(&oll_moves);
    let oll_end = total_moves.len();
    phases.push(PhaseInfo {
        name: format!("ステップ 2: 上面色揃え (OLL: {})", oll_name),
        start: oll_start,
        end: oll_end,
    });

    // --- ステップ 3: 上面位置揃え (PLL) ---
    let (pll_moves, pll_name) = solve_pll(&current)?;
    let pll_start = total_moves.len();
    total_moves.extend(&pll_moves);
    let pll_end = total_moves.len();
    phases.push(PhaseInfo {
        name: format!("ステップ 3: 上面位置揃え (PLL: {})", pll_name),
        start: pll_start,
        end: pll_end,
    });

    // 冗長手のキャンセル（ステップ境界での不要な U U' 等を整理）
    let (optimized_moves, optimized_phases) =
        cancel_redundant_moves_with_phases(total_moves, phases);

    // 検証
    let final_cube = cube::apply(rc, &optimized_moves);
    if final_cube != RawCube::default() {
        return Err("LBL法の解法検証に失敗しました。".to_string());
    }

    Ok((optimized_moves, optimized_phases))
}

/// 下面4つのコーナー（4, 5, 6, 7）が位置・向きともに完全に揃っているかを判定
#[inline]
pub fn is_first_layer_solved(cube: &RawCube) -> bool {
    cube.cp[4] == Corner::DFR
        && cube.co[4] == 0
        && cube.cp[5] == Corner::DLF
        && cube.co[5] == 0
        && cube.cp[6] == Corner::DBL
        && cube.co[6] == 0
        && cube.cp[7] == Corner::DRB
        && cube.co[7] == 0
}

/// Group B 距離テーブル（下面4コーナーの厳密最短距離）に沿って第1層をO(d)で解決
fn solve_first_layer(cube: &RawCube) -> Result<Vec<usize>, String> {
    if is_first_layer_solved(cube) {
        return Ok(Vec::new());
    }

    let group_b = PruningTable::group_b();
    let mut cur = *cube;
    let mut dist = group_b[get_group_b_idx(&cur)];
    let mut moves = Vec::with_capacity(dist as usize);
    let mut last_face = 255usize;

    while dist > 0 {
        let mut next_step = None;
        'search: for face in 0..6 {
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
                let candidate = cur.multiply(move_cube_18(m));
                if group_b[get_group_b_idx(&candidate)] == dist - 1 {
                    next_step = Some((m, face, candidate));
                    break 'search;
                }
            }
        }
        let Some((m, face, next_cube)) = next_step else {
            return Err("完全1層の手順が見つかりませんでした。".into());
        };
        moves.push(m);
        cur = next_cube;
        last_face = face;
        dist -= 1;
    }

    if !is_first_layer_solved(&cur) {
        return Err("完全1層の手順が見つかりませんでした。".into());
    }

    Ok(moves)
}

// --- ステップ 2: OLL (Orientation of Last Layer) ---

/// 上面4つのコーナーの向き（co[0..4]）がすべて 0（黄色が上面）かを判定
#[inline]
pub fn is_oll_solved(cube: &RawCube) -> bool {
    cube.co[0] == 0 && cube.co[1] == 0 && cube.co[2] == 0 && cube.co[3] == 0
}

struct OllAlgorithm {
    name: &'static str,
    moves: &'static str,
}

const OLL_ALGORITHMS: &[OllAlgorithm] = &[
    OllAlgorithm {
        name: "Sune (スーネ)",
        moves: "R U R' U R U2 R'",
    },
    OllAlgorithm {
        name: "Anti-Sune (アンチスーネ)",
        moves: "R U2 R' U' R U' R'",
    },
    OllAlgorithm {
        name: "H型 (4角反転)",
        moves: "R2 U2 R U2 R2",
    },
    OllAlgorithm {
        name: "Pi型 (側面前後)",
        moves: "F R U R' U' R U R' U' F'",
    },
    OllAlgorithm {
        name: "T型 (2角外向き)",
        moves: "R U R' U' R' F R F'",
    },
    OllAlgorithm {
        name: "U型 (2角前向き)",
        moves: "F R U R' U' F'",
    },
    OllAlgorithm {
        name: "L型 (ボウタイ)",
        moves: "F R' F' R U R U' R'",
    },
];

pub(super) fn parsed_oll_algorithms() -> &'static [(&'static str, Vec<usize>)] {
    static PARSED: OnceLock<Vec<(&'static str, Vec<usize>)>> = OnceLock::new();
    PARSED
        .get_or_init(|| {
            OLL_ALGORITHMS
                .iter()
                .map(|alg| (alg.name, cube::parse_moves(alg.moves).unwrap()))
                .collect()
        })
        .as_slice()
}

fn solve_oll(cube: &RawCube) -> Result<(Vec<usize>, String), String> {
    if is_oll_solved(cube) {
        return Ok((Vec::new(), "完成 (スキップ)".to_string()));
    }

    // 事前 AUF (U回転: 0:なし, 0:U, 1:U2, 2:U')
    let auf_moves: &[&[usize]] = &[
        &[],
        &[0], // U
        &[1], // U2
        &[2], // U'
    ];

    for auf in auf_moves {
        let cube_after_auf = cube::apply(cube, auf);

        for (name, alg_moves) in parsed_oll_algorithms() {
            let result_cube = cube::apply(&cube_after_auf, alg_moves);

            // 上面向きが揃い、かつ第1層が維持されていること
            if is_oll_solved(&result_cube) && is_first_layer_solved(&result_cube) {
                let mut combined = Vec::with_capacity(auf.len() + alg_moves.len());
                combined.extend_from_slice(auf);
                combined.extend_from_slice(alg_moves);
                return Ok((combined, (*name).to_string()));
            }
        }
    }

    Err("OLL の適用パターンが見つかりませんでした。".into())
}

// --- ステップ 3: PLL (Permutation of Last Layer) ---

struct PllAlgorithm {
    name: &'static str,
    moves: &'static str,
}

const PLL_ALGORITHMS: &[PllAlgorithm] = &[
    PllAlgorithm {
        name: "スキップ (完成)",
        moves: "",
    },
    PllAlgorithm {
        name: "隣接交換 (T-perm)",
        moves: "R U R' U' R' F R2 U' R' U' R U R' F'",
    },
    PllAlgorithm {
        name: "対角交換 (Y-perm)",
        moves: "F R U' R' U' R U R' F' R U R' U' R' F R F'",
    },
];

fn parsed_pll_algorithms() -> &'static [(&'static str, Vec<usize>)] {
    static PARSED: OnceLock<Vec<(&'static str, Vec<usize>)>> = OnceLock::new();
    PARSED
        .get_or_init(|| {
            PLL_ALGORITHMS
                .iter()
                .map(|alg| {
                    let moves = if alg.moves.is_empty() {
                        Vec::new()
                    } else {
                        cube::parse_moves(alg.moves).unwrap()
                    };
                    (alg.name, moves)
                })
                .collect()
        })
        .as_slice()
}

fn solve_pll(cube: &RawCube) -> Result<(Vec<usize>, String), String> {
    let auf_choices: &[&[usize]] = &[
        &[],
        &[0], // U
        &[1], // U2
        &[2], // U'
    ];

    for pre_auf in auf_choices {
        let cube_after_pre = cube::apply(cube, pre_auf);

        for (name, alg_moves) in parsed_pll_algorithms() {
            let cube_after_alg = cube::apply(&cube_after_pre, alg_moves);

            for post_auf in auf_choices {
                let final_cube = cube::apply(&cube_after_alg, post_auf);

                if final_cube == RawCube::default() {
                    let mut combined =
                        Vec::with_capacity(pre_auf.len() + alg_moves.len() + post_auf.len());
                    combined.extend_from_slice(pre_auf);
                    combined.extend_from_slice(alg_moves);
                    combined.extend_from_slice(post_auf);
                    return Ok((combined, (*name).to_string()));
                }
            }
        }
    }

    Err("PLL の適用パターンが見つかりませんでした。".into())
}

/// 連続する同一面の回転を結合・キャンセルし、フェーズ情報の境界インデックスを補正します。
pub fn cancel_redundant_moves_with_phases(
    moves: Vec<usize>,
    phases: Vec<PhaseInfo>,
) -> (Vec<usize>, Vec<PhaseInfo>) {
    crate::supercube::cancel_redundant_moves_with_phases(moves, phases)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lbl_on_solved_cube() {
        let rc = RawCube::default();
        let (moves, _phases) = solve_lbl(&rc).expect("solved cube should be solvable");
        assert!(moves.is_empty());
        assert_eq!(cube::apply(&rc, &moves), RawCube::default());
    }

    #[test]
    fn test_lbl_on_scrambled_cube() {
        for seed in [42, 100, 2024, 777, 9999] {
            let sc = cube::scramble(seed);
            let rc = cube::apply(&RawCube::default(), &sc);
            let (moves, phases) = solve_lbl(&rc).expect("scrambled cube should be solvable by LBL");
            assert_eq!(cube::apply(&rc, &moves), RawCube::default());
            assert!(
                phases.len() <= 3 && !phases.is_empty(),
                "phases count={}",
                phases.len()
            );
        }
    }
}
