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
    let face_chars = b"URFDLB";
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
        let (u, f, invert) = if delta_a == -1 {
            (a, b, false)
        } else {
            (a, b, true)
        };

        let r_vec = [
            normals[u][1] * normals[f][2] - normals[u][2] * normals[f][1],
            normals[u][2] * normals[f][0] - normals[u][0] * normals[f][2],
            normals[u][0] * normals[f][1] - normals[u][1] * normals[f][0],
        ];
        let r = (0..6).find(|&i| normals[i] == r_vec).unwrap();
        let d = match u {
            0 => 3,
            1 => 4,
            2 => 5,
            3 => 0,
            4 => 1,
            _ => 2,
        };
        let l = match r {
            0 => 3,
            1 => 4,
            2 => 5,
            3 => 0,
            4 => 1,
            _ => 2,
        };
        let b_opp = match f {
            0 => 3,
            1 => 4,
            2 => 5,
            3 => 0,
            4 => 1,
            _ => 2,
        };
        let map = [u, r, f, d, l, b_opp];

        let remapped_moves: String = base_alg
            .split_whitespace()
            .map(|token| {
                let old_face = match token.as_bytes()[0] {
                    b'U' => 0,
                    b'R' => 1,
                    b'F' => 2,
                    b'D' => 3,
                    b'L' => 4,
                    b'B' => 5,
                    _ => 0,
                };
                let new_face = face_chars[map[old_face]] as char;
                let suffix = &token[1..];
                format!("{}{}", new_face, suffix)
            })
            .collect::<Vec<_>>()
            .join(" ");

        let mv = parse_moves(&remapped_moves).unwrap();
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
        _ => ("U", "D"),
    };
    let x = names[face];
    let alg = format!("{x} {a} {b} {x}2 {a}' {b}' {x} {a} {b} {x}2 {a}' {b}'");
    parse_moves(&alg).unwrap()
}

/// Cancel adjacent redundant moves (e.g. R R' -> nothing, U U -> U2)
pub fn cancel_redundant_moves(moves: &[usize]) -> Vec<usize> {
    let mut res: Vec<usize> = Vec::new();
    for &m in moves {
        let f = m / 3;
        let t = match m % 3 {
            0 => 1,
            1 => 2,
            2 => 3,
            _ => 0,
        };
        if let Some(&last) = res.last() {
            let lf = last / 3;
            if lf == f {
                let lt = match last % 3 {
                    0 => 1,
                    1 => 2,
                    2 => 3,
                    _ => 0,
                };
                res.pop();
                let combined = (lt + t) % 4;
                if combined > 0 {
                    let new_m = f * 3
                        + match combined {
                            1 => 0,
                            2 => 1,
                            3 => 2,
                            _ => 0,
                        };
                    res.push(new_m);
                }
                continue;
            }
        }
        res.push(m);
    }
    res
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
