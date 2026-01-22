pub mod enums;
pub mod io;
pub mod rotation;
pub mod validation;

pub use self::enums::{Color, Face, Move, Sticker, NUM_STICKERS, STICKERS_PER_FACE};

/// 完成状態における標準的なステッカーの向きパターン。
pub const CLOCKWISE_ORIENTATION_PATTERN: [u8; 9] = [0; 9];

/// 3x3 ルービックキューブを表す構造体。
///
/// 54枚のステッカー（[`Sticker`]）をフラットな配列として保持します。
/// 内部構造は面の順序と各面内のインデックスによって定義されます。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cube {
    /// 全ステッカーの配列（総数54）。
    ///
    /// インデックスの割り当て:
    /// - 0-8:   上面 (Up)
    /// - 9-17:  下面 (Down)
    /// - 18-26: 左面 (Left)
    /// - 27-35: 右面 (Right)
    /// - 36-44: 前面 (Front)
    /// - 45-53: 背面 (Back)
    pub stickers: [Sticker; 54],
}

impl Cube {
    /// 完成した状態の新しいキューブを作成します。
    ///
    /// 各面は標準的な配色（白・黄・緑・青・赤・橙）で塗り分けられ、
    /// ステッカーの向きは標準的な時計回りパターン `[1, 2, 0, 3]` に設定されます。
    ///
    /// # 例
    ///
    /// ```
    /// use rubiks_cube_3x3::cube::Cube;
    /// let cube = Cube::new();
    /// assert!(cube.is_solved_with_orientation());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        let mut stickers = [Sticker::new(Color::White); NUM_STICKERS];

        let faces = [
            (Color::White, Face::Up),
            (Color::Yellow, Face::Down),
            (Color::Orange, Face::Left),
            (Color::Red, Face::Right),
            (Color::Green, Face::Front),
            (Color::Blue, Face::Back),
        ];

        for (color, face) in faces {
            let start = face.start_index();
            for i in 0..STICKERS_PER_FACE {
                stickers[start + i] = Sticker {
                    color,
                    orientation: CLOCKWISE_ORIENTATION_PATTERN[i],
                };
            }
        }

        Self { stickers }
    }

    /// 各面のステッカーの色がすべて一致しているか（完成しているか）を判定します。
    ///
    /// ステッカー自体の向き（矢印などの方向）は無視されます。
    ///
    /// # 戻り値
    ///
    /// すべての面が単色であれば `true` を返します。
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

    /// 色が揃っており、かつステッカーの向きも初期状態と同じであるかを判定します。
    ///
    /// # 戻り値
    ///
    /// 色と向きの両方が初期状態と一致していれば `true` を返します。
    #[must_use]
    pub fn is_solved_with_orientation(&self) -> bool {
        if !self.is_solved() {
            return false;
        }
        for face in Face::all() {
            let start = face.start_index();
            for (i, &expected_orientation) in CLOCKWISE_ORIENTATION_PATTERN.iter().enumerate() {
                if self.stickers[start + i].orientation != expected_orientation {
                    return false;
                }
            }
        }
        true
    }

    /// 指定した絶対インデックス（0-23）のステッカーを取得します。
    #[must_use]
    pub fn get_sticker(&self, index: usize) -> Sticker {
        self.stickers[index]
    }

    /// 指定した絶対インデックスのステッカーの色を設定します。
    ///
    /// 色を変更すると、そのステッカーの向き（orientation）は 0 にリセットされます。
    pub fn set_sticker_color(&mut self, index: usize, color: Color) {
        self.stickers[index].color = color;
        // 向きはリセット（手動入力時は向きを0にする）
        self.stickers[index].orientation = 0;
    }

    /// 各ステッカーの色を維持したまま、向き情報のみを標準的な時計回りパターンにリセットした新しいキューブを返します。
    ///
    /// ソルバーで色のみを解いた後に、物理的に正しい向きを再設定する場合などに使用されます。
    #[must_use]
    pub fn with_clockwise_orientations(&self) -> Self {
        let mut new_cube = self.clone();

        for face in Face::all() {
            let face_start = face.start_index();
            for (offset, &pattern) in CLOCKWISE_ORIENTATION_PATTERN.iter().enumerate() {
                let idx = face_start + offset;
                new_cube.stickers[idx].orientation = pattern;
            }
        }

        new_cube
    }

    /// 54個の色配列から新しいキューブを作成します（物理的な向きを自動復元）。
    ///
    /// 色の配置が物理的に可能でない場合はエラーを返します。
    pub fn from_colors(colors: &[Color; 54]) -> crate::error::Result<Self> {
        let mut stickers = [Sticker::new(Color::White); 54];
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

    /// 指定された色配列（54要素）がキューブとして妥当であるかを検証します。
    pub fn validate_colors(colors: &[Color; 54]) -> crate::error::Result<()> {
        validation::validate_colors(colors)
    }

    /// キューブの現在の状態（色と向きの組み合わせ）が、物理的に到達可能な有効な状態であるかを判定します。
    pub fn is_valid_state(&self) -> crate::error::Result<()> {
        validation::is_valid_state(self)
    }

    /// キューブの現在の内容を、ファイル保存用のテキスト形式に変換します。
    pub fn to_file_format(&self) -> String {
        io::to_file_format(self)
    }

    /// ファイル保存形式の文字列をパースして、新しいキューブを作成します。
    pub fn from_file_format(s: &str) -> crate::error::Result<Self> {
        io::from_file_format(s)
    }

    /// 回転操作を適用します。
    pub fn apply_move(&mut self, mv: Move) {
        rotation::apply_move(self, mv);
    }

    /// 指定回数のランダムな回転操作を適用します。
    pub fn scramble(&mut self, moves: usize) {
        rotation::scramble(self, moves);
    }

    /// 現在の色配置に基づいて、物理的に正しいステッカー向（twist）を瞬時に復元します。
    ///
    /// 各コーナーピースの3色の配置を、24通りの完成状態のいずれかと照合することで、
    /// 探索を行わずに正しい向きを特定します。
    ///
    /// # エラー
    ///
    /// コーナーピースの色の組み合わせが物理的に存在しない（例：白と黄が隣接している）
    /// 場合などは `CubeError::InvalidState` を返します。
    pub fn restore_orientation_instantly(&mut self) -> crate::error::Result<()> {
        // 色の妥当性チェック
        let mut colors_array = [Color::White; 54];
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

    /// ソルバーで見つかった解を利用して向きを復元します（現在は内部で `restore_orientation_instantly` を呼び出します）。
    pub fn apply_orientation_solution(
        &mut self,
        _solution: &crate::solver::Solution,
    ) -> crate::error::Result<()> {
        self.restore_orientation_instantly()
    }

    /// 各ステッカーの向き情報をリセット（0に設定）したキューブの複製を返します。
    ///
    /// ハッシュマップやセットで、向きによらず「色の配置」のみで状態を管理したい場合に使用します。
    #[must_use]
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
