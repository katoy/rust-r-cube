use super::{Color, Cube};
use crate::error::{CubeError, Result};

/// キューブの状態をファイル形式の文字列に変換
/// 形式: 各ステッカーを1文字で表現（例: W, G, .）
pub fn to_file_format(cube: &Cube) -> String {
    let mut result = String::new();

    // ヘルパー関数：面の情報を取得
    let get_face = |face_idx: usize| -> String {
        let start = face_idx * crate::cube::STICKERS_PER_FACE;
        let mut face_str = String::new();
        for i in 0..crate::cube::STICKERS_PER_FACE {
            let sticker = cube.stickers[start + i];
            let c = match sticker.color {
                Color::White => 'W',
                Color::Yellow => 'Y',
                Color::Green => 'G',
                Color::Blue => 'B',
                Color::Red => 'R',
                Color::Orange => 'O',
                Color::Gray => '.', // 未設定はドット
            };
            face_str.push(c);
        }
        face_str
    };

    // 展開図形式で出力
    // 1行目: Up
    result.push_str("          ");
    result.push_str(&get_face(0)); // Up
    result.push('\n');

    // 2行目: Left Front Right Back
    result.push_str(&get_face(2)); // Left
    result.push(' ');
    result.push_str(&get_face(4)); // Front
    result.push(' ');
    result.push_str(&get_face(3)); // Right
    result.push(' ');
    result.push_str(&get_face(5)); // Back
    result.push('\n');

    // 3行目: Down
    result.push_str("          ");
    result.push_str(&get_face(1)); // Down
    result.push('\n');

    result
}

/// ファイル形式の文字列からキューブを作成
pub fn from_file_format(s: &str) -> Result<Cube> {
    let lines: Vec<&str> = s.lines().collect();

    if lines.len() != 3 {
        return Err(CubeError::InvalidFormat(format!(
            "3行必要ですが{}行しかありません",
            lines.len()
        )));
    }

    // 色を解析
    let parse_colors = |s: &str| -> Result<Vec<Color>> {
        s.chars()
            .filter(|c| !c.is_whitespace())
            .map(|c| match c.to_ascii_uppercase() {
                'W' => Ok(Color::White),
                'Y' => Ok(Color::Yellow),
                'G' => Ok(Color::Green),
                'B' => Ok(Color::Blue),
                'R' => Ok(Color::Red),
                'O' => Ok(Color::Orange),
                '.' => Ok(Color::Gray),
                _ => Err(CubeError::InvalidColorChar(c)),
            })
            .collect()
    };

    // 各行から色を取得
    let line1_colors = parse_colors(lines[0])?;
    let line2_colors = parse_colors(lines[1])?;
    let line3_colors = parse_colors(lines[2])?;

    // 検証
    if line1_colors.len() != 9 {
        return Err(CubeError::InvalidFormat(format!(
            "1行目: 9ステッカー必要ですが{}個です",
            line1_colors.len()
        )));
    }
    if line2_colors.len() != 36 {
        return Err(CubeError::InvalidFormat(format!(
            "2行目: 36ステッカー必要ですが{}個です",
            line2_colors.len()
        )));
    }
    if line3_colors.len() != 9 {
        return Err(CubeError::InvalidFormat(format!(
            "3行目: 9ステッカー必要ですが{}個です",
            line3_colors.len()
        )));
    }

    // 24色の配列を作成（内部順序: Up, Down, Left, Right, Front, Back）

    // 各面の配置ルール: (面、ソースとなる色のスライス)
    const SPF: usize = crate::cube::STICKERS_PER_FACE;
    use crate::cube::{Face, Sticker};
    let mut stickers = [Sticker::new(Color::White); crate::cube::NUM_STICKERS];

    let mut map_face = |face: Face, line_colors: &[Color]| {
        let start = face.start_index();
        for i in 0..SPF {
            stickers[start + i] = Sticker::new(line_colors[i]);
        }
    };

    map_face(Face::Up, &line1_colors);
    map_face(Face::Down, &line3_colors);
    map_face(Face::Left, &line2_colors[0..9]);
    map_face(Face::Front, &line2_colors[9..18]);
    map_face(Face::Right, &line2_colors[18..27]);
    map_face(Face::Back, &line2_colors[27..36]);

    let mut cube = Cube { stickers };

    // スキャン途中（Grayあり）でない場合は、向きを初期化
    let has_gray = cube.stickers.iter().any(|s| s.color == Color::Gray);
    if !has_gray {
        cube = cube.with_clockwise_orientations();
    }

    Ok(cube)
}
