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
        0 => ("R", "L"),
        1 => ("U", "D"),
        2 => ("U", "D"),
        3 => ("R", "L"),
        4 => ("U", "D"),
        5 => ("U", "D"),
        _ => unreachable!("face index must be 0..5, got {face}"),
    };
    let x = names[face];
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

/// Solves remaining center rotations so that all 6 centers have 0 rotation mod 4.
/// `current_rotations`: current net rotation of each face center (0..6) in quarter turns (0..4).
pub fn solve_center_orientations(current_rotations: [i32; 6]) -> Vec<usize> {
    let mut needed = [0i32; 6];
    for f in 0..6 {
        needed[f] = (-current_rotations[f]).rem_euclid(4);
    }

    if needed.iter().all(|&x| x == 0) {
        return Vec::new();
    }

    let mut cur = needed;
    let mut moves: Vec<usize> = Vec::new();

    // Step 1: Pair up faces with odd rotations (1 or 3)
    while let Some(a) = (0..6).find(|&f| cur[f] % 2 != 0) {
        let Some(b) = (0..6).find(|&f| f != a && cur[f] % 2 != 0) else {
            break; // Odd count cannot be fully paired if parity was invalid
        };
        let delta_a = if cur[a] == 1 { 1 } else { -1 };
        let delta_b = -delta_a;
        let fix = rotate_two_centers(a, delta_a, b, delta_b);
        moves.extend(fix);
        cur[a] = (cur[a] - delta_a).rem_euclid(4);
        cur[b] = (cur[b] - delta_b).rem_euclid(4);
    }

    // Step 2: Fix any remaining 180° rotations (cur[f] == 2)
    for (f, rot) in cur.iter_mut().enumerate() {
        if *rot == 2 {
            let fix = rotate_center_180(f);
            moves.extend(fix);
            *rot = 0;
        }
    }

    cancel_redundant_moves(&moves)
}
