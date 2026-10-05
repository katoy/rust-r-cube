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

/// 54文字の展開図文字列（U, R, F, D, L, B）を解析し、合法な `RawCube` 構造体に変換します。
/// 各面の枚数（各9枚）、センター色の一致、ピースの重複、コーナー・エッジの向きパリティ、
/// およびピース置換全体の偶奇パリティを厳密に検証します。
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

/// 置換の転倒数を数え、パリティ（偶置換=0, 奇置換=1）を計算します。
pub(crate) fn parity(p: &[u8]) -> usize {
    (0..p.len())
        .map(|i| (i + 1..p.len()).filter(|j| p[i] > p[*j]).count())
        .sum::<usize>()
        % 2
}
/// `RawCube` 構造体の内部状態から、54文字の展開図文字列（URFDLB）を生成します。
pub fn facelets(cube: &RawCube) -> String {
    let mut f = <[u8; 54]>::try_from(SOLVED.as_bytes()).unwrap();
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

/// 単一の回転インデックス（0..18）を標準文字列表現（"R", "U2", "F'" など）に変換します。
pub fn notation(m: usize) -> String {
    format!("{}{}", FACES[m / 3] as char, ["", "2", "'"][m % 3])
}

/// 指定したキューブ状態に一連の回転操作を順次適用した新しい状態を返します。
pub fn apply(cube: &RawCube, moves: &[usize]) -> RawCube {
    moves
        .iter()
        .fold(*cube, |c, m| c.multiply(move_cube_18(*m)))
}

#[inline]
fn is_opposite_face(f1: usize, f2: usize) -> bool {
    (f1 < f2 && f1 + 3 == f2) || (f2 < f1 && f2 + 3 == f1)
}

/// 指定した疑似乱数シードから、対向面や同一面の冗長回転を排除した25手のスクランブル手順を生成します。
pub fn scramble(seed: u32) -> Vec<usize> {
    let mut x = seed.max(1);
    let mut moves = Vec::with_capacity(25);
    while moves.len() < 25 {
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        let m = x as usize % 18;
        let face = m / 3;

        let len = moves.len();
        if len > 0 && moves[len - 1] / 3 == face {
            continue;
        }
        if len > 1 && moves[len - 2] / 3 == face && is_opposite_face(moves[len - 1] / 3, face) {
            continue;
        }
        moves.push(m);
    }
    moves
}

// シリアライズに依存しないピース定義の明示的参照を維持
const _: usize = Corner::UFR as usize + Edge::UR as usize;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_moves_too_long() {
        let long_str = "R ".repeat(2049); // 4098 chars > 4096
        assert!(parse_moves(&long_str).is_err());
    }

    #[test]
    fn test_parse_moves_tokens() {
        assert_eq!(parse_moves("R U' F2").unwrap(), vec![3, 2, 7]);
        assert!(parse_moves("X").is_err());
        assert!(parse_moves("R3").is_err());
    }

    #[test]
    fn test_parse_state_center_mismatch() {
        // 各面9枚あるが、センター（index 4と13）が入れ替わっている
        let mut f = SOLVED.as_bytes().to_vec();
        f.swap(4, 13);
        let s = String::from_utf8(f).unwrap();
        let err = parse_state(&s).unwrap_err();
        assert!(err.contains("センターの色は変更できません"));
    }

    #[test]
    fn test_parse_state_corner_duplicate_and_invalid() {
        // 1. コーナーの色の組み合わせが不正: UFR [8, 9, 20] の F(20) を D にし、DFR の D(29) を F にする
        // 全面9枚を維持したまま、存在しないコーナー (U, R, D) を作成
        let mut f1 = SOLVED.as_bytes().to_vec();
        f1[20] = b'D';
        f1[29] = b'F';
        let s1 = String::from_utf8(f1).unwrap();
        let err1 = parse_state(&s1).unwrap_err();
        assert!(err1.contains("色の組み合わせが不正"));

        // 2. 重複コーナー: UFR [8, 9, 20] と同一配色を UFL [6, 18, 38] に配置し、
        // DRB [35, 17, 51] の 17(R) を L にして全体の9枚カウントを保持
        let mut f2 = SOLVED.as_bytes().to_vec();
        f2[6] = b'U';
        f2[18] = b'R';
        f2[38] = b'F';
        f2[17] = b'L';
        let s2 = String::from_utf8(f2).unwrap();
        let err2 = parse_state(&s2).unwrap_err();
        assert_eq!(err2, "同じコーナーピースが複数あります。");
    }

    #[test]
    fn test_parse_state_edge_duplicate_and_invalid() {
        // 1. エッジの色の組み合わせが不正: UR [5, 10] の R(10) を D にし、DR [32, 16] の D(32) を R にする
        // 全面9枚を維持したまま、存在しないエッジ (U, D) を作成
        let mut f1 = SOLVED.as_bytes().to_vec();
        f1[10] = b'D';
        f1[32] = b'R';
        let s1 = String::from_utf8(f1).unwrap();
        let err1 = parse_state(&s1).unwrap_err();
        assert!(err1.contains("色の組み合わせが不正"));

        // 2. 重複エッジ: UR [5, 10] と同一配色を UF [7, 19] に配置し、
        // FR [23, 12] の 12(R) を F にして全体の9枚カウントを保持
        let mut f2 = SOLVED.as_bytes().to_vec();
        f2[7] = b'U';
        f2[19] = b'R';
        f2[12] = b'F';
        let s2 = String::from_utf8(f2).unwrap();
        let err2 = parse_state(&s2).unwrap_err();
        assert_eq!(err2, "同じエッジピースが複数あります。");
    }

    #[test]
    fn test_scramble_and_notation() {
        let sc = scramble(42);
        assert_eq!(sc.len(), 25);
        for &m in &sc {
            let not = notation(m);
            assert!(!not.is_empty());
        }
    }
}
