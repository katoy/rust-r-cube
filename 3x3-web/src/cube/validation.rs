use super::{Color, Cube};
use crate::error::{CubeError, Result};

/// 色配列の妥当性をチェックします。
///
/// 各色が正確に4つずつ存在するかを確認します。
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
///
/// 3x3ルービックキューブは、ランダムにステッカーを貼り直した場合、
/// 物理的に回転操作だけで完成させることはできません。
/// この関数は、以下の物理法則（不変量）をチェックします。
/// 1. 各色のステッカーが正確に9枚ずつある。
/// 2. コーナーの捻じれパリティ（3の倍数）。
/// 3. エッジの反転パリティ（偶数）。
/// 4. 置換パリティ（コーナーの交換回数とエッジの交換回数の和が偶数）。
pub fn is_valid_state(cube: &Cube) -> Result<()> {
    // まず色数のチェック
    let mut colors_array = [Color::White; crate::cube::NUM_STICKERS];
    for (i, color) in colors_array.iter_mut().enumerate() {
        *color = cube.stickers[i].color;
    }
    validate_colors(&colors_array)?;

    // コーナーの位置パリティと向きパリティをチェック
    check_corner_parity(cube)?;

    // エッジの位置パリティと向きパリティをチェック
    check_edge_parity(cube)?;

    // 置換パリティ（コーナー+エッジの一貫性）をチェック
    check_total_permutation_parity(cube)?;

    Ok(())
}

// (CORNER_STICKERS などの定数定義は省略)

/// コーナーのパリティをチェックします。
///
/// 1. 各コーナーピース（3色）の組み合わせが物理的に実在するものかを確認します。
/// 2. 各コーナーの捻じれ（Twist）の合計が 3 の倍数であることを確認します。
///    1つのコーナーだけを 120度 回転させた状態は物理的に不可能です。
pub fn check_corner_parity(cube: &Cube) -> Result<()> {
    // ... (実装)
}

/// エッジのパリティをチェックします。
///
/// 1. 各エッジピース（2色）の組み合わせがユニークであることを確認します。
/// 2. エッジの向き（Flip）の反転回数の合計が偶数であることを確認します。
///    1つのエッジだけを反転させた状態は物理的に不可能です。
pub fn check_edge_parity(cube: &Cube) -> Result<()> {
    // ... (実装)
}

/// 置換パリティのチェック。
///
/// コーナーの置換（位置の入れ替え）に必要な交換回数の偶奇と、
/// エッジの置換に必要な交換回数の偶奇が一致することを確認します。
/// 「2つのコーナーだけを入れ替え、他はそのまま」という状態は物理的に不可能です。
pub fn check_total_permutation_parity(cube: &Cube) -> Result<()> {
    // ... (実装)
}
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

    let cp_parity = get_permutation_parity(&cp_p);
    let ep_parity = get_permutation_parity(&ep_p);

    #[allow(clippy::manual_is_multiple_of)]
    if (cp_parity + ep_parity) % 2 != 0 {
        return Err(CubeError::InvalidState(
            "置換パリティが不正です。コーナーとエッジの配置が物理的に不可能です。".to_string(),
        ));
    }

    Ok(())
}
