use super::CubeApp;
use super::{InputState, MAX_FILE_SIZE};
use crate::cube::{Color, Cube};

impl CubeApp {
    /// キューブの状態をファイルに保存
    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        // 現在表示されているキューブ（スキャン中なら入力バッファベース）を保存
        let content = self.display_cube().to_file_format();
        std::fs::write(path, content).map_err(|e| format!("ファイルの保存に失敗しました: {}", e))
    }

    /// ファイルからキューブの状態を読み込み
    pub fn load_from_file(&mut self, path: &str) -> Result<String, String> {
        let metadata = std::fs::metadata(path)
            .map_err(|e| format!("ファイル情報の取得に失敗しました: {}", e))?;

        if metadata.len() > MAX_FILE_SIZE as u64 {
            return Err(format!(
                "ファイルサイズが大きすぎます (最大: {} MB)",
                MAX_FILE_SIZE / (1024 * 1024)
            ));
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("ファイルの読み込みに失敗しました: {}", e))?;

        self.load_from_content(&content)
    }

    /// 文字列（ファイル内容）からキューブの状態を読み込み
    pub fn load_from_content(&mut self, content: &str) -> Result<String, String> {
        let loaded_cube = Cube::from_file_format(content).map_err(|e| e.to_string())?;

        let mut warning = String::new();

        // 読み込んだキューブに Gray (未設定) が含まれているかチェック
        let has_gray = loaded_cube.stickers.iter().any(|s| s.color == Color::Gray);

        if has_gray {
            // スキャンモードとして復元
            self.input_state = InputState::Scanning { face_index: 0 };
            for (i, sticker) in loaded_cube.stickers.iter().enumerate() {
                self.input_buffer[i] = if sticker.color == Color::Gray {
                    None
                } else {
                    Some(sticker.color)
                };
            }
            self.cube = Cube::new(); // 内部状態はリセット
            warning = "スキャン途中の状態を読み込みました".to_string();
        } else {
            // 通常のキューブとして復元
            let mut new_cube = loaded_cube;

            // 全ての向きが0（旧形式またはリセット直後）の場合のみ、向きの自動復元を試みる
            let all_zero_orientation = new_cube.stickers.iter().all(|s| s.orientation == 0);
            if all_zero_orientation {
                if let Err(e) = new_cube.restore_orientation_instantly() {
                    warning = format!("警告: 向きの復元に失敗しました ({})", e);
                }
            }

            // パリティチェック（skip_parity_checkフラグで制御）
            if !self.skip_parity_check {
                if let Err(e) = new_cube.is_valid_state() {
                    let parity_warning = format!("警告: 無効なキューブ状態です ({})", e);
                    warning = if warning.is_empty() {
                        parity_warning
                    } else {
                        format!("{}\n{}", warning, parity_warning)
                    };
                }
            }

            self.cube = new_cube;
            self.input_state = InputState::Normal;
            self.input_buffer = [None; 54];
        }

        self.solution = None;
        self.solution_text.clear();
        self.animation = None;
        self.move_queue.clear();
        self.input_error_message.clear();

        Ok(warning)
    }

    /// 保存ダイアログを表示して保存
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save_with_dialog(&mut self) {
        let task = rfd::FileDialog::new()
            .set_directory(".")
            .add_filter("Text files", &["txt"])
            .set_file_name("cube_state.txt")
            .save_file();

        if let Some(path) = task {
            let path_str = path.to_string_lossy();
            match self.save_to_file(&path_str) {
                Ok(_) => {
                    self.input_error_message = format!(
                        "保存しました: {}",
                        path.file_name().unwrap_or_default().to_string_lossy()
                    );
                }
                Err(e) => {
                    self.input_error_message = format!("保存エラー: {}", e);
                }
            }
        }
    }

    /// Web版: ブラウザのダウンロード機能でファイルを保存
    #[cfg(target_arch = "wasm32")]
    pub fn save_with_dialog(&mut self) {
        use wasm_bindgen::JsCast;
        use web_sys::{Blob, HtmlAnchorElement, Url};

        let content = self.display_cube().to_file_format();
        let array = js_sys::Uint8Array::from(content.as_bytes());
        let blob_parts = js_sys::Array::new();
        blob_parts.push(&array);

        let options = web_sys::BlobPropertyBag::new();
        options.set_type("text/plain");

        match Blob::new_with_u8_array_sequence_and_options(&blob_parts, &options) {
            Ok(blob) => match Url::create_object_url_with_blob(&blob) {
                Ok(url) => {
                    if let Some(window) = web_sys::window() {
                        if let Some(document) = window.document() {
                            if let Ok(element) = document.create_element("a") {
                                match element.dyn_into::<HtmlAnchorElement>() {
                                    Ok(anchor) => {
                                        anchor.set_href(&url);
                                        anchor.set_download("cube_state.txt");
                                        anchor.click();

                                        let _ = Url::revoke_object_url(&url);
                                        self.input_error_message =
                                            "ダウンロードを開始しました: cube_state.txt"
                                                .to_string();
                                    }
                                    Err(_) => {
                                        self.input_error_message =
                                            "保存エラー: a要素へのキャスト失敗".to_string();
                                    }
                                }
                            } else {
                                self.input_error_message =
                                    "保存エラー: a要素の作成に失敗".to_string();
                            }
                        } else {
                            self.input_error_message =
                                "保存エラー: documentオブジェクト取得失敗".to_string();
                        }
                    } else {
                        self.input_error_message =
                            "保存エラー: windowオブジェクト取得失敗".to_string();
                    }
                }
                Err(_) => {
                    self.input_error_message = "保存エラー: URLの生成に失敗".to_string();
                }
            },
            Err(_) => {
                self.input_error_message = "保存エラー: Blobの作成に失敗".to_string();
            }
        }
    }

    /// 読込ダイアログを表示して読み込み
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_with_dialog(&mut self) {
        let task = rfd::FileDialog::new()
            .set_directory(".")
            .add_filter("Text files", &["txt"])
            .pick_file();

        if let Some(path) = task {
            let path_str = path.to_string_lossy();
            match self.load_from_file(&path_str) {
                Ok(warning) => {
                    if warning.is_empty() {
                        self.input_error_message = format!(
                            "読み込みました: {}",
                            path.file_name().unwrap_or_default().to_string_lossy()
                        );
                    } else {
                        self.input_error_message = format!("読み込み完了: {}", warning);
                    }
                }
                Err(e) => {
                    self.input_error_message = format!("読み込みエラー: {}", e);
                }
            }
        }
    }

    /// Web版: ブラウザのファイル選択機能でファイルを読み込み
    #[cfg(target_arch = "wasm32")]
    pub fn load_with_dialog(&mut self) {
        let (tx, rx) = std::sync::mpsc::channel();
        self.file_receiver = Some(rx);

        // WASM環境では、AsyncFileDialogを使用して非同期にファイルを選択
        wasm_bindgen_futures::spawn_local(async move {
            let task = rfd::AsyncFileDialog::new()
                .add_filter("Text files", &["txt"])
                .pick_file();

            if let Some(file_handle) = task.await {
                let bytes = file_handle.read().await;

                if bytes.len() > MAX_FILE_SIZE {
                    let _ = tx.send(Err(format!(
                        "ファイルサイズが大きすぎます (最大: {} MB)",
                        MAX_FILE_SIZE / (1024 * 1024)
                    )));
                    return;
                }

                let content = String::from_utf8_lossy(&bytes).to_string();
                let _ = tx.send(Ok(content));
            }
        });
    }
}
