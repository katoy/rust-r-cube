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

/// キューブの状態が有効かどうかを判定
///
/// 3x3ルービックキューブとして物理的に可能な配置かどうかをチェックします。
/// - 各色が9つずつあるか
/// - コーナーの位置パリティと向きパリティ
/// - エッジの位置パリティと向きパリティ（3x3特有）
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

/// コーナーの構成ステッカーのインデックス定義 (Kociemba順: 0:UFR, 1:UFL, 2:ULB, 3:UBR, 4:DFR, 5:DLF, 6:DBL, 7:DRB)
/// 各スロットのステッカー順序: (PrimaryFace(U/D) -> Side1 -> Side2)
pub const CORNER_STICKERS: [[usize; 3]; 8] = [
    [8, 38, 27],  // 0: UFR (U8, F38, R27)
    [6, 20, 36],  // 1: UFL (U6, L20, F36)
    [0, 47, 18],  // 2: ULB (U0, B47, L18)
    [2, 29, 45],  // 3: UBR (U2, R29, B45)
    [11, 33, 44], // 4: DFR (D11, R33, F44)
    [9, 42, 26],  // 5: DLF (D9, F42, L26)
    [15, 24, 53], // 6: DBL (D15, L24, B53)
    [17, 51, 35], // 7: DRB (D17, B51, R35)
];

/// エッジの構成ステッカーのインデックス定義 (Kociemba順: 0:UR, 1:UF, 2:UL, 3:UB, 4:DR, 5:DF, 6:DL, 7:DB, 8:FR, 9:FL, 10:BL, 11:BR)
/// 各スロットのステッカー順序: (PrimaryFace(U/D/F/B) -> Side)
pub const EDGE_STICKERS: [[usize; 2]; 12] = [
    [5, 28],  // 0: UR (U5, R1)
    [7, 37],  // 1: UF (U7, F1)
    [3, 19],  // 2: UL (U3, L1)
    [1, 46],  // 3: UB (U1, B1)
    [14, 34], // 4: DR (D5, R7)
    [10, 43], // 5: DF (D1, F7)
    [12, 25], // 6: DL (D3, L7)
    [16, 52], // 7: DB (D7, B7)
    [41, 30], // 8: FR (F5, R3)
    [39, 23], // 9: FL (F3, L5)
    [50, 21], // 10: BL (B5, L3)
    [48, 32], // 11: BR (B3, R5)
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
        if is_opposite(colors[0], colors[1])
            || is_opposite(colors[1], colors[2])
            || is_opposite(colors[0], colors[2])
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
