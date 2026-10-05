use std::sync::OnceLock;

fn get_base_alg() -> &'static [usize] {
    static BASE_ALG: OnceLock<Vec<usize>> = OnceLock::new();
    BASE_ALG.get_or_init(|| {
        crate::cube::parse_moves(
            "R U R' U' R' F R2 U' R' U' R U R' F' R' L D R F' R' F R F R2 D' R F R F' R' L' R",
        )
        .unwrap()
    })
}

pub fn rotate_two_centers(a: usize, delta_a: i32, b: usize, delta_b: i32) -> Vec<usize> {
    assert_eq!(delta_a + delta_b, 0);
    assert!(delta_a == 1 || delta_a == -1);
    let normals: [[i32; 3]; 6] = [
        [0, 1, 0],  // U
        [1, 0, 0],  // R
        [0, 0, 1],  // F
        [0, -1, 0], // D
        [-1, 0, 0], // L
        [0, 0, -1], // B
    ];

    let is_adjacent = |f1: usize, f2: usize| {
        normals[f1][0] * normals[f2][0]
            + normals[f1][1] * normals[f2][1]
            + normals[f1][2] * normals[f2][2]
            == 0
    };

    if is_adjacent(a, b) {
        // (U, F) の関係を (u, f) へ写像
        // base_alg は u を -90° (反時計回り)、f を +90° (時計回り) 回転させる基本手順
        let (u, f) = (a, b);
        let invert = delta_a == 1;

        let r_vec = [
            normals[u][1] * normals[f][2] - normals[u][2] * normals[f][1],
            normals[u][2] * normals[f][0] - normals[u][0] * normals[f][2],
            normals[u][0] * normals[f][1] - normals[u][1] * normals[f][0],
        ];
        let r = (0..6).find(|&i| normals[i] == r_vec).unwrap();
        // URFDLB 順の対向面マッピング: U↔D, R↔L, F↔B
        let opposite = [3, 4, 5, 0, 1, 2];
        let map = [u, r, f, opposite[u], opposite[r], opposite[f]];
        let mv: Vec<usize> = get_base_alg()
            .iter()
            .map(|&m| map[m / 3] * 3 + m % 3)
            .collect();
        if invert {
            mv.iter().rev().map(|&m| m / 3 * 3 + (2 - m % 3)).collect()
        } else {
            mv
        }
    } else {
        // 対向面ペアの場合: a と b の両方に隣接する中間面 c を経由して合成
        let c = (0..6)
            .find(|&i| is_adjacent(a, i) && is_adjacent(b, i))
            .unwrap();
        let mv1 = rotate_two_centers(a, delta_a, c, -delta_a);
        let mv2 = rotate_two_centers(c, delta_a, b, delta_b);
        [mv1, mv2].concat()
    }
}

pub fn rotate_center_180(face: usize) -> Vec<usize> {
    let (a_face, b_face) = match face {
        0 | 3 => (1, 4), // R, L
        _ => (0, 3),     // U, D
    };
    let x = face * 3;
    let a = a_face * 3;
    let b = b_face * 3;
    vec![x, a, b, x + 1, a + 2, b + 2, x, a, b, x + 1, a + 2, b + 2]
}

fn is_opposite_face(f1: usize, f2: usize) -> bool {
    matches!(
        (f1, f2),
        (0, 3) | (3, 0) | (1, 4) | (4, 1) | (2, 5) | (5, 2)
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaggedMove {
    pub mv: usize,
    pub phase: usize,
}

fn cancel_tagged_moves_single_pass(moves: &[TaggedMove]) -> Vec<TaggedMove> {
    let mut reduced: Vec<TaggedMove> = Vec::new();
    for &tm in moves {
        let face = tm.mv / 3;
        let mut turns = tm.mv % 3 + 1;
        let mut i = reduced.len();
        let mut matched = false;

        while i > 0 {
            let prev_face = reduced[i - 1].mv / 3;
            if prev_face == face {
                let prev_turns = reduced[i - 1].mv % 3 + 1;
                turns = (prev_turns + turns) % 4;
                let prev_phase = reduced[i - 1].phase;
                reduced.remove(i - 1);
                if turns != 0 {
                    reduced.insert(
                        i - 1,
                        TaggedMove {
                            mv: face * 3 + turns - 1,
                            phase: prev_phase,
                        },
                    );
                }
                matched = true;
                break;
            } else if is_opposite_face(prev_face, face) {
                i -= 1;
            } else {
                break;
            }
        }
        if !matched && turns != 0 {
            reduced.push(tm);
        }
    }
    reduced
}

pub fn cancel_tagged_moves(moves: &[TaggedMove]) -> Vec<TaggedMove> {
    let mut current = moves.to_vec();
    loop {
        let prev_len = current.len();
        current = cancel_tagged_moves_single_pass(&current);
        if current.len() == prev_len {
            break;
        }
    }
    current
}

/// 隣接する冗長回転および可換な対向面の相殺（例: R R' -> 除去、U D U' -> D）。
/// 固定点反復（fixed-point iteration）を用いて、多重にネストしたカスケード相殺を網羅的に解決します。
pub fn cancel_redundant_moves(moves: &[usize]) -> Vec<usize> {
    let tagged: Vec<TaggedMove> = moves
        .iter()
        .map(|&mv| TaggedMove { mv, phase: 0 })
        .collect();
    cancel_tagged_moves(&tagged)
        .into_iter()
        .map(|tm| tm.mv)
        .collect()
}

/// 手順列（3x3 の各手）を適用した後の 6 面のセンター回転角度（0: 0°, 1: 90°, 2: 180°, 3: 270°）を計算します。
pub fn apply_moves_to_centers(mut centers: [i32; 6], moves: &[usize]) -> [i32; 6] {
    for &m in moves {
        let f = m / 3;
        let t = match m % 3 {
            0 => 1,
            1 => 2,
            _ => -1,
        };
        centers[f] = (centers[f] + t).rem_euclid(4);
    }
    centers
}

/// 手順列とフェーズ情報を受け取り、フェーズ境界をまたぐ冗長手や可換対向面を相殺・統合した上で、
/// 各フェーズの開始・終了インデックス（start, end）を整合的に再構築します。
pub fn cancel_redundant_moves_with_phases(
    moves: Vec<usize>,
    phase_infos: Vec<crate::PhaseInfo>,
) -> (Vec<usize>, Vec<crate::PhaseInfo>) {
    if moves.is_empty() || phase_infos.is_empty() {
        return (cancel_redundant_moves(&moves), phase_infos);
    }

    let mut tagged = Vec::with_capacity(moves.len());
    for (i, &mv) in moves.iter().enumerate() {
        let p_idx = phase_infos
            .iter()
            .position(|p| i >= p.start && i < p.end)
            .unwrap_or(phase_infos.len().saturating_sub(1));
        tagged.push(TaggedMove { mv, phase: p_idx });
    }

    let reduced_tagged = cancel_tagged_moves(&tagged);
    let final_moves: Vec<usize> = reduced_tagged.iter().map(|tm| tm.mv).collect();

    let mut new_phase_infos = Vec::new();
    for (p_idx, p_info) in phase_infos.iter().enumerate() {
        let indices: Vec<usize> = reduced_tagged
            .iter()
            .enumerate()
            .filter(|(_, tm)| tm.phase == p_idx)
            .map(|(i, _)| i)
            .collect();

        if let (Some(&first), Some(&last)) = (indices.first(), indices.last()) {
            new_phase_infos.push(crate::PhaseInfo {
                name: p_info.name.clone(),
                start: first,
                end: last + 1,
            });
        }
    }

    (final_moves, new_phase_infos)
}

fn generate_perfect_matchings(faces: &[usize]) -> Vec<Vec<(usize, usize)>> {
    if faces.is_empty() {
        return vec![vec![]];
    }
    let first = faces[0];
    let rest = &faces[1..];
    let mut matchings = Vec::new();
    for (i, &partner) in rest.iter().enumerate() {
        let mut remaining = Vec::with_capacity(rest.len() - 1);
        remaining.extend_from_slice(&rest[..i]);
        remaining.extend_from_slice(&rest[i + 1..]);
        for mut sub in generate_perfect_matchings(&remaining) {
            sub.push((first, partner));
            matchings.push(sub);
        }
    }
    matchings
}

fn evaluate_matching(mut cur: [i32; 6], matching: &[(usize, usize)]) -> Vec<usize> {
    let mut moves = Vec::new();
    for &(a, b) in matching {
        let da = if cur[a] == 1 { 1 } else { -1 };
        let (delta_a, delta_b) = (da, -da);
        moves.extend(rotate_two_centers(a, delta_a, b, delta_b));
        cur[a] = (cur[a] - delta_a).rem_euclid(4);
        cur[b] = (cur[b] - delta_b).rem_euclid(4);
    }
    for (f, &rot) in cur.iter().enumerate() {
        if rot == 2 {
            moves.extend(rotate_center_180(f));
        }
    }
    cancel_redundant_moves(&moves)
}

/// 6面すべてのセンター回転が mod 4 で 0 になるように残余センター向きを解決します。
/// `current_rotations`: 各面センターの正味回転角（0..6面、90°単位 0..4）。
pub fn solve_center_orientations(current_rotations: [i32; 6]) -> Result<Vec<usize>, String> {
    let mut needed = [0i32; 6];
    for f in 0..6 {
        needed[f] = (4 - current_rotations[f].rem_euclid(4)) % 4;
    }

    if needed.iter().all(|&x| x == 0) {
        return Ok(Vec::new());
    }

    let odd_faces: Vec<usize> = (0..6).filter(|&f| needed[f] % 2 != 0).collect();

    if !odd_faces.len().is_multiple_of(2) {
        return Err("センター向きのパリティが不正です（奇数回転のセンター数が奇数です）".into());
    }

    let matchings = generate_perfect_matchings(&odd_faces);
    let mut best_moves: Option<Vec<usize>> = None;

    for matching in &matchings {
        let moves = evaluate_matching(needed, matching);
        if best_moves.as_ref().is_none_or(|b| moves.len() < b.len()) {
            best_moves = Some(moves);
        }
    }
    Ok(best_moves.unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfect_matching_optimality() {
        let current = [3, 3, 1, 1, 0, 0];
        let solution = solve_center_orientations(current).unwrap();
        assert!(!solution.is_empty());
        assert!(solution.len() <= 62);
    }

    #[test]
    fn test_odd_center_rotations_parity_rejected() {
        let single_odd = [1, 0, 0, 0, 0, 0];
        let res1 = solve_center_orientations(single_odd);
        assert!(
            res1.is_err(),
            "奇数個の奇数回転センターはパリティ不正として拒否されること"
        );

        let three_odds = [1, 1, 1, 0, 0, 0];
        let res3 = solve_center_orientations(three_odds);
        assert!(
            res3.is_err(),
            "奇数個の奇数回転センターはパリティ不正として拒否されること"
        );
    }

    #[test]
    fn test_rotate_center_180_invariance() {
        use crate::coord::RawCube;
        use crate::cube::apply;

        let solved = RawCube::default();
        for f in 0..6 {
            let moves = rotate_center_180(f);
            assert_eq!(moves.len(), 12, "単独180度回転は12手であること");
            let res = apply(&solved, &moves);
            assert_eq!(res, solved, "キューブ状態は完成のまま維持されること");

            let mut centers = [0i32; 6];
            for &m in &moves {
                let face = m / 3;
                let turns = match m % 3 {
                    0 => 1,
                    1 => 2,
                    _ => -1,
                };
                centers[face] = (centers[face] + turns).rem_euclid(4);
            }
            let mut expected = [0i32; 6];
            expected[f] = 2;
            assert_eq!(centers, expected, "対象面のみが180度回転すること");
        }
    }

    #[test]
    fn test_cancel_redundant_moves_cascading() {
        use crate::cube::parse_moves;

        // ネストされた相殺: R U D D' U' R' -> 空
        let moves = parse_moves("R U D D' U' R'").unwrap();
        let canceled = cancel_redundant_moves(&moves);
        assert!(canceled.is_empty(), "カスケード相殺で空配列になること");

        // 対向面を跨いだ多重相殺: U R L L' R' U' -> 空
        let moves2 = parse_moves("U R L L' R' U'").unwrap();
        let canceled2 = cancel_redundant_moves(&moves2);
        assert!(
            canceled2.is_empty(),
            "対向面を跨いだ多重相殺で空配列になること"
        );

        // 3面連鎖の相殺: F U R R' U' F' -> 空
        let moves3 = parse_moves("F U R R' U' F'").unwrap();
        let canceled3 = cancel_redundant_moves(&moves3);
        assert!(canceled3.is_empty(), "3面連鎖の相殺で空配列になること");
    }

    #[test]
    fn test_cancel_redundant_moves_with_phases() {
        use crate::cube::parse_moves;

        // 1. 接合部相殺: [R, U] + [U', D] -> [R, D]
        let moves = parse_moves("R U U' D").unwrap();
        let phases = vec![
            crate::PhaseInfo {
                name: "Phase 1".to_string(),
                start: 0,
                end: 2,
            },
            crate::PhaseInfo {
                name: "Phase 2".to_string(),
                start: 2,
                end: 4,
            },
        ];
        let (opt_moves, opt_phases) = cancel_redundant_moves_with_phases(moves, phases);
        let expected = parse_moves("R D").unwrap();
        assert_eq!(opt_moves, expected);
        assert_eq!(opt_phases.len(), 2);
        assert_eq!(opt_phases[0].start, 0);
        assert_eq!(opt_phases[0].end, 1);
        assert_eq!(opt_phases[1].start, 1);
        assert_eq!(opt_phases[1].end, 2);

        // 2. 接合部合算: [R, U] + [U, D] -> [R, U2, D]
        let moves2 = parse_moves("R U U D").unwrap();
        let phases2 = vec![
            crate::PhaseInfo {
                name: "Phase 1".to_string(),
                start: 0,
                end: 2,
            },
            crate::PhaseInfo {
                name: "Phase 2".to_string(),
                start: 2,
                end: 4,
            },
        ];
        let (opt_moves2, opt_phases2) = cancel_redundant_moves_with_phases(moves2, phases2);
        let expected2 = parse_moves("R U2 D").unwrap();
        assert_eq!(opt_moves2, expected2);
        assert_eq!(opt_phases2.len(), 2);
        assert_eq!(opt_phases2[0].start, 0);
        assert_eq!(opt_phases2[0].end, 2);
        assert_eq!(opt_phases2[1].start, 2);
        assert_eq!(opt_phases2[1].end, 3);
    }
}
