use super::coord::{move_cube_18, Corner, RawCube};
use super::cube;
use super::lbl::{cancel_redundant_moves_with_phases, parsed_oll_algorithms};
use crate::PhaseInfo;
use std::collections::VecDeque;
use std::sync::OnceLock;

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

const FIRST_FACE_GOAL: u16 = 0x5500;

/// 最初の1面判定用の 16bit キーを計算する。
/// 各スロット i (0..8) について 2bit を割り当て、Uピース=0、Dピース=(1 + co[i]) を格納する。
/// スロット 4..7 がすべて D ピースかつ向き 0 のときに FIRST_FACE_GOAL (0x5500) と一致する。
#[inline]
fn first_face_key(cube: &RawCube) -> u16 {
    let mut key = 0u16;
    for i in 0..8 {
        if (cube.cp[i] as u8) >= 4 {
            key |= ((cube.co[i] as u16) + 1) << (i * 2);
        }
    }
    key
}

#[inline]
fn apply_move_to_first_face_key(key: u16, m: usize) -> u16 {
    let mv = move_cube_18(m);
    let mut next_key = 0u16;
    for i in 0..8 {
        let src = mv.cp[i] as usize;
        let code = (key >> (src * 2)) & 3;
        if code != 0 {
            let new_co = (code as u8 - 1 + mv.co[i]) % 3;
            next_key |= ((new_co as u16) + 1) << (i * 2);
        }
    }
    next_key
}

fn first_face_table() -> &'static [u8] {
    static TABLE: OnceLock<Box<[u8]>> = OnceLock::new();
    TABLE
        .get_or_init(|| {
            let mut table = vec![u8::MAX; 65536].into_boxed_slice();
            let mut q = VecDeque::with_capacity(5670);
            table[FIRST_FACE_GOAL as usize] = 0;
            q.push_back(FIRST_FACE_GOAL);
            while let Some(key) = q.pop_front() {
                let next_dist = table[key as usize] + 1;
                for m in 0..18 {
                    let next_key = apply_move_to_first_face_key(key, m);
                    if table[next_key as usize] == u8::MAX {
                        table[next_key as usize] = next_dist;
                        q.push_back(next_key);
                    }
                }
            }
            table
        })
        .as_ref()
}

/// 事前計算済みの距離テーブルに沿って最初の1面の色を O(d) で解決
fn solve_first_face(cube: &RawCube) -> Result<Vec<usize>, String> {
    if is_first_face_solved(cube) {
        return Ok(Vec::new());
    }

    let table = first_face_table();
    let mut key = first_face_key(cube);
    let mut dist = table[key as usize];
    if dist == u8::MAX {
        return Err("最初の1面の手順が見つかりませんでした。".into());
    }

    let mut moves = Vec::with_capacity(dist as usize);
    let mut last_face = usize::MAX;
    while dist > 0 {
        let mut found = None;
        'step: for face in 0..6 {
            // U/D 等の可換な手を含む最短経路を塞がないよう、同一面連続のみスキップ
            if face == last_face {
                continue;
            }
            for turn in 0..3 {
                let m = face * 3 + turn;
                let next_key = apply_move_to_first_face_key(key, m);
                if table[next_key as usize] == dist - 1 {
                    found = Some((m, face, next_key));
                    break 'step;
                }
            }
        }
        let Some((m, face, next_key)) = found else {
            return Err("最初の1面の手順が見つかりませんでした。".into());
        };
        moves.push(m);
        key = next_key;
        last_face = face;
        dist -= 1;
    }

    Ok(moves)
}

// --- ステップ 2: OLL ---

fn solve_ortega_oll(cube: &RawCube) -> Result<(Vec<usize>, String), String> {
    if is_ortega_oll_solved(cube) {
        return Ok((Vec::new(), "完成 (スキップ)".to_string()));
    }

    let auf_choices: &[&[usize]] = &[&[], &[0], &[1], &[2]];

    for auf in auf_choices {
        let cube_after_auf = cube::apply(cube, auf);

        for (name, alg_moves) in parsed_oll_algorithms() {
            let result_cube = cube::apply(&cube_after_auf, alg_moves);

            if is_ortega_oll_solved(&result_cube) && is_first_face_solved(&result_cube) {
                let mut combined = Vec::with_capacity(auf.len() + alg_moves.len());
                combined.extend_from_slice(auf);
                combined.extend_from_slice(alg_moves);
                return Ok((combined, (*name).to_string()));
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

type ParsedPblEntry = (&'static str, [(Vec<usize>, RawCube); 4]);

fn parsed_pbl_algs() -> &'static [ParsedPblEntry] {
    static PARSED: OnceLock<Vec<ParsedPblEntry>> = OnceLock::new();
    PARSED
        .get_or_init(|| {
            PBL_ALGORITHMS
                .iter()
                .map(|&(name, alg_str)| {
                    let base = if alg_str.is_empty() {
                        Vec::new()
                    } else {
                        cube::parse_moves(alg_str).unwrap()
                    };
                    let rots = std::array::from_fn(|y_rot| {
                        let moves: Vec<usize> = base.iter().map(|&m| rotate_y(m, y_rot)).collect();
                        let rc = cube::apply(&RawCube::default(), &moves);
                        (moves, rc)
                    });
                    (name, rots)
                })
                .collect()
        })
        .as_slice()
}

const U_AUFS: [Option<usize>; 4] = [None, Some(0), Some(1), Some(2)];
const D_AUFS: [Option<usize>; 4] = [None, Some(9), Some(10), Some(11)]; // 9: D, 10: D2, 11: D'
const SOLVED_U_CP: [Corner; 4] = [Corner::UFR, Corner::UFL, Corner::ULB, Corner::UBR];
const SOLVED_D_CP: [Corner; 4] = [Corner::DFR, Corner::DLF, Corner::DBL, Corner::DRB];

#[inline]
fn find_post_u(c: &RawCube) -> Option<Option<usize>> {
    for &mv in &U_AUFS {
        let after = if let Some(m) = mv {
            c.multiply(move_cube_18(m))
        } else {
            *c
        };
        if after.cp[0..4] == SOLVED_U_CP {
            return Some(mv);
        }
    }
    None
}

#[inline]
fn find_post_d(c: &RawCube) -> Option<Option<usize>> {
    for &mv in &D_AUFS {
        let after = if let Some(m) = mv {
            c.multiply(move_cube_18(m))
        } else {
            *c
        };
        if after.cp[4..8] == SOLVED_D_CP {
            return Some(mv);
        }
    }
    None
}

fn solve_pbl(cube: &RawCube) -> Result<(Vec<usize>, String), String> {
    for &pre_u in &U_AUFS {
        let after_u = if let Some(m) = pre_u {
            cube.multiply(move_cube_18(m))
        } else {
            *cube
        };
        for &pre_d in &D_AUFS {
            let after_pre = if let Some(m) = pre_d {
                after_u.multiply(move_cube_18(m))
            } else {
                after_u
            };

            for (name, rots) in parsed_pbl_algs() {
                for (alg_moves, alg_cube) in rots {
                    let after_alg = after_pre.multiply(alg_cube);
                    // U / D の post-AUF はコーナーの向きを変えないため co != 0 は即枝刈りし、U 層と D 層を独立判定
                    if after_alg.co != [0; 8] {
                        continue;
                    }
                    let Some(post_u) = find_post_u(&after_alg) else {
                        continue;
                    };
                    let Some(post_d) = find_post_d(&after_alg) else {
                        continue;
                    };

                    let mut combined = Vec::with_capacity(alg_moves.len() + 4);
                    if let Some(m) = pre_u {
                        combined.push(m);
                    }
                    if let Some(m) = pre_d {
                        combined.push(m);
                    }
                    combined.extend_from_slice(alg_moves);
                    if let Some(m) = post_u {
                        combined.push(m);
                    }
                    if let Some(m) = post_d {
                        combined.push(m);
                    }
                    return Ok((combined, (*name).to_string()));
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
    fn test_first_face_table_reaches_all_5670_states_and_solves_them() {
        let table = first_face_table();
        let reachable = table.iter().filter(|&&d| d != u8::MAX).count();
        assert_eq!(reachable, 5670);

        for seed in 1..=200 {
            let sc = cube::scramble(seed);
            let rc = cube::apply(&RawCube::default(), &sc);
            let moves = solve_first_face(&rc).expect("first face must be solvable");
            let solved = cube::apply(&rc, &moves);
            assert!(
                is_first_face_solved(&solved),
                "seed {seed} failed first face"
            );
            assert_eq!(moves.len(), table[first_face_key(&rc) as usize] as usize);
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
