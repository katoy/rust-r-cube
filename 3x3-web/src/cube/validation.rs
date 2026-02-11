use super::{Color, Cube};
use crate::error::{CubeError, Result};

/// 色配列の妥当性をチェックします。
///
/// 各色が正確に9つずつ存在するかを確認します。
pub fn validate_colors(colors: &[Color; crate::cube::NUM_STICKERS]) -> Result<()> {
    let mut counts = [0usize; 7]; // Color Enum の数に合わせて7 (Grayを含む)
    for &color in colors {
        let idx = color as usize;
        if idx < 7 {
            counts[idx] += 1;
        }
    }

    // 各色が9つずつあるかチェック
    let expected_colors = [
        Color::White,
        Color::Yellow,
        Color::Green,
        Color::Blue,
        Color::Red,
        Color::Orange,
    ];

    for &color in &expected_colors {
        let count = counts[color as usize];
        if count != crate::cube::STICKERS_PER_FACE {
            if count == 0 {
                return Err(CubeError::ColorNotFound(format!("{color:?}")));
            } else {
                return Err(CubeError::InvalidColors(format!(
                    "{color:?}の数が{count}個です（{}個である必要があります）",
                    crate::cube::STICKERS_PER_FACE
                )));
            }
        }
    }

    Ok(())
}

/// キューブの状態が有効かどうかを判定します。
pub fn is_valid_state(cube: &Cube) -> Result<()> {
    // まず色数のチェック
    let mut colors_array = [Color::White; crate::cube::NUM_STICKERS];
    for (i, color) in colors_array.iter_mut().enumerate() {
        *color = cube.stickers[i].color;
    }
    validate_colors(&colors_array)?;

    // コーナーのパリティと一貫性をチェック
    check_corner_parity(cube)?;

    // エッジのパリティと一貫性をチェック
    check_edge_parity(cube)?;

    // 置換パリティをチェック
    check_total_permutation_parity(cube)?;

    Ok(())
}

pub const CORNER_STICKERS: [[usize; 3]; 8] = [
    [8, 38, 27],  // UFR
    [6, 20, 36],  // UFL
    [0, 47, 18],  // ULB
    [2, 29, 45],  // UBR
    [11, 33, 44], // DFR
    [9, 42, 26],  // DLF
    [15, 24, 53], // DBL
    [17, 51, 35], // DRB
];

pub const EDGE_STICKERS: [[usize; 2]; 12] = [
    [5, 28],  // UR
    [7, 37],  // UF
    [3, 19],  // UL
    [1, 46],  // UB
    [14, 34], // DR
    [10, 43], // DF
    [12, 25], // DL
    [16, 52], // DB
    [41, 30], // FR
    [39, 23], // FL
    [50, 21], // BL
    [48, 32], // BR
];

pub const CENTER_STICKERS: [usize; 6] = [4, 13, 22, 31, 40, 49];

/// コーナーのパリティをチェックします。
pub fn check_corner_parity(cube: &Cube) -> Result<()> {
    let rc = crate::kociemba::RawCube::from_cube(cube).map_err(CubeError::InvalidState)?;

    // ユニーク性チェック
    let mut corner_counts = [0u8; 8];
    for &c in &rc.cp {
        corner_counts[c as usize] += 1;
    }
    if corner_counts.iter().any(|&count| count != 1) {
        return Err(CubeError::InvalidState(
            "コーナーピースの重複または欠落があります".to_string(),
        ));
    }

    let total_twist: u8 = rc.co.iter().sum();
    if total_twist % 3 != 0 {
        return Err(CubeError::InvalidState(format!(
            "コーナーの向きパリティが不正です (合計捻れ: {})",
            total_twist
        )));
    }
    Ok(())
}

/// エッジのパリティをチェックします。
pub fn check_edge_parity(cube: &Cube) -> Result<()> {
    let rc = crate::kociemba::RawCube::from_cube(cube).map_err(CubeError::InvalidState)?;

    // ユニーク性チェック
    let mut edge_counts = [0u8; 12];
    for &e in &rc.ep {
        edge_counts[e as usize] += 1;
    }
    if edge_counts.iter().any(|&count| count != 1) {
        return Err(CubeError::InvalidState(
            "エッジピースの重複または欠落があります".to_string(),
        ));
    }

    let total_flip: u8 = rc.eo.iter().sum();
    if total_flip % 2 != 0 {
        return Err(CubeError::InvalidState(format!(
            "エッジの向きパリティが不正です (合計反転: {})",
            total_flip
        )));
    }
    Ok(())
}

/// 置換パリティのチェック。
pub fn check_total_permutation_parity(cube: &Cube) -> Result<()> {
    use crate::kociemba::RawCube;
    let rc = RawCube::from_cube(cube).map_err(CubeError::InvalidState)?;

    fn get_permutation_parity(p: &[usize]) -> usize {
        let mut visited = vec![false; p.len()];
        let mut total_swaps = 0;
        for i in 0..p.len() {
            if !visited[i] {
                let mut curr = i;
                let mut cycle_len = 0;
                while !visited[curr] {
                    visited[curr] = true;
                    curr = p[curr];
                    cycle_len += 1;
                }
                if cycle_len > 1 {
                    total_swaps += cycle_len - 1;
                }
            }
        }
        total_swaps % 2
    }

    let cp_p: Vec<usize> = rc.cp.iter().map(|&c| c as usize).collect();
    let ep_p: Vec<usize> = rc.ep.iter().map(|&e| e as usize).collect();

    // 置換パリティを計算する前に、既にユニーク性チェックが通っている前提
    // (重複があると permutation として成立しない)

    let cp_parity = get_permutation_parity(&cp_p);
    let ep_parity = get_permutation_parity(&ep_p);

    if (cp_parity + ep_parity) % 2 != 0 {
        return Err(CubeError::InvalidState(
            "置換パリティが不正です。コーナーとエッジの配置が物理的に不可能です。".to_string(),
        ));
    }

    Ok(())
}
