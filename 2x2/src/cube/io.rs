use super::{Color, Cube};
// from_colorsはmod.rsにあるのでCube::from_colorsで呼べる

/// キューブの状態をファイル形式の文字列に変換
pub fn to_file_format(cube: &Cube) -> String {
    let mut result = String::new();

    // ヘルパー関数：面の4文字を取得
    let get_face = |face_idx: usize| -> String {
        let start = face_idx * 4;
        (0..4)
            .map(|i| match cube.stickers[start + i].color {
                Color::White => 'W',
                Color::Yellow => 'Y',
                Color::Green => 'G',
                Color::Blue => 'B',
                Color::Red => 'R',
                Color::Orange => 'O',
                Color::Gray => ' ',
            })
            .collect()
    };

    // 展開図形式で出力
    // 1行目: Up (Down面は使わない、White面)
    result.push_str("     ");
    result.push_str(&get_face(0)); // Up
    result.push('\n');

    // 2行目: Left Front Right Back (Yellow Green Blue Red)
    result.push_str(&get_face(2)); // Left
    result.push(' ');
    result.push_str(&get_face(4)); // Front
    result.push(' ');
    result.push_str(&get_face(3)); // Right
    result.push(' ');
    result.push_str(&get_face(5)); // Back
    result.push('\n');

    // 3行目: Down (Orange面)
    result.push_str("     ");
    result.push_str(&get_face(1)); // Down
    result.push('\n');

    result
}

/// ファイル形式の文字列からキューブを作成
pub fn from_file_format(s: &str) -> Result<Cube, String> {
    let lines: Vec<&str> = s.lines().collect();

    if lines.len() != 3 {
        return Err(format!("3行必要ですが{}行しかありません", lines.len()));
    }

    // 色を解析
    let parse_colors = |s: &str| -> Result<Vec<Color>, String> {
        s.chars()
            .filter(|c| !c.is_whitespace())
            .map(|c| match c.to_ascii_uppercase() {
                'W' => Ok(Color::White),
                'Y' => Ok(Color::Yellow),
                'G' => Ok(Color::Green),
                'B' => Ok(Color::Blue),
                'R' => Ok(Color::Red),
                'O' => Ok(Color::Orange),
                _ => Err(format!("無効な色文字: {}", c)),
            })
            .collect()
    };

    // 各行から色を取得
    let line1_colors = parse_colors(lines[0])?;
    let line2_colors = parse_colors(lines[1])?;
    let line3_colors = parse_colors(lines[2])?;

    // 検証
    if line1_colors.len() != 4 {
        return Err(format!(
            "1行目: 4文字必要ですが{}文字です",
            line1_colors.len()
        ));
    }
    if line2_colors.len() != 16 {
        return Err(format!(
            "2行目: 16文字必要ですが{}文字です",
            line2_colors.len()
        ));
    }
    if line3_colors.len() != 4 {
        return Err(format!(
            "3行目: 4文字必要ですが{}文字です",
            line3_colors.len()
        ));
    }

    // 24色の配列を作成（内部順序: Up, Down, Left, Right, Front, Back）
    let mut colors = vec![Color::White; 24];

    // Up (0-3)
    colors[0..4].copy_from_slice(&line1_colors);

    // Down (4-7)
    colors[4..8].copy_from_slice(&line3_colors);

    // Left (8-11)
    colors[8..12].copy_from_slice(&line2_colors[0..4]);

    // Right (12-15)
    colors[12..16].copy_from_slice(&line2_colors[8..12]);

    // Front (16-19)
    colors[16..20].copy_from_slice(&line2_colors[4..8]);

    // Back (20-23)
    colors[20..24].copy_from_slice(&line2_colors[12..16]);

    let colors_array: [Color; 24] = colors
        .try_into()
        .map_err(|_| "色の数が24個ではありません".to_string())?;

    // 妥当性チェック
    use super::validation;
    validation::validate_colors(&colors_array)?;

    Cube::from_colors(&colors_array)
}

/// 配列スライスへのコピーヘルパー（標準ライブラリにあるが明示的に使用）
trait CopySlice<T> {
    fn copy_from_slice(&mut self, src: &[T])
    where
        T: Copy;
}

impl<T> CopySlice<T> for [T] {
    fn copy_from_slice(&mut self, src: &[T])
    where
        T: Copy,
    {
        self.copy_from_slice(src)
    }
}
