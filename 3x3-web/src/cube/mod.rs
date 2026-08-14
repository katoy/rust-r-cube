pub mod enums;
pub mod io;
pub mod piece;
pub mod rotation;
pub mod validation;

pub use self::enums::{Color, Face, Move, Sticker, NUM_STICKERS, STICKERS_PER_FACE};

/// 完成状態における標準的なステッカーの向きパターン。
pub const CLOCKWISE_ORIENTATION_PATTERN: [u8; 9] = [0; 9];

/// 3x3 ルービックキューブを表す構造体。
///
/// 54枚のステッカー（[`Sticker`]）をフラットな配列として保持します。
/// 内部構造は面の順序と各面内のインデックスによって定義されます。
///
/// # ⚠️ 開発上の注意 (二重表現の同期)
///
/// 本構造体は `stickers`（表示/Kociemba座標用）と `pieces`（3D物理幾何用）の2層で状態を管理しています。
/// 外部から `pieces` または `stickers` の中身を直接書き換える場合は、状態の不整合を防ぐため、
/// 必ず直後に `sync_stickers()` を呼び出して同期させてください。
#[derive(Debug, Clone)]
pub struct Cube {
    pub stickers: [Sticker; 54],
    pub pieces: [piece::Cubie; 26],
}

impl PartialEq for Cube {
    fn eq(&self, other: &Self) -> bool {
        self.stickers == other.stickers
    }
}

impl Eq for Cube {}

impl std::hash::Hash for Cube {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.stickers.hash(state);
    }
}

impl Cube {
    #[must_use]
    pub fn new() -> Self {
        let stickers = [Sticker::new(Color::White); NUM_STICKERS];
        let pieces = piece::get_initial_pieces();
        let mut cube = Self { stickers, pieces };
        cube.sync_stickers();
        cube
    }

    /// ピースの状態からステッカー配列を更新します。
    pub fn sync_stickers(&mut self) {
        for p in &self.pieces {
            p.project_to_stickers(&mut self.stickers);
        }
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

        let mut cube = Cube {
            stickers,
            pieces: piece::get_initial_pieces(),
        };
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
        self.assert_stickers_synced();
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
    /// 各ステッカーの色情報から、ピースの位置と回転を物理的に正しい状態に復元します。
    pub fn restore_orientation_instantly(&mut self) -> crate::error::Result<()> {
        // 色の妥当性チェック
        let mut colors_array = [Color::White; 54];
        for (i, color) in colors_array.iter_mut().enumerate() {
            *color = self.stickers[i].color;
        }
        Self::validate_colors(&colors_array)?;

        use crate::cube::validation::{CENTER_STICKERS, CORNER_STICKERS, EDGE_STICKERS};

        // 24通りの完成状態（全方位）を取得
        let solved_states = crate::solver::get_solved_states();
        let mut new_pieces_vec = Vec::new();

        // 1. まずセンターの位置関係から、現在の「全体の方位」を特定する。
        //    センターのみが完全に一致する solved_state を探す。
        let mut preferred_state_idx = 0;
        let mut found_preferred = false;
        for (idx, solved) in solved_states.iter().enumerate() {
            let mut centers_match = true;
            for &c_idx in &CENTER_STICKERS {
                if self.stickers[c_idx].color != solved.stickers[c_idx].color {
                    centers_match = false;
                    break;
                }
            }
            if centers_match {
                preferred_state_idx = idx;
                found_preferred = true;
                break;
            }
        }

        if !found_preferred {
            return Err(crate::error::CubeError::InvalidState(
                "中心ピースの色配置が不正です".to_string(),
            ));
        }
        let preferred_state = &solved_states[preferred_state_idx];

        if !preferred_state.is_solved_with_orientation() {
            return Err(crate::error::CubeError::InvalidState(
                "内部解決状態の不整合".to_string(),
            ));
        }

        // 2. 特定された preferred_state に基づいてセンターピースを復元
        for &idx in &CENTER_STICKERS {
            self.restore_piece_at_slot(
                &[idx],
                std::slice::from_ref(preferred_state),
                &mut new_pieces_vec,
            )?;
        }

        // 3. コーナーとエッジを復元（これらは色情報から物理的に一義的に決まる）
        // コーナー (8個)
        for &slot_indices in &CORNER_STICKERS {
            self.restore_piece_at_slot(&slot_indices, solved_states, &mut new_pieces_vec)?;
        }

        // エッジ (12個)
        for &slot_indices in &EDGE_STICKERS {
            self.restore_piece_at_slot(&slot_indices, solved_states, &mut new_pieces_vec)?;
        }

        if new_pieces_vec.len() != 26 {
            return Err(crate::error::CubeError::InvalidState(format!(
                "ピースの復元に失敗しました（計{}個）",
                new_pieces_vec.len()
            )));
        }

        // pieces 配列を更新
        self.pieces = new_pieces_vec.try_into().map_err(|_| {
            crate::error::CubeError::InvalidState("ピース配列の変換に失敗しました".to_string())
        })?;

        // 最後に pieces の状態を stickers (特に orientation) に反映
        self.sync_stickers();

        // 最終的なチェック
        self.is_valid_state()
    }

    /// 特定のスロットにあるピースを、 solved_states を利用して復元します。
    fn restore_piece_at_slot(
        &self,
        slot_indices: &[usize],
        solved_states: &[Cube],
        target_pieces: &mut Vec<piece::Cubie>,
    ) -> crate::error::Result<()> {
        let current_colors: Vec<Color> = slot_indices
            .iter()
            .map(|&idx| self.stickers[idx].color)
            .collect();

        for solved in solved_states {
            let solved_colors: Vec<Color> = slot_indices
                .iter()
                .map(|&idx| solved.stickers[idx].color)
                .collect();

            if current_colors == solved_colors {
                // どのステッカーも同じピースに属しているはずなので、slot_indices[0] を使ってピースを特定する
                let test_idx = slot_indices[0];

                // solved.pieces を探索して、test_idx にステッカーを投影するピースを探す
                for p in &solved.pieces {
                    let mut temp_stickers = [Sticker::new(Color::Gray); NUM_STICKERS];
                    p.project_to_stickers(&mut temp_stickers);
                    if temp_stickers[test_idx].color != Color::Gray {
                        // 見つけた！(位置も回転も solved Cube のものが現在のスロットの状態を正しく表している)
                        target_pieces.push(p.clone());
                        return Ok(());
                    }
                }
            }
        }

        Err(crate::error::CubeError::InvalidState(format!(
            "指定された色の組み合わせを持つピースが見つかりません: {:?}",
            current_colors
        )))
    }

    /// ソルバーで見つかった解を利用して向きを復元します（現在は内部で `restore_orientation_instantly` を呼び出します）。
    pub fn apply_orientation_solution(
        &mut self,
        _solution: &crate::solver::Solution,
    ) -> crate::error::Result<()> {
        self.restore_orientation_instantly()
    }

    /// ステッカー配列に設定された中央方位（orientation）情報を、内部のピース状態に強制的に同期させます。
    ///
    /// 通常、`apply_move` を呼ぶとピースの状態からステッカーが再生成されますが、
    /// テストなどでステッカーの方位を直接書き換えた場合、このメソッドを呼ぶことで
    /// その方位を物理的なピースの回転状態として定着させることができます。
    pub fn force_sync_orientation_to_pieces(&mut self) {
        use crate::cube::validation::CENTER_STICKERS;

        for &idx in &CENTER_STICKERS {
            let ori = self.stickers[idx].orientation;
            if ori == 0 {
                continue;
            }

            // 該当するセンターピースを見つける
            let target_color = self.stickers[idx].color;
            for p in &mut self.pieces {
                if p.piece_type == crate::cube::piece::PieceType::Center
                    && p.stickers[0].color == target_color
                {
                    // センターピースの向きを更新する
                    // calculate_orientation の逆関数的な動作が必要。
                    // 0 -> 単位行列
                    // 1 -> 軸を中心に時計回り 90度
                    // 2 -> 180度
                    // 3 -> 反時計回り 90度

                    let normal = p.stickers[0].initial_normal;
                    let angle = match ori {
                        1 => -std::f32::consts::FRAC_PI_2, // CW
                        2 => std::f32::consts::PI,         // 180
                        3 => std::f32::consts::FRAC_PI_2,  // CCW
                        _ => 0.0,
                    };

                    // 現在の回転をリセットして、指定された方位に設定する
                    // (注意: このメソッドは「色が揃っている解決状態」付近での使用を想定している)
                    p.current_rot = glam::Mat4::from_axis_angle(normal, angle);
                    break;
                }
            }
        }
        // pieces を更新したので、再度 stickers に反映（整合性確保）
        self.sync_stickers();
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

    /// デバッグビルド用：`pieces` の状態と `stickers` の状態が同期しているかを検証します。
    /// 同期漏れバグの早期発見のためのアサーションです。
    pub fn assert_stickers_synced(&self) {
        #[cfg(debug_assertions)]
        {
            let mut temp_stickers = [Sticker::new(Color::Gray); NUM_STICKERS];
            for p in &self.pieces {
                p.project_to_stickers(&mut temp_stickers);
            }
            for i in 0..54 {
                if temp_stickers[i].color != Color::Gray {
                    assert_eq!(
                        self.stickers[i].color,
                        temp_stickers[i].color,
                        "キューブ状態の同期エラー: インデックス {} において、stickersの色 ({:?}) と pieces から投影された色 ({:?}) が不整合です。状態変更後に sync_stickers() が呼ばれているか確認してください。",
                        i, self.stickers[i].color, temp_stickers[i].color
                    );
                }
            }
        }
    }
}

impl Default for Cube {
    fn default() -> Self {
        Self::new()
    }
}
