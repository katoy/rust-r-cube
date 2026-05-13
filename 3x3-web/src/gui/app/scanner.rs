use super::CubeApp;
use super::InputState;
use crate::cube::{Color, Cube};

impl CubeApp {
    /// スキャンモード開始
    pub fn start_scanning_mode(&mut self) {
        self.input_state = InputState::Scanning { face_index: 0 };
        self.input_buffer = [None; 54];
        self.selected_input_color = Color::White;
        self.input_error_message.clear();
    }

    /// スキャンモードをキャンセル
    pub fn cancel_scanning_mode(&mut self) {
        self.input_state = InputState::Normal;
        self.input_buffer = [None; 54];
        self.input_error_message.clear();
    }

    /// 次の面へ進む
    pub fn next_face(&mut self) {
        if let InputState::Scanning { face_index } = self.input_state {
            if face_index < 5 {
                self.input_state = InputState::Scanning {
                    face_index: face_index + 1,
                };
            }
        }
    }

    /// 前の面へ戻る
    pub fn prev_face(&mut self) {
        if let InputState::Scanning { face_index } = self.input_state {
            if face_index > 0 {
                self.input_state = InputState::Scanning {
                    face_index: face_index - 1,
                };
            }
        }
    }

    /// 現在の面のステッカーに色を設定
    /// position: 面内の位置 0-3 (左上、右上、左下、右下)
    pub fn set_current_face_sticker(&mut self, position: usize, color: Color) {
        if let InputState::Scanning { face_index } = self.input_state {
            let global_index = face_index * 9 + position;
            if global_index < 54 {
                self.input_buffer[global_index] = Some(color);
            }
        }
    }

    /// 現在の面の指定位置のステッカー色を取得
    pub fn get_current_face_sticker(&self, position: usize) -> Option<Color> {
        if let InputState::Scanning { face_index } = self.input_state {
            let global_index = face_index * 9 + position;
            if global_index < 54 {
                return self.input_buffer[global_index];
            }
        }
        None
    }

    /// 現在の面の名前を取得
    pub fn get_current_face_name(&self) -> &str {
        if let InputState::Scanning { face_index } = self.input_state {
            match face_index {
                0 => "Up (上面)",
                1 => "Down (下面)",
                2 => "Left (左面)",
                3 => "Right (右面)",
                4 => "Front (前面)",
                5 => "Back (背面)",
                _ => "不明",
            }
        } else {
            "不明"
        }
    }

    /// 現在の面が全て入力済みかチェック
    pub fn is_current_face_complete(&self) -> bool {
        if let InputState::Scanning { face_index } = self.input_state {
            let start = face_index * 9;
            let end = start + 9;
            return self.input_buffer[start..end].iter().all(|c| c.is_some());
        }
        false
    }

    /// スキャン完了（キューブに反映）
    pub fn finish_scanning(&mut self) {
        // 全てのステッカーが入力されているかチェック
        if self.input_buffer.iter().any(|c| c.is_none()) {
            self.input_error_message = "全ての面を入力してください".to_string();
            return;
        }

        // Option<Color>をColorに変換
        let colors: [Color; 54] = self
            .input_buffer
            .iter()
            .map(|c| c.expect("全ての色が入力されています"))
            .collect::<Vec<_>>()
            .try_into()
            .expect("配列は54要素です");

        // 妥当性チェック
        if let Err(e) = Cube::validate_colors(&colors) {
            self.input_error_message = e.to_string();
            return;
        }

        // キューブに反映
        let new_cube = match Cube::from_colors(&colors) {
            Ok(cube) => cube,
            Err(e) => {
                self.input_error_message = format!("キューブの作成に失敗: {}", e);
                return;
            }
        };

        // パリティチェック（物理的に可能な配置かチェック）
        if !self.skip_parity_check {
            if let Err(e) = new_cube.is_valid_state() {
                self.input_error_message = format!("無効なキューブ状態: {}", e);
                return;
            }
        }

        self.cube = new_cube;
        self.input_state = InputState::Normal;
        self.input_buffer = [None; 54];
        self.input_error_message.clear();

        // 向きの自動復元（即時）
        if let Err(e) = self.cube.restore_orientation_instantly() {
            self.input_error_message = format!("警告: 向きの復元に失敗しました ({})", e);
        }

        // 解法やアニメーションをクリア
        self.solution = None;
        self.solution_text.clear();
        self.animation = None;
        self.move_queue.clear();
    }
}
