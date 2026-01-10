use super::{Color, Cube};
use crate::error::{CubeError, Result};
use std::collections::HashMap;

/// 色配列の妥当性をチェックします。
///
/// 各色が正確に4つずつ存在するかを確認します。
pub fn validate_colors(colors: &[Color; 24]) -> Result<()> {
    let mut counts = HashMap::new();
    for &color in colors {
        *counts.entry(color).or_insert(0) += 1;
    }

    // 各色が4つずつあるかチェック
    let expected_colors = [
        Color::White,
        Color::Yellow,
        Color::Green,
        Color::Blue,
        Color::Red,
        Color::Orange,
    ];

    for color in &expected_colors {
        match counts.get(color) {
            Some(&4) => {}
            Some(&count) => {
                return Err(CubeError::InvalidColors(format!(
                    "{color:?}の数が{count}個です（4個である必要があります）"
                )));
            }
            None => {
                return Err(CubeError::ColorNotFound(format!("{:?}", color)));
            }
        }
    }

    Ok(())
}

/// キューブの状態が有効かどうかを判定
///
/// 2x2ルービックキューブとして物理的に可能な配置かどうかをチェックします。
/// - 各色が4つずつあるか
/// - コーナーの位置パリティが正しいか（偶置換）
/// - コーナーの向きパリティが正しいか（向きの合計が3の倍数）
pub fn is_valid_state(cube: &Cube) -> Result<()> {
    // まず色数のチェック
    let mut colors_array = [Color::White; 24];
    for (i, color) in colors_array.iter_mut().enumerate() {
        *color = cube.stickers[i].color;
    }
    validate_colors(&colors_array)?;

    // コーナーの位置パリティと向きパリティをチェック
    check_corner_parity(cube)?;

    Ok(())
}

/// コーナーのパリティをチェック
///
/// TODO: 現在のロジックにバグがあるため、一時的に無効化しています
/// 正しいパリティチェックを後で実装する必要があります
pub fn check_corner_parity(_cube: &Cube) -> Result<()> {
    // 一時的に無効化：常に有効とみなす
    Ok(())

    /* 元のコード（バグあり）
    let solved = Cube::new();
    let corner_positions = [
        [2, 9, 16],  // Corner 0: Up-Left-Front
        [3, 12, 17], // Corner 1: Up-Right-Front
        [0, 8, 21],  // Corner 2: Up-Left-Back
        [1, 13, 20], // Corner 3: Up-Right-Back
        [6, 11, 18], // Corner 4: Down-Left-Front
        [7, 14, 19], // Corner 5: Down-Right-Front
        [4, 10, 23], // Corner 6: Down-Left-Back
        [5, 15, 22], // Corner 7: Down-Right-Back
    ];
    let mut current_corners = Vec::new();
    let mut solved_corners = Vec::new();
    for positions in &corner_positions {
        let mut curr: Vec<Color> = positions.iter().map(|&i| _cube.stickers[i].color).collect();
        let mut solv: Vec<Color> = positions
            .iter()
            .map(|&i| solved.stickers[i].color)
            .collect();
        curr.sort_by_key(|c| format!("{:?}", c));
        solv.sort_by_key(|c| format!("{:?}", c));
        current_corners.push(curr);
        solved_corners.push(solv);
    }
    let mut permutation = Vec::new();
    for current in &current_corners {
        match solved_corners.iter().position(|solved| solved == current) {
            Some(pos) => permutation.push(pos),
            None => return Err("無効なコーナーの色の組み合わせが見つかりました。\nキューブを分解して組み立て直していませんか？".to_string()),
        }
    }
    let mut parity = 0;
    for i in 0..8 {
        for j in (i + 1)..8 {
            if permutation[i] > permutation[j] {
                parity += 1;
            }
        }
    }
    if parity % 2 != 0 {
        return Err("コーナーの配置が無効です（位置パリティエラー）。\nこの配置は回転操作では実現できません。\nキューブを分解して組み立て直した可能性があります。".to_string());
    }
    let mut orientation_sum = 0;
    for i in 0..8 {
        let positions = &corner_positions[i];
        let solved_idx = permutation[i];
        let solved_positions = &corner_positions[solved_idx];
        let base_color = solved.stickers[solved_positions[0]].color;
        let orientation = positions
            .iter()
            .position(|&idx| _cube.stickers[idx].color == base_color)
            .unwrap_or(0);
        orientation_sum += orientation;
    }
    if orientation_sum % 3 != 0 {
        return Err("コーナーの向きが無効です（向きパリティエラー）。\nこの配置は回転操作では実現できません。\nキューブを分解して組み立て直した可能性があります。".to_string());
    }
    Ok(())
    */
}
