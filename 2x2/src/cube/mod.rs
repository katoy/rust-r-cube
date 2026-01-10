pub mod enums;
pub mod io;
pub mod rotation;
pub mod validation;

pub use self::enums::{Color, Face, Move, Sticker};

/// 2x2 ルービックキューブ
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cube {
    /// 各面のステッカー（各面4つ）
    /// インデックス: 0-3 (Up), 4-7 (Down), 8-11 (Left), 12-15 (Right), 16-19 (Front), 20-23 (Back)
    pub(crate) stickers: [Sticker; 24],
}

impl Cube {
    /// 完成状態のキューブを作成します。
    #[must_use]
    pub fn new() -> Self {
        let mut stickers = [Sticker::new(Color::White); 24];
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
    #[must_use]
    pub fn get_sticker(&self, index: usize) -> Sticker {
        self.stickers[index]
    }

    /// 指定したインデックスのステッカーの色を設定します。
    pub fn set_sticker_color(&mut self, index: usize, color: Color) {
        self.stickers[index].color = color;
        // 向きはリセット（手動入力時は向きを0にする）
        self.stickers[index].orientation = 0;
    }

    /// 色を保持したまま、全てのステッカーのorientationを時計回りパターンにリセットします。
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
    pub fn from_colors(colors: &[Color; 24]) -> Result<Self, String> {
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
    pub fn validate_colors(colors: &[Color; 24]) -> Result<(), String> {
        validation::validate_colors(colors)
    }

    /// キューブの状態が有効かどうかを判定
    pub fn is_valid_state(&self) -> Result<(), String> {
        validation::is_valid_state(self)
    }

    /// キューブの状態をファイル形式の文字列に変換
    pub fn to_file_format(&self) -> String {
        io::to_file_format(self)
    }

    /// ファイル形式の文字列からキューブを作成
    pub fn from_file_format(s: &str) -> Result<Self, String> {
        io::from_file_format(s)
    }

    /// 回転操作を実行
    pub fn apply_move(&mut self, mv: Move) {
        rotation::apply_move(self, mv);
    }

    /// ランダムなスクランブルを生成します。
    pub fn scramble(&mut self, moves: usize) {
        rotation::scramble(self, moves);
    }

    /// ソリューション（向き無視で解いたもの）を使って、現在のキューブの正しい向きを復元します。
    pub fn apply_orientation_solution(
        &mut self,
        solution: &crate::solver::Solution,
    ) -> Result<(), String> {
        // Solved状態から逆操作を適用して、現在の色配置を再現
        let mut reference_cube = Cube::new();
        // 解の手順: Current -> Solved
        // 逆手順: Solved -> Current
        for mv in solution.moves.iter().rev() {
            reference_cube.apply_move(mv.inverse());
        }

        // Orientationのみコピー
        for (i, sticker) in self.stickers.iter_mut().enumerate() {
            let ref_sticker = reference_cube.stickers[i];
            if sticker.color != ref_sticker.color {
                // これが起きるのはソルバーにバグがあるか、スレッド競合等の異常事態
                return Err(format!(
                    "内部エラー: ソルバーによる復元で色が不一致です。Index: {}",
                    i
                ));
            }
            sticker.orientation = ref_sticker.orientation;
        }

        Ok(())
    }

    /// 色情報のみ比較するために、向き情報をリセットしたキューブを返します。
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

#[cfg(test)]
mod tests_coverage;
