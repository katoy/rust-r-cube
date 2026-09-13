use crate::coord::{move_cube_18, Corner, Edge, RawCube};

pub const SOLVED: &str = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
pub const FACES: &[u8; 6] = b"URFDLB";
pub const CORNERS: [[usize; 3]; 8] = [
    [8, 9, 20],
    [6, 18, 38],
    [0, 36, 47],
    [2, 45, 11],
    [29, 26, 15],
    [27, 44, 24],
    [33, 53, 42],
    [35, 17, 51],
];
pub const EDGES: [[usize; 2]; 12] = [
    [5, 10],
    [7, 19],
    [3, 37],
    [1, 46],
    [32, 16],
    [28, 25],
    [30, 43],
    [34, 52],
    [23, 12],
    [21, 41],
    [50, 39],
    [48, 14],
];

pub fn parse_state(text: &str) -> Result<RawCube, String> {
    let f = text.as_bytes();
    if f.len() != 54 {
        return Err("54マスすべての色を入力してください。".into());
    }
    for (i, color) in FACES.iter().enumerate() {
        if f.iter().filter(|v| *v == color).count() != 9 {
            return Err(format!(
                "{} 面の色は9枚必要です。色の残数を確認してください。",
                *color as char
            ));
        }
        if f[i * 9 + 4] != *color {
            return Err(
                "センターの色は変更できません。上が白、前が緑になるように持ってください。".into(),
            );
        }
    }
    let solved = RawCube::default();
    let mut cube = solved;
    let mut seen_c = [false; 8];
    for (slot, indices) in CORNERS.iter().enumerate() {
        let mut found = false;
        for (piece, home) in CORNERS.iter().enumerate() {
            for orientation in 0..3 {
                if (0..3).all(|n| f[indices[(n + orientation) % 3]] == FACES[home[n] / 9]) {
                    if seen_c[piece] {
                        return Err("同じコーナーピースが複数あります。".into());
                    }
                    seen_c[piece] = true;
                    cube.cp[slot] = solved.cp[piece];
                    cube.co[slot] = orientation as u8;
                    found = true;
                }
            }
        }
        if !found {
            return Err(format!(
                "コーナー {} の色の組み合わせが不正です。隣接する面の向きも確認してください。",
                slot + 1
            ));
        }
    }
    let mut seen_e = [false; 12];
    for (slot, indices) in EDGES.iter().enumerate() {
        let mut found = false;
        for (piece, home) in EDGES.iter().enumerate() {
            for orientation in 0..2 {
                if (0..2).all(|n| f[indices[(n + orientation) % 2]] == FACES[home[n] / 9]) {
                    if seen_e[piece] {
                        return Err("同じエッジピースが複数あります。".into());
                    }
                    seen_e[piece] = true;
                    cube.ep[slot] = solved.ep[piece];
                    cube.eo[slot] = orientation as u8;
                    found = true;
                }
            }
        }
        if !found {
            return Err(format!("エッジ {} の色の組み合わせが不正です。", slot + 1));
        }
    }
    if cube.co.iter().sum::<u8>() % 3 != 0 {
        return Err("コーナーのねじれが不正です。通常の回転では揃えられません。".into());
    }
    if cube.eo.iter().sum::<u8>() % 2 != 0 {
        return Err("エッジの反転が不正です。通常の回転では揃えられません。".into());
    }
    if parity(&cube.cp.map(|v| v as u8)) != parity(&cube.ep.map(|v| v as u8)) {
        return Err(
            "ピースの置換パリティが不正です。2つのピースが入れ替わっていないか確認してください。"
                .into(),
        );
    }
    Ok(cube)
}
pub(crate) fn parity(p: &[u8]) -> usize {
    (0..p.len())
        .map(|i| (i + 1..p.len()).filter(|j| p[i] > p[*j]).count())
        .sum::<usize>()
        % 2
}
pub fn facelets(cube: &RawCube) -> String {
    let mut f = *SOLVED.as_bytes().first_chunk::<54>().unwrap();
    for (slot, indices) in CORNERS.iter().enumerate() {
        for n in 0..3 {
            f[indices[(n + cube.co[slot] as usize) % 3]] =
                FACES[CORNERS[cube.cp[slot] as usize][n] / 9];
        }
    }
    for (slot, indices) in EDGES.iter().enumerate() {
        for n in 0..2 {
            f[indices[(n + cube.eo[slot] as usize) % 2]] =
                FACES[EDGES[cube.ep[slot] as usize][n] / 9];
        }
    }
    String::from_utf8(f.to_vec()).unwrap()
}
pub fn parse_moves(text: &str) -> Result<Vec<usize>, String> {
    if text.len() > 4096 {
        return Err("手順は4096文字以内にしてください。".into());
    }
    text.split_whitespace()
        .map(|token| {
            let bytes = token.as_bytes();
            let face = FACES.iter().position(|v| Some(v) == bytes.first());
            let turns = match bytes.get(1..) {
                Some([]) => Some(0),
                Some(b"2") => Some(1),
                Some(b"'") => Some(2),
                _ => None,
            };
            face.zip(turns).map(|(f, t)| f * 3 + t).ok_or_else(|| {
                format!(
                    "「{token}」は未対応です。U R F D L B と '・2 を使い、空白で区切ってください。"
                )
            })
        })
        .collect()
}
pub fn notation(m: usize) -> String {
    format!("{}{}", FACES[m / 3] as char, ["", "2", "'"][m % 3])
}
pub fn apply(cube: &RawCube, moves: &[usize]) -> RawCube {
    moves
        .iter()
        .fold(*cube, |c, m| c.multiply(move_cube_18(*m)))
}
pub fn scramble(seed: u32) -> Vec<usize> {
    let mut x = seed.max(1);
    let mut moves = Vec::new();
    while moves.len() < 25 {
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        let m = x as usize % 18;
        if moves.last().is_some_and(|last| last / 3 == m / 3) {
            continue;
        }
        moves.push(m);
    }
    moves
}

// Keep enum references explicit for stable serialization-free piece identities.
const _: usize = Corner::UFR as usize + Edge::UR as usize;
