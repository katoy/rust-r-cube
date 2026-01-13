pub mod enums;
pub mod io;
pub mod rotation;
pub mod validation;

pub use self::enums::{Color, Face, Move, Sticker, NUM_STICKERS, STICKERS_PER_FACE};

/// 2x2 ルービックキューブ
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cube {
    /// 各面のステッカー（各面4つ）
    /// インデックス: 0-3 (Up), 4-7 (Down), 8-11 (Left), 12-15 (Right), 16-19 (Front), 20-23 (Back)
    pub stickers: [Sticker; 24],
}

impl Cube {
    /// 完成状態のキューブを作成します。
    #[must_use]
    pub fn new() -> Self {
        let mut stickers = [Sticker::new(Color::White); NUM_STICKERS];
        let clockwise_pattern = [1, 2, 0, 3];

        let faces = [
            (Color::White, Face::Up),
            (Color::Yellow, Face::Down),
            (Color::Green, Face::Left),
            (Color::Blue, Face::Right),
            (Color::Red, Face::Front),
            (Color::Orange, Face::Back),
        ];

        for (color, face) in faces {
            let start = face.start_index();
            for i in 0..STICKERS_PER_FACE {
                stickers[start + i] = Sticker {
                    color,
                    orientation: clockwise_pattern[i],
                };
            }
        }

        Self { stickers }
    }

    /// キューブが完成しているか判定します（色のみ、向きは無視）。
    #[must_use]
    pub fn is_solved(&self) -> bool {
        for face in Face::all() {
            let face_start = face.start_index();
            let color = self.stickers[face_start].color;
            for i in 1..STICKERS_PER_FACE {
                if self.stickers[face_start + i].color != color {
                    return false;
                }
            }
        }
        true
    }

    /// キューブが完成しているか判定します（色と向きの両方）。
    #[must_use]
    pub fn is_solved_with_orientation(&self) -> bool {
        if !self.is_solved() {
            return false;
        }
        let clockwise_pattern = [1, 2, 0, 3];
        for face in Face::all() {
            let start = face.start_index();
            for (i, &expected_orientation) in clockwise_pattern.iter().enumerate() {
                if self.stickers[start + i].orientation != expected_orientation {
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

        for face in Face::all() {
            let face_start = face.start_index();
            for (offset, &pattern) in clockwise_pattern.iter().enumerate() {
                let idx = face_start + offset;
                new_cube.stickers[idx].orientation = pattern;
            }
        }

        new_cube
    }

    /// 24個の色配列から新しいキューブを作成します。
    pub fn from_colors(colors: &[Color; 24]) -> crate::error::Result<Self> {
        let mut stickers = [Sticker::new(Color::White); 24];
        for (i, &color) in colors.iter().enumerate() {
            stickers[i] = Sticker {
                color,
                orientation: 0, // 一時的に0で初期化
            };
        }

        let mut cube = Cube { stickers };
        // Call the static validate_colors method
        Self::validate_colors(colors)?;

        // 3色から物理的に正しい向きを復元
        cube.restore_orientation_instantly()?;

        Ok(cube)
    }

    /// 色配列の妥当性をチェックします。
    pub fn validate_colors(colors: &[Color; 24]) -> crate::error::Result<()> {
        validation::validate_colors(colors)
    }

    /// キューブの状態が有効かどうかを判定
    pub fn is_valid_state(&self) -> crate::error::Result<()> {
        validation::is_valid_state(self)
    }

    /// キューブの状態をファイル形式の文字列に変換
    pub fn to_file_format(&self) -> String {
        io::to_file_format(self)
    }

    /// ファイル形式の文字列からキューブを作成
    pub fn from_file_format(s: &str) -> crate::error::Result<Self> {
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

    /// 探索を使わずに、現在の色配置から物理的に正しい向きを瞬時に復元します。
    /// 1. UFL スロットの色配置に基づいて、24通りの完成状態から現在の「座標系（方位）」を1つ特定します。
    /// 2. 特定された方位全体の完成状態をテンプレートとして、現在の各ピースの向き（twist）を特定し、
    ///    物理的に正しい方向を設定します。
    pub fn restore_orientation_instantly(&mut self) -> crate::error::Result<()> {
        // 色の妥当性チェック
        let mut colors_array = [Color::White; 24];
        for (i, color) in colors_array.iter_mut().enumerate() {
            *color = self.stickers[i].color;
        }
        Self::validate_colors(&colors_array)?;

        use crate::cube::validation::CORNER_STICKERS;

        // 24通りの完成状態（すべて [1, 2, 0, 3] パターンを持つ）を取得
        let solved_states = crate::solver::get_solved_states();

        // 各スロットに対して、24通りの完成状態のいずれかから「同じ色の配置（twist一致）」を持つものを探し、
        // その向きをコピーする。
        // rotation.rs の物理整合性により、どの解決方位から見つけても結果は一貫する。
        for &slot_indices in &CORNER_STICKERS {
            let current_colors = [
                self.stickers[slot_indices[0]].color,
                self.stickers[slot_indices[1]].color,
                self.stickers[slot_indices[2]].color,
            ];

            let mut found = false;
            for solved in solved_states {
                let solved_colors = [
                    solved.stickers[slot_indices[0]].color,
                    solved.stickers[slot_indices[1]].color,
                    solved.stickers[slot_indices[2]].color,
                ];

                // 色の並び（twist）まで完全一致
                if current_colors == solved_colors {
                    for &idx in &slot_indices {
                        self.stickers[idx].orientation = solved.stickers[idx].orientation;
                    }
                    found = true;
                    break;
                }
            }

            if !found {
                return Err(crate::error::CubeError::InvalidState(format!(
                    "不正な色のピース配置: {:?}",
                    current_colors
                )));
            }
        }

        // 最終的なチェック
        self.is_valid_state()
    }

    /// ソリューション（向き無視で解いたもの）を使って、現在のキューブの正しい向きを復元します。
    /// 旧方式のソルバーベースの復元（非推奨）ですが、互換性のために残し、内部で即時復元を呼び出すように変更します。
    pub fn apply_orientation_solution(
        &mut self,
        _solution: &crate::solver::Solution,
    ) -> crate::error::Result<()> {
        self.restore_orientation_instantly()
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
