use crate::cube::parse_moves;

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
    let base_alg =
        "R U R' U' R' F R2 U' R' U' R U R' F' R' L D R F' R' F R F R2 D' R F R F' R' L' R";

    let is_adjacent = |f1: usize, f2: usize| {
        normals[f1][0] * normals[f2][0]
            + normals[f1][1] * normals[f2][1]
            + normals[f1][2] * normals[f2][2]
            == 0
    };

    if is_adjacent(a, b) {
        // Map (U, F) to (u, f)
        // base_alg rotates u by -1, f by +1.
        let (u, f) = (a, b);
        let invert = delta_a == 1;

        let r_vec = [
            normals[u][1] * normals[f][2] - normals[u][2] * normals[f][1],
            normals[u][2] * normals[f][0] - normals[u][0] * normals[f][2],
            normals[u][0] * normals[f][1] - normals[u][1] * normals[f][0],
        ];
        let r = (0..6).find(|&i| normals[i] == r_vec).unwrap();
        // Opposite faces in URFDLB order: U↔D, R↔L, F↔B.
        let opposite = [3, 4, 5, 0, 1, 2];
        let map = [u, r, f, opposite[u], opposite[r], opposite[f]];
        let mv: Vec<usize> = parse_moves(base_alg)
            .unwrap()
            .into_iter()
            .map(|m| map[m / 3] * 3 + m % 3)
            .collect();
        if invert {
            mv.iter().rev().map(|&m| m / 3 * 3 + (2 - m % 3)).collect()
        } else {
            mv
        }
    } else {
        // Opposite faces: find an intermediate face c adjacent to both a and b
        let c = (0..6)
            .find(|&i| is_adjacent(a, i) && is_adjacent(b, i))
            .unwrap();
        let mv1 = rotate_two_centers(a, delta_a, c, -delta_a);
        let mv2 = rotate_two_centers(c, delta_a, b, delta_b);
        [mv1, mv2].concat()
    }
}

pub fn rotate_center_180(face: usize) -> Vec<usize> {
    let names = ["U", "R", "F", "D", "L", "B"];
    let (a, b) = match face {
        0 | 3 => ("R", "L"),
        _ => ("U", "D"),
    };
    let x = names[face % 6];
    let alg = format!("{x} {a} {b} {x}2 {a}' {b}' {x} {a} {b} {x}2 {a}' {b}'");
    parse_moves(&alg).unwrap()
}

fn is_opposite_face(f1: usize, f2: usize) -> bool {
    matches!(
        (f1, f2),
        (0, 3) | (3, 0) | (1, 4) | (4, 1) | (2, 5) | (5, 2)
    )
}

/// Cancel adjacent redundant moves and commutative opposite-face moves (e.g. R R' -> nothing, U D U' -> D)
pub fn cancel_redundant_moves(moves: &[usize]) -> Vec<usize> {
    let mut reduced: Vec<usize> = Vec::new();
    for &m in moves {
        let face = m / 3;
        let mut turns = m % 3 + 1;
        let mut i = reduced.len();
        let mut matched = false;

        while i > 0 {
            let prev_face = reduced[i - 1] / 3;
            if prev_face == face {
                let prev_turns = reduced[i - 1] % 3 + 1;
                turns = (prev_turns + turns) % 4;
                reduced.remove(i - 1);
                if turns != 0 {
                    reduced.insert(i - 1, face * 3 + turns - 1);
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
            reduced.push(face * 3 + turns - 1);
        }
    }
    reduced
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
        let (delta_a, delta_b) = if (cur[a] == 1 && cur[b] == 3) || (cur[a] == 3 && cur[b] == 1) {
            if cur[a] == 1 {
                (1, -1)
            } else {
                (-1, 1)
            }
        } else {
            let da = if cur[a] == 1 { 1 } else { -1 };
            (da, -da)
        };
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

/// Solves remaining center rotations so that all 6 centers have 0 rotation mod 4.
/// `current_rotations`: current net rotation of each face center (0..6) in quarter turns (0..4).
pub fn solve_center_orientations(current_rotations: [i32; 6]) -> Result<Vec<usize>, String> {
    let mut needed = [0i32; 6];
    for f in 0..6 {
        needed[f] = (-current_rotations[f]).rem_euclid(4);
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
        assert!(res1.is_err(), "Single odd center rotation must be rejected");

        let three_odds = [1, 1, 1, 0, 0, 0];
        let res3 = solve_center_orientations(three_odds);
        assert!(res3.is_err(), "Three odd center rotations must be rejected");
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
}
