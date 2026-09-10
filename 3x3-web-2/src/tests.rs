use crate::{
    coord::{move_cube_18, RawCube},
    cube::*,
    solve_state,
};

#[test]
fn moves_and_inverse_roundtrip() {
    let c = apply(&RawCube::default(), &scramble(42));
    for face in 0..6 {
        assert_eq!(apply(&c, &[face * 3; 4]), c);
        for turn in 0..3 {
            let m = face * 3 + turn;
            let moved = apply(&c, &[m]);
            assert_eq!(parse_state(&facelets(&moved)).unwrap(), moved);
            assert_eq!(apply(&moved, &[face * 3 + 2 - turn]), c);
        }
    }
}

// Independent spatial rotation checks guard against self-consistent but wrong
// piece tables. Facelets are rotated clockwise as seen from outside each face.
#[test]
fn all_moves_match_geometry() {
    fn geometry(i: usize) -> ([i32; 3], [i32; 3]) {
        let r = (i % 9 / 3) as i32;
        let c = (i % 3) as i32;
        match i / 9 {
            0 => ([c - 1, 1, r - 1], [0, 1, 0]),
            1 => ([1, 1 - r, 1 - c], [1, 0, 0]),
            2 => ([c - 1, 1 - r, 1], [0, 0, 1]),
            3 => ([c - 1, -1, 1 - r], [0, -1, 0]),
            4 => ([-1, 1 - r, c - 1], [-1, 0, 0]),
            _ => ([1 - c, 1 - r, -1], [0, 0, -1]),
        }
    }
    fn rotate(v: [i32; 3], n: [i32; 3]) -> [i32; 3] {
        let dot = (0..3).map(|i| v[i] * n[i]).sum::<i32>();
        let cross = [
            n[1] * v[2] - n[2] * v[1],
            n[2] * v[0] - n[0] * v[2],
            n[0] * v[1] - n[1] * v[0],
        ];
        std::array::from_fn(|i| n[i] * dot - cross[i])
    }
    let c = apply(&RawCube::default(), &scramble(991));
    for face in 0..6 {
        let normal = geometry(face * 9 + 4).1;
        let mut f = facelets(&c).into_bytes();
        for turn in 0..3 {
            let old = f.clone();
            for (i, color) in old.iter().enumerate() {
                let (p, n) = geometry(i);
                if (0..3).map(|a| p[a] * normal[a]).sum::<i32>() == 1 {
                    let target = (rotate(p, normal), rotate(n, normal));
                    let j = (0..54).find(|j| geometry(*j) == target).unwrap();
                    f[j] = *color;
                }
            }
            assert_eq!(
                String::from_utf8(f.clone()).unwrap(),
                facelets(&c.multiply(move_cube_18(face * 3 + turn))),
                "face {face} turn {turn}"
            );
        }
    }
}
#[test]
fn rejects_impossible_states() {
    assert!(parse_state("bad").is_err());
    let mut c = RawCube::default();
    c.eo[0] = 1;
    assert!(parse_state(&facelets(&c)).unwrap_err().contains("反転"));
    let mut c = RawCube::default();
    c.co[0] = 1;
    assert!(parse_state(&facelets(&c)).unwrap_err().contains("ねじれ"));
    let mut c = RawCube::default();
    c.ep.swap(0, 1);
    assert!(parse_state(&facelets(&c)).unwrap_err().contains("パリティ"));
    let mut f = SOLVED.as_bytes().to_vec();
    f[0] = b'R';
    assert!(parse_state(&String::from_utf8(f).unwrap()).is_err());
    assert!(parse_moves("R X nope").is_err());
    assert!(parse_moves("R’").is_err());
}
#[test]
fn solves_known_states() {
    for sequence in [
        "",
        "R",
        "R U R' U'",
        "U R2 F B R B2 R U2 L B2 R U' D' R2 F R' L B2 U2 F2",
    ] {
        let state = facelets(&apply(&RawCube::default(), &parse_moves(sequence).unwrap()));
        let solution = solve_state(&state, 30000).unwrap();
        assert_eq!(solution.state, SOLVED);
        assert_eq!(solution.states.len(), solution.moves.len() + 1);
    }
}
#[test]
fn solves_one_thousand_scrambles() {
    for seed in 1..=1000 {
        let cube = apply(&RawCube::default(), &scramble(seed));
        let state = facelets(&cube);
        assert_eq!(parse_state(&state).unwrap(), cube);
        let solution = solve_state(&state, 5000).unwrap_or_else(|e| panic!("seed {seed}: {e}"));
        assert_eq!(solution.state, SOLVED, "seed {seed}");
    }
}
#[test]
fn deadline_and_table_integrity() {
    let state = facelets(&apply(&RawCube::default(), &scramble(9)));
    assert!(solve_state(&state, 0).is_err());
    let original = crate::TABLE_BYTES.unwrap();
    assert_eq!(crate::tables::encode(), original);
}
