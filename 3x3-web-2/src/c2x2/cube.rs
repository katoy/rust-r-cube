use super::coord::{move_cube_18, Corner, RawCube};

pub const SOLVED: &str = "UUUURRRRFFFFDDDDLLLLBBBB";
pub const FACES: &[u8; 6] = b"URFDLB";

/// 2x2 の8個のコーナースロットにある3枚のステッカーのインデックス
/// 順序: [U/D面, 他の面1, 他の面2]
/// 0: UFR: U3 (3), R0 (4), F1 (9)
/// 1: UFL: U2 (2), F0 (8), L1 (17)
/// 2: ULB: U0 (0), L0 (16), B1 (21)
/// 3: UBR: U1 (1), B0 (20), R1 (5)
/// 4: DFR: D1 (13), F3 (11), R2 (6)
/// 5: DLF: D0 (12), L3 (19), F2 (10)
/// 6: DBL: D2 (14), B3 (23), L2 (18)
/// 7: DRB: D3 (15), R3 (7), B2 (22)
pub const CORNERS: [[usize; 3]; 8] = [
    [3, 4, 9],    // 0: UFR (U, R, F)
    [2, 8, 17],   // 1: UFL (U, F, L)
    [0, 16, 21],  // 2: ULB (U, L, B)
    [1, 20, 5],   // 3: UBR (U, B, R)
    [13, 11, 6],  // 4: DFR (D, F, R)
    [12, 19, 10], // 5: DLF (D, L, F)
    [14, 23, 18], // 6: DBL (D, B, L)
    [15, 7, 22],  // 7: DRB (D, R, B)
];

/// 24文字の展開図文字列（U, R, F, D, L, B）を解析し、`RawCube` 構造体に変換します。
/// 各面の色枚数（各4枚）、各コーナーピースの整合性、およびねじれ（Twist）パリティを厳密に検証します。
pub fn parse_state(text: &str) -> Result<RawCube, String> {
    let f = text.as_bytes();
    if f.len() != 24 {
        return Err("24マスすべての色を入力してください。".into());
    }

    for color in FACES.iter() {
        let count = f.iter().filter(|v| *v == color).count();
        if count != 4 {
            return Err(format!(
                "{} 面の色は4枚必要です（現在: {}枚）。",
                *color as char, count
            ));
        }
    }

    let mut cube = RawCube::default();
    let mut seen = [false; 8];

    for (slot, indices) in CORNERS.iter().enumerate() {
        let mut found = false;
        for (piece, home) in CORNERS.iter().enumerate() {
            for orientation in 0..3 {
                if (0..3).all(|n| f[indices[(n + orientation) % 3]] == FACES[home[n] / 4]) {
                    if seen[piece] {
                        return Err("同じコーナーピースが複数あります。".into());
                    }
                    seen[piece] = true;
                    cube.cp[slot] = Corner::from_u8(piece as u8);
                    cube.co[slot] = orientation as u8;
                    found = true;
                    break;
                }
            }
            if found {
                break;
            }
        }
        if !found {
            return Err(format!(
                "スロット {} のコーナー配色が物理的に不正です。",
                slot + 1
            ));
        }
    }

    // コーナー向きの総和（Twist Parity）チェック: 3の倍数でなければならない
    if cube.co.iter().map(|&o| o as usize).sum::<usize>() % 3 != 0 {
        return Err("コーナーの向きが不正です（ねじれパリティエラー）。".into());
    }

    Ok(cube)
}

/// 置換の転倒数を数え、偶奇パリティ（偶置換=0, 奇置換=1）を計算します。
pub fn parity(p: &[u8]) -> usize {
    (0..p.len())
        .map(|i| (i + 1..p.len()).filter(|j| p[i] > p[*j]).count())
        .sum::<usize>()
        % 2
}

/// `RawCube` 構造体の内部状態から、24文字の展開図文字列を生成します。
pub fn facelets(cube: &RawCube) -> String {
    let mut f = [0u8; 24];
    for (slot, indices) in CORNERS.iter().enumerate() {
        let piece_idx = cube.cp[slot] as usize;
        let home = CORNERS[piece_idx];
        let ori = cube.co[slot] as usize;
        for n in 0..3 {
            f[indices[(n + ori) % 3]] = FACES[home[n] / 4];
        }
    }
    String::from_utf8(f.to_vec()).unwrap()
}

/// 空白区切りの回転記号文字列（例: "R U R' U'"）を解析し、回転インデックス列に変換します。
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

/// 単一の回転インデックス（0..17）を標準文字列表現（"R", "U2", "F'" など）に変換します。
pub fn notation(m: usize) -> String {
    format!("{}{}", FACES[m / 3] as char, ["", "2", "'"][m % 3])
}

/// 指定したキューブ状態に一連の回転操作を順次適用した新しい状態を返します。
pub fn apply(cube: &RawCube, moves: &[usize]) -> RawCube {
    moves
        .iter()
        .fold(*cube, |c, m| c.multiply(move_cube_18(*m)))
}

/// 疑似乱数シードに基づき、冗長手を排除した 2x2 WCA 風スクランブル手順（9〜11手）を生成します。
pub fn scramble(seed: u32) -> Vec<usize> {
    let mut rng = seed.wrapping_mul(1103515245).wrapping_add(12345);
    let len = 9 + (rng % 3) as usize; // 9, 10, または 11 手
    let mut moves = Vec::with_capacity(len);
    let mut last_face = 255usize;

    // 2x2 では DBL コーナーを固定して U, R, F の3面でスクランブルするのが標準
    let allowed_faces = [0usize, 1usize, 2usize]; // U, R, F

    while moves.len() < len {
        rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
        let face_idx = (rng as usize) % allowed_faces.len();
        let face = allowed_faces[face_idx];
        if face == last_face {
            continue;
        }
        rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
        let turn = (rng as usize) % 3; // 0: 90°, 1: 180°, 2: 270°(')
        moves.push(face * 3 + turn);
        last_face = face;
    }
    moves
}
