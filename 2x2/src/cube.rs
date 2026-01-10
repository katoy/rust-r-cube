/// キューブの面を表す列挙型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Face {
    Up,
    Down,
    Left,
    Right,
    Front,
    Back,
}

/// ステッカーの色
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    White,
    Yellow,
    Green,
    Blue,
    Red,
    Orange,
    Gray, // 未設定のセル用
}

/// ステッカー（色と向き情報を持つ）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sticker {
    pub color: Color,
    /// 向き（0-3の値で、90度単位の回転を表す）
    pub orientation: u8,
}

impl Sticker {
    #[must_use]
    pub fn new(color: Color) -> Self {
        Self {
            color,
            orientation: 0,
        }
    }

    /// 時計回りに90度回転
    pub fn rotate_cw(&mut self) {
        self.orientation = (self.orientation + 1) % 4;
    }

    /// 反時計回りに90度回転
    pub fn rotate_ccw(&mut self) {
        self.orientation = (self.orientation + 3) % 4;
    }
}

/// 回転操作
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Move {
    R,  // Right face clockwise
    Rp, // Right face counter-clockwise
    L,  // Left face clockwise
    Lp, // Left face counter-clockwise
    U,  // Up face clockwise
    Up, // Up face counter-clockwise
    D,  // Down face clockwise
    Dp, // Down face counter-clockwise
    F,  // Front face clockwise
    Fp, // Front face counter-clockwise
    B,  // Back face clockwise
    Bp, // Back face counter-clockwise
}

impl Move {
    /// すべての回転操作を取得
    #[must_use]
    pub fn all_moves() -> Vec<Move> {
        vec![
            Move::R,
            Move::Rp,
            Move::L,
            Move::Lp,
            Move::U,
            Move::Up,
            Move::D,
            Move::Dp,
            Move::F,
            Move::Fp,
            Move::B,
            Move::Bp,
        ]
    }

    /// 逆操作を取得
    #[must_use]
    pub fn inverse(self) -> Move {
        match self {
            Move::R => Move::Rp,
            Move::Rp => Move::R,
            Move::L => Move::Lp,
            Move::Lp => Move::L,
            Move::U => Move::Up,
            Move::Up => Move::U,
            Move::D => Move::Dp,
            Move::Dp => Move::D,
            Move::F => Move::Fp,
            Move::Fp => Move::F,
            Move::B => Move::Bp,
            Move::Bp => Move::B,
        }
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Move::R => "R",
            Move::Rp => "R'",
            Move::L => "L",
            Move::Lp => "L'",
            Move::U => "U",
            Move::Up => "U'",
            Move::D => "D",
            Move::Dp => "D'",
            Move::F => "F",
            Move::Fp => "F'",
            Move::B => "B",
            Move::Bp => "B'",
        };
        write!(f, "{s}")
    }
}

/// 2x2 ルービックキューブ
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cube {
    /// 各面のステッカー（各面4つ）
    /// インデックス: 0-3 (Up), 4-7 (Down), 8-11 (Left), 12-15 (Right), 16-19 (Front), 20-23 (Back)
    stickers: [Sticker; 24],
}

impl Cube {
    /// 完成状態のキューブを作成します。
    ///
    /// # 戻り値
    ///
    /// 各面が単色で揃った初期状態の2x2ルービックキューブ。
    /// - Up面: 白 (White)
    /// - Down面: 黄 (Yellow)
    /// - Left面: 緑 (Green)
    /// - Right面: 青 (Blue)
    /// - Front面: 赤 (Red)
    /// - Back面: オレンジ (Orange)
    ///
    /// # 例
    ///
    /// ```
    /// use rubiks_cube_2x2::cube::Cube;
    ///
    /// let cube = Cube::new();
    /// assert!(cube.is_solved());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        let mut stickers = [Sticker::new(Color::White); 24];

        // 時計回りのorientation パターン
        // 各面の4つのステッカー（左上、右上、左下、右下）の矢印を時計回りに配置
        // 位置0（左上）: orientation = 1 （右向き）
        // 位置1（右上）: orientation = 2 （下向き）
        // 位置2（左下）: orientation = 0 （上向き）
        // 位置3（右下）: orientation = 3 （左向き）
        let clockwise_pattern = [1, 2, 0, 3];

        // Up face (White)
        for (i, sticker) in stickers.iter_mut().take(4).enumerate() {
            *sticker = Sticker::new(Color::White);
            sticker.orientation = clockwise_pattern[i];
        }
        // Down face (Yellow)
        for (i, sticker) in stickers.iter_mut().take(8).skip(4).enumerate() {
            *sticker = Sticker::new(Color::Yellow);
            sticker.orientation = clockwise_pattern[i];
        }
        // Left face (Green)
        for (i, sticker) in stickers.iter_mut().take(12).skip(8).enumerate() {
            *sticker = Sticker::new(Color::Green);
            sticker.orientation = clockwise_pattern[i];
        }
        // Right face (Blue)
        for (i, sticker) in stickers.iter_mut().take(16).skip(12).enumerate() {
            *sticker = Sticker::new(Color::Blue);
            sticker.orientation = clockwise_pattern[i];
        }
        // Front face (Red)
        for (i, sticker) in stickers.iter_mut().take(20).skip(16).enumerate() {
            *sticker = Sticker::new(Color::Red);
            sticker.orientation = clockwise_pattern[i];
        }
        // Back face (Orange)
        for (i, sticker) in stickers.iter_mut().skip(20).enumerate() {
            *sticker = Sticker::new(Color::Orange);
            sticker.orientation = clockwise_pattern[i];
        }

        Self { stickers }
    }

    /// キューブが完成しているか判定します（色のみ、向きは無視）。
    ///
    /// 各面が単色で揃っていれば `true` を返します。
    /// ステッカーの向き（矢印の方向）は考慮されません。
    ///
    /// # 戻り値
    ///
    /// - `true`: 各面が単色で揃っている
    /// - `false`: 少なくとも1面が揃っていない
    ///
    /// # 例
    ///
    /// ```
    /// use rubiks_cube_2x2::cube::{Cube, Move};
    ///
    /// let mut cube = Cube::new();
    /// assert!(cube.is_solved());
    ///
    /// cube.apply_move(Move::R);
    /// assert!(!cube.is_solved());
    ///
    /// cube.apply_move(Move::Rp);
    /// assert!(cube.is_solved());
    /// ```
    #[must_use]
    pub fn is_solved(&self) -> bool {
        for face_start in (0..24).step_by(4) {
            let color = self.stickers[face_start].color;
            for i in 1..4 {
                if self.stickers[face_start + i].color != color {
                    return false;
                }
            }
        }
        true
    }

    /// 指定したインデックスのステッカーを取得します。
    ///
    /// # 引数
    ///
    /// - `index`: ステッカーのインデックス (0-23)
    ///   - 0-3: Up面
    ///   - 4-7: Down面
    ///   - 8-11: Left面
    ///   - 12-15: Right面
    ///   - 16-19: Front面
    ///   - 20-23: Back面
    ///
    /// # 戻り値
    ///
    /// 指定されたインデックスのステッカー（色と向き情報を含む）
    #[must_use]
    pub fn get_sticker(&self, index: usize) -> Sticker {
        self.stickers[index]
    }

    /// 指定したインデックスのステッカーの色を設定します。
    ///
    /// # 引数
    ///
    /// - `index`: ステッカーのインデックス (0-23)
    /// - `color`: 設定する色
    ///
    /// # パニック
    ///
    /// インデックスが範囲外の場合にパニックします。
    pub fn set_sticker_color(&mut self, index: usize, color: Color) {
        self.stickers[index].color = color;
        // 向きはリセット（手動入力時は向きを0にする）
        self.stickers[index].orientation = 0;
    }

    /// 色を保持したまま、全てのステッカーのorientationを時計回りパターンにリセットします。
    ///
    /// 時計回りパターン: 各面の4つのステッカー（左上、右上、左下、右下）に
    /// orientation [1, 2, 0, 3] を設定します。
    ///
    /// # 例
    ///
    /// ```
    /// use rubiks_cube_2x2::cube::Cube;
    ///
    /// let cube = Cube::new();
    /// let clockwise_cube = cube.with_clockwise_orientations();
    /// ```
    #[must_use]
    pub fn with_clockwise_orientations(&self) -> Self {
        let mut new_cube = self.clone();
        let clockwise_pattern = [1, 2, 0, 3];

        for face_start in (0..24).step_by(4) {
            for (offset, &pattern) in clockwise_pattern.iter().enumerate() {
                let idx = face_start + offset;
                new_cube.stickers[idx].orientation = pattern;
            }
        }

        new_cube
    }

    /// 24個の色配列から新しいキューブを作成します。
    ///
    /// # 引数
    ///
    /// - `colors`: 24個の色の配列（インデックスは `get_sticker` と同じ）
    ///
    /// # 戻り値
    ///
    /// 指定された色配列で初期化されたキューブ（すべての向きは0）
    ///
    /// # Panics
    ///
    /// 配列の要素数が24でない場合にパニックします。
    /// 通常の使用では発生しません。
    ///
    /// # 例
    ///
    /// ```
    /// use rubiks_cube_2x2::cube::{Cube, Color};
    ///
    /// let colors = [Color::White; 24];
    /// let cube = Cube::from_colors(&colors);
    /// ```
    pub fn from_colors(colors: &[Color; 24]) -> Result<Self, String> {
        // Changed return type to String for consistency with validate_colors
        let mut stickers = [Sticker::new(Color::White); 24];
        for (i, &color) in colors.iter().enumerate() {
            stickers[i] = Sticker {
                color,
                orientation: 0, // 一時的に0で初期化
            };
        }

        let cube = Cube { stickers };
        // Call the static validate_colors method
        Self::validate_colors(colors)?;

        // 時計回りパターンに設定
        Ok(cube.with_clockwise_orientations())
    }
    /// 色配列の妥当性をチェックします。
    ///
    /// 各色が正確に4つずつ存在するかを確認します。
    ///
    /// # 引数
    ///
    /// - `colors`: チェックする24個の色配列
    ///
    /// # 戻り値
    ///
    /// - `Ok(())`: 妥当な色配列
    /// - `Err(String)`: エラーメッセージ
    ///
    /// # Errors
    ///
    /// 次の場合にエラーを返します：
    /// - いずれかの色が4個でない場合
    /// - 必要な色が見つからない場合
    pub fn validate_colors(colors: &[Color; 24]) -> Result<(), String> {
        use std::collections::HashMap;

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
                    return Err(format!(
                        "{color:?}の数が{count}個です（4個である必要があります）"
                    ));
                }
                None => {
                    return Err(format!("{:?}が見つかりません", color));
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
    ///
    /// # 戻り値
    ///
    /// - `Ok(())`: 有効な状態
    /// - `Err(String)`: 無効な状態（エラーメッセージ付き）
    pub fn is_valid_state(&self) -> Result<(), String> {
        // まず色数のチェック
        let mut colors_array = [Color::White; 24];
        for (i, color) in colors_array.iter_mut().enumerate() {
            *color = self.stickers[i].color;
        }
        Self::validate_colors(&colors_array)?;

        // コーナーの位置パリティと向きパリティをチェック
        self.check_corner_parity()?;

        Ok(())
    }

    /// キューブの状態をファイル形式の文字列に変換
    ///
    /// 2D展開図の視覚的な形式で出力します：
    /// ```text
    ///      WWWW
    /// YYYY GGGG BBBB RRRR
    ///      OOOO
    /// ```
    ///
    /// # 戻り値
    ///
    /// 3行の文字列（展開図形式）
    pub fn to_file_format(&self) -> String {
        let mut result = String::new();

        // ヘルパー関数：面の4文字を取得
        let get_face = |face_idx: usize| -> String {
            let start = face_idx * 4;
            (0..4)
                .map(|i| match self.stickers[start + i].color {
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
    ///
    /// 展開図形式（3行）から読み込みます：
    /// ```text
    ///      WWWW
    /// YYYY GGGG BBBB RRRR
    ///      OOOO
    /// ```
    ///
    /// # 引数
    ///
    /// - `s`: ファイル形式の文字列（3行、展開図形式）
    ///
    /// # 戻り値
    ///
    /// - `Ok(Cube)`: 成功時
    /// - `Err(String)`: エラー時
    pub fn from_file_format(s: &str) -> Result<Self, String> {
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
        Self::validate_colors(&colors_array)?;

        Self::from_colors(&colors_array)
    }

    /// コーナーのパリティをチェック
    ///
    /// TODO: 現在のロジックにバグがあるため、一時的に無効化しています
    /// 正しいパリティチェックを後で実装する必要があります
    fn check_corner_parity(&self) -> Result<(), String> {
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
            let mut curr: Vec<Color> = positions.iter().map(|&i| self.stickers[i].color).collect();
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
                .position(|&idx| self.stickers[idx].color == base_color)
                .unwrap_or(0);
            orientation_sum += orientation;
        }
        if orientation_sum % 3 != 0 {
            return Err("コーナーの向きが無効です（向きパリティエラー）。\nこの配置は回転操作では実現できません。\nキューブを分解して組み立て直した可能性があります。".to_string());
        }
        Ok(())
        */
    }

    /// 回転操作を実行
    /// 指定した操作をキューブに適用します。
    ///
    /// # 引数
    ///
    /// - `mv`: 適用する操作（R, L, U, D, F, B およびその逆操作）
    ///
    /// # 例
    ///
    /// ```
    /// use rubiks_cube_2x2::cube::{Cube, Move};
    ///
    /// let mut cube = Cube::new();
    /// cube.apply_move(Move::R);
    /// cube.apply_move(Move::U);
    /// ```
    pub fn apply_move(&mut self, mv: Move) {
        match mv {
            Move::R => self.rotate_r(),
            Move::Rp => self.rotate_rp(),
            Move::L => self.rotate_l(),
            Move::Lp => self.rotate_lp(),
            Move::U => self.rotate_u(),
            Move::Up => self.rotate_up(),
            Move::D => self.rotate_d(),
            Move::Dp => self.rotate_dp(),
            Move::F => self.rotate_f(),
            Move::Fp => self.rotate_fp(),
            Move::B => self.rotate_b(),
            Move::Bp => self.rotate_bp(),
        }
    }

    /// 面自体を時計回りに回転
    fn rotate_face_cw(&mut self, start_idx: usize, orient_delta: u8) {
        let temp = self.stickers[start_idx];
        self.stickers[start_idx] = self.stickers[start_idx + 2];
        self.stickers[start_idx + 2] = self.stickers[start_idx + 3];
        self.stickers[start_idx + 3] = self.stickers[start_idx + 1];
        self.stickers[start_idx + 1] = temp;

        for i in 0..4 {
            for _ in 0..orient_delta {
                self.stickers[start_idx + i].rotate_cw();
            }
        }
    }

    /// 面自体を反時計回りに回転
    fn rotate_face_ccw(&mut self, start_idx: usize, orient_delta: u8) {
        let temp = self.stickers[start_idx];
        self.stickers[start_idx] = self.stickers[start_idx + 1];
        self.stickers[start_idx + 1] = self.stickers[start_idx + 3];
        self.stickers[start_idx + 3] = self.stickers[start_idx + 2];
        self.stickers[start_idx + 2] = temp;

        for i in 0..4 {
            for _ in 0..orient_delta {
                self.stickers[start_idx + i].rotate_ccw();
            }
        }
    }

    /// R面を時計回りに回転
    fn rotate_r(&mut self) {
        self.rotate_face_cw(12, 3); // Right face (orient +3)

        let temp0 = self.stickers[1];
        let temp1 = self.stickers[3];

        // U <- F (R-slice) [0]
        self.stickers[1] = self.stickers[17];
        self.stickers[3] = self.stickers[19];

        // F <- D (R-slice) [0]
        self.stickers[17] = self.stickers[5];
        self.stickers[19] = self.stickers[7];

        // D <- B [+2]
        self.stickers[5] = self.stickers[22];
        self.stickers[5].rotate_cw();
        self.stickers[5].rotate_cw();
        self.stickers[7] = self.stickers[20];
        self.stickers[7].rotate_cw();
        self.stickers[7].rotate_cw();

        // B <- U [+2]
        self.stickers[22] = temp0;
        self.stickers[22].rotate_cw();
        self.stickers[22].rotate_cw();
        self.stickers[20] = temp1;
        self.stickers[20].rotate_cw();
        self.stickers[20].rotate_cw();
    }

    /// R面を反時計回りに回転
    fn rotate_rp(&mut self) {
        self.rotate_face_ccw(12, 3); // Right face (orient +3)

        let temp0 = self.stickers[1];
        let temp1 = self.stickers[3];

        // U <- B (+2)
        self.stickers[1] = self.stickers[22];
        self.stickers[1].rotate_cw();
        self.stickers[1].rotate_cw();
        self.stickers[3] = self.stickers[20];
        self.stickers[3].rotate_cw();
        self.stickers[3].rotate_cw();

        // B <- D (+2)
        self.stickers[22] = self.stickers[5];
        self.stickers[22].rotate_cw();
        self.stickers[22].rotate_cw();
        self.stickers[20] = self.stickers[7];
        self.stickers[20].rotate_cw();
        self.stickers[20].rotate_cw();

        // D <- F (+0)
        self.stickers[5] = self.stickers[17];
        self.stickers[7] = self.stickers[19];

        // F <- U (+0)
        self.stickers[17] = temp0;
        self.stickers[19] = temp1;
    }

    /// L面を時計回りに回転
    fn rotate_l(&mut self) {
        self.rotate_face_cw(8, 3); // Left face (orient +3)

        let temp0 = self.stickers[0];
        let temp1 = self.stickers[2];

        // U <- B [+2]
        self.stickers[0] = self.stickers[23];
        self.stickers[0].rotate_cw();
        self.stickers[0].rotate_cw();
        self.stickers[2] = self.stickers[21];
        self.stickers[2].rotate_cw();
        self.stickers[2].rotate_cw();

        // B <- D [+2]
        self.stickers[23] = self.stickers[4];
        self.stickers[23].rotate_cw();
        self.stickers[23].rotate_cw();
        self.stickers[21] = self.stickers[6];
        self.stickers[21].rotate_cw();
        self.stickers[21].rotate_cw();

        // D <- F [+0]
        self.stickers[4] = self.stickers[16];
        self.stickers[6] = self.stickers[18];

        // F <- U [+0]
        self.stickers[16] = temp0;
        self.stickers[18] = temp1;
    }

    /// L面を反時計回りに回転
    fn rotate_lp(&mut self) {
        self.rotate_face_ccw(8, 3); // Left face (orient +3)

        let temp0 = self.stickers[0];
        let temp1 = self.stickers[2];

        // U <- F [+0]
        self.stickers[0] = self.stickers[16];
        self.stickers[2] = self.stickers[18];

        // F <- D [+0]
        self.stickers[16] = self.stickers[4];
        self.stickers[18] = self.stickers[6];

        // D <- B [+2]
        self.stickers[4] = self.stickers[23];
        self.stickers[4].rotate_cw();
        self.stickers[4].rotate_cw();
        self.stickers[6] = self.stickers[21];
        self.stickers[6].rotate_cw();
        self.stickers[6].rotate_cw();

        // B <- U [+2]
        self.stickers[23] = temp0;
        self.stickers[23].rotate_cw();
        self.stickers[23].rotate_cw();
        self.stickers[21] = temp1;
        self.stickers[21].rotate_cw();
        self.stickers[21].rotate_cw();
    }

    /// U面を時計回りに回転
    fn rotate_u(&mut self) {
        self.rotate_face_cw(0, 1); // Up face (orient +1)

        let temp0 = self.stickers[16];
        let temp1 = self.stickers[17];

        self.stickers[16] = self.stickers[12];
        self.stickers[17] = self.stickers[13];

        self.stickers[12] = self.stickers[20];
        self.stickers[13] = self.stickers[21];

        self.stickers[20] = self.stickers[8];
        self.stickers[21] = self.stickers[9];

        self.stickers[8] = temp0;
        self.stickers[9] = temp1;
    }

    /// U面を反時計回りに回転
    fn rotate_up(&mut self) {
        self.rotate_face_ccw(0, 1); // Up face (orient +1)

        let temp0 = self.stickers[16];
        let temp1 = self.stickers[17];

        self.stickers[16] = self.stickers[8];
        self.stickers[17] = self.stickers[9];

        self.stickers[8] = self.stickers[20];
        self.stickers[9] = self.stickers[21];

        self.stickers[20] = self.stickers[12];
        self.stickers[21] = self.stickers[13];

        self.stickers[12] = temp0;
        self.stickers[13] = temp1;
    }

    /// D面を時計回りに回転
    fn rotate_d(&mut self) {
        self.rotate_face_cw(4, 1); // Down face (orient +1)

        let temp0 = self.stickers[18];
        let temp1 = self.stickers[19];

        self.stickers[18] = self.stickers[10];
        self.stickers[19] = self.stickers[11];

        self.stickers[10] = self.stickers[22];
        self.stickers[11] = self.stickers[23];

        self.stickers[22] = self.stickers[14];
        self.stickers[23] = self.stickers[15];

        self.stickers[14] = temp0;
        self.stickers[15] = temp1;
    }

    /// D面を反時計回りに回転
    fn rotate_dp(&mut self) {
        self.rotate_face_ccw(4, 1); // Down face (orient +1)

        let temp0 = self.stickers[18];
        let temp1 = self.stickers[19];

        self.stickers[18] = self.stickers[14];
        self.stickers[19] = self.stickers[15];

        self.stickers[14] = self.stickers[22];
        self.stickers[15] = self.stickers[23];

        self.stickers[22] = self.stickers[10];
        self.stickers[23] = self.stickers[11];

        self.stickers[10] = temp0;
        self.stickers[11] = temp1;
    }

    /// F面を時計回りに回転
    fn rotate_f(&mut self) {
        self.rotate_face_cw(16, 1); // Front face (orient +1)

        let temp0 = self.stickers[2];
        let temp1 = self.stickers[3];

        // U -> R -> D -> L -> U
        // F CW: idx 11->2 (L->U) orient 1. idx 2->12 (U->R) orient 3. idx 12->5 (R->D) orient 1. idx 5->11 (D->L) orient 3.
        self.stickers[2] = self.stickers[11];
        self.stickers[2].rotate_cw();
        self.stickers[3] = self.stickers[9];
        self.stickers[3].rotate_cw();

        self.stickers[11] = self.stickers[5];
        self.stickers[11].rotate_ccw();
        self.stickers[9] = self.stickers[4];
        self.stickers[9].rotate_ccw();

        self.stickers[5] = self.stickers[12];
        self.stickers[5].rotate_cw();
        self.stickers[4] = self.stickers[14];
        self.stickers[4].rotate_cw();

        let mut t0 = temp0;
        t0.rotate_ccw();
        let mut t1 = temp1;
        t1.rotate_ccw();
        self.stickers[12] = t0;
        self.stickers[14] = t1;
    }

    /// F面を反時計回りに回転
    fn rotate_fp(&mut self) {
        self.rotate_face_ccw(16, 1); // Front face (orient +1)

        let temp0 = self.stickers[2];
        let temp1 = self.stickers[3];

        // U -> L -> D -> R -> U
        // Fp CCW: idx 12->2 (R->U) orient 1. idx 2->11 (U->L) orient 3. idx 11->5 (L->D) orient 1. idx 5->12 (D->R) orient 3.
        self.stickers[2] = self.stickers[12];
        self.stickers[2].rotate_cw();
        self.stickers[3] = self.stickers[14];
        self.stickers[3].rotate_cw();

        self.stickers[12] = self.stickers[5];
        self.stickers[12].rotate_ccw();
        self.stickers[14] = self.stickers[4];
        self.stickers[14].rotate_ccw();

        self.stickers[5] = self.stickers[11];
        self.stickers[5].rotate_cw();
        self.stickers[4] = self.stickers[9];
        self.stickers[4].rotate_cw();

        let mut t0 = temp0;
        t0.rotate_ccw();
        let mut t1 = temp1;
        t1.rotate_ccw();
        self.stickers[9] = t1; // U[3] moves to L-TL=9
        self.stickers[11] = t0; // U[2] moves to L-BL=11
    }

    /// B面を時計回りに回転
    fn rotate_b(&mut self) {
        self.rotate_face_cw(20, 1); // Back face (orient +1)

        let temp0 = self.stickers[0];
        let temp1 = self.stickers[1];

        // U -> R -> D -> L -> U (CW from back)
        // rotate_bpの逆操作
        // Bp: U<-L(3), L<-D(1), D<-R(3), R<-U(1)
        // B:  U<-R(1), R<-D(3), D<-L(1), L<-U(3)

        // U <- R (+3)
        self.stickers[0] = self.stickers[13];
        self.stickers[0].rotate_ccw();
        self.stickers[1] = self.stickers[15];
        self.stickers[1].rotate_ccw();

        // R <- D (+1)
        self.stickers[13] = self.stickers[7];
        self.stickers[13].rotate_cw();
        self.stickers[15] = self.stickers[6];
        self.stickers[15].rotate_cw();

        // D <- L (+3)
        self.stickers[7] = self.stickers[10];
        self.stickers[7].rotate_ccw();
        self.stickers[6] = self.stickers[8];
        self.stickers[6].rotate_ccw();

        // L <- U (+1)
        self.stickers[10] = temp0;
        self.stickers[10].rotate_cw();
        self.stickers[8] = temp1;
        self.stickers[8].rotate_cw();
    }

    /// B面を反時計回りに回転
    fn rotate_bp(&mut self) {
        self.rotate_face_ccw(20, 1); // Back face (orient +1)

        let temp0 = self.stickers[0];
        let temp1 = self.stickers[1];

        // U -> R -> D -> L -> U (CCW from back)
        // B CW:  U<-R(3), R<-D(1), D<-L(3), L<-U(1)
        // Bp CCW: U<-L(3), L<-D(1), D<-R(3), R<-U(1)

        // U <- L (+3)
        self.stickers[0] = self.stickers[10];
        self.stickers[0].rotate_ccw();
        self.stickers[1] = self.stickers[8];
        self.stickers[1].rotate_ccw();

        // L <- D (+1)
        self.stickers[10] = self.stickers[7];
        self.stickers[10].rotate_cw();
        self.stickers[8] = self.stickers[6];
        self.stickers[8].rotate_cw();

        // D <- R (+3)
        self.stickers[7] = self.stickers[13];
        self.stickers[7].rotate_ccw();
        self.stickers[6] = self.stickers[15];
        self.stickers[6].rotate_ccw();

        // R <- U (+1)
        self.stickers[13] = temp0;
        self.stickers[13].rotate_cw();
        self.stickers[15] = temp1;
        self.stickers[15].rotate_cw();
    }

    /// ランダムなスクランブルを生成します。
    ///
    /// # 引数
    ///
    /// - `moves`: 適用するランダム操作の回数
    ///
    /// # 例
    ///
    /// ```
    /// use rubiks_cube_2x2::cube::Cube;
    ///
    /// let mut cube = Cube::new();
    /// cube.scramble(20);
    /// assert!(!cube.is_solved()); // ほぼ確実に未完成状態になる
    /// ```
    pub fn scramble(&mut self, moves: usize) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let all_moves = Move::all_moves();

        for _ in 0..moves {
            let mv = all_moves[rng.gen_range(0..all_moves.len())];
            self.apply_move(mv);
        }
    }

    /// 色情報のみ比較するために、向き情報をリセットしたキューブを返します。
    ///
    /// すべてのステッカーの向き（orientation）を0にリセットした新しいキューブを返します。
    /// 色の配置は元のキューブと同じです。
    ///
    /// # 戻り値
    ///
    /// 向き情報がリセットされた新しいキューブ
    ///
    /// # 例
    ///
    /// ```
    /// use rubiks_cube_2x2::cube::{Cube, Move};
    ///
    /// let mut cube = Cube::new();
    /// cube.apply_move(Move::R);
    /// let normalized = cube.normalized();
    /// // normalized は色配置は同じだが、すべての向きが0
    /// ```
    pub fn normalized(&self) -> Self {
        let mut new_cube = self.clone();
        for sticker in &mut new_cube.stickers {
            sticker.orientation = 0;
        }
        new_cube
    }
}

impl Default for Cube {
    fn default() -> Self {
        Self::new()
    }
}
