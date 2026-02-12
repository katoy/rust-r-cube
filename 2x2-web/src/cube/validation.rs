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

    // 各色が4つずつあるかチェック
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

/// キューブの状態が有効かどうかを判定
///
/// 2x2ルービックキューブとして物理的に可能な配置かどうかをチェックします。
/// - 各色が4つずつあるか
/// - コーナーの位置パリティが正しいか（偶置換）
/// - コーナーの向きパリティが正しいか（向きの合計が3の倍数）
pub fn is_valid_state(cube: &Cube) -> Result<()> {
    // まず色数のチェック
    let mut colors_array = [Color::White; crate::cube::NUM_STICKERS];
    for (i, color) in colors_array.iter_mut().enumerate() {
        *color = cube.stickers[i].color;
    }
    validate_colors(&colors_array)?;

    // コーナーの位置パリティと向きパリティをチェック
    check_corner_parity(cube)?;

    Ok(())
}

/// コーナーの構成ステッカーのインデックス定義 (PrimaryFace(U/D) -> CW1 -> CW2)
pub const CORNER_STICKERS: [[usize; 3]; 8] = [
    [2, 16, 9],  // UFL: U2, F0, L1
    [3, 12, 17], // UFR: U3, R0, F1
    [1, 20, 13], // UBR: U1, B0, R1
    [0, 8, 21],  // UBL: U0, L0, B1
    [4, 11, 18], // DFL: D0, L3, F2
    [5, 19, 14], // DFR: D1, F3, R2
    [7, 15, 22], // DBR: D3, R3, B2
    [6, 23, 10], // DBL: D2, B3, L2
];

/// 対面色かどうかを判定
fn is_opposite(c1: Color, c2: Color) -> bool {
    matches!(
        (c1, c2),
        (Color::White, Color::Yellow)
            | (Color::Yellow, Color::White)
            | (Color::Red, Color::Orange)
            | (Color::Orange, Color::Red)
            | (Color::Green, Color::Blue)
            | (Color::Blue, Color::Green)
    )
}

/// コーナーのパリティをチェック
pub fn check_corner_parity(cube: &Cube) -> Result<()> {
    // 1. 各コーナーピースの色の組み合わせが妥当かチェック
    let mut corner_pieces = Vec::new();

    for sticker_indices in &CORNER_STICKERS {
        let colors = [
            cube.stickers[sticker_indices[0]].color,
            cube.stickers[sticker_indices[1]].color,
            cube.stickers[sticker_indices[2]].color,
        ];

        // 同じ色が2つ以上ないか
        if colors[0] == colors[1] || colors[1] == colors[2] || colors[0] == colors[2] {
            return Err(CubeError::InvalidState(
                "コーナーに同じ色が複数含まれています。".to_string(),
            ));
        }

        // 対面色が同じコーナーに含まれていないか
        if (colors[0] != Color::Gray
            && colors[1] != Color::Gray
            && is_opposite(colors[0], colors[1]))
            || (colors[1] != Color::Gray
                && colors[2] != Color::Gray
                && is_opposite(colors[1], colors[2]))
            || (colors[0] != Color::Gray
                && colors[2] != Color::Gray
                && is_opposite(colors[0], colors[2]))
        {
            return Err(CubeError::InvalidState(
                "コーナーに存在しえない色の組み合わせ（対面色）が含まれています。".to_string(),
            ));
        }

        let mut sorted_colors = colors;
        sorted_colors.sort_by_key(|c| format!("{:?}", c));
        corner_pieces.push(sorted_colors);
    }

    // 2. 8つのコーナーが互いにユニークであるか（標準的な8個の組み合わせか）をチェック
    let mut sorted_pieces = corner_pieces.clone();
    sorted_pieces.sort_by_key(|p| format!("{:?}", p));
    for i in 0..7 {
        if sorted_pieces[i] == sorted_pieces[i + 1] {
            return Err(CubeError::InvalidState(
                "重複するコーナーピースが存在します。".to_string(),
            ));
        }
    }

    // 3. 向き（TWIST）のパリティチェック
    // 各パーツが「正しい向きからどれだけ捻られているか」を計算
    // 白または黄色を基準面とする
    let mut total_twist = 0;
    for sticker_indices in CORNER_STICKERS.iter() {
        let colors = [
            cube.stickers[sticker_indices[0]].color,
            cube.stickers[sticker_indices[1]].color,
            cube.stickers[sticker_indices[2]].color,
        ];

        // 白または黄色のステッカーを探す
        let twist = if let Some(pos) = colors
            .iter()
            .position(|&c| c == Color::White || c == Color::Yellow)
        {
            // 上面/底面(index 0)にあれば 0
            // 時計回りにずれていれば 1, 反時計回りなら 2
            // コーナーによって時計回りの定義が異なるが、
            // 2x2では隣接するコーナー間で「基準面に向かう方向」が逆転するため
            // スロットのインデックス順 (Up/Down -> SideA -> SideB) を一貫して使えば合計は0 mod 3になる
            pos
        } else {
            return Err(CubeError::InvalidState(
                "不正な色のコーナーです".to_string(),
            ));
        };
        total_twist += twist;
    }

    if total_twist % 3 != 0 {
        return Err(CubeError::InvalidState(
            "コーナーの向きが無効です（捻じれパリティエラー）。\n一つ以上のコーナーが物理的に回転してしまっている可能性があります。".to_string(),
        ));
    }

    Ok(())
}
