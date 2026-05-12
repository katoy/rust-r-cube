use crate::cube::{Color, Cube, Move};
use crate::gui::constants::*;
use crate::gui::renderer_3d::{draw_cube_3d, View3D};
use crate::history::History;
use crate::solver;
use crate::statistics::Statistics;
use std::sync::mpsc::Receiver;

use web_time::Instant;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// WASM環境で確認ダイアログとUI更新後のコールバック実行
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(inline_js = "
    export function confirm_solver_start() {
        console.log('🔍 confirm_solver_start called at', Date.now());
        const result = confirm('Kociemba アルゴリズムによる解法を探索します。\\nよろしいですか？');
        console.log('✅ confirm result:', result, 'at', Date.now());
        return result;
    }

    // UI更新を待ってからコールバックを実行
    // requestAnimationFrameを2回使うことで、確実に描画後に実行
    export function schedule_after_render(callback) {
        console.log('📅 Scheduling callback after render');
        requestAnimationFrame(() => {
            requestAnimationFrame(() => {
                console.log('🎬 Executing callback after render');
                callback();
            });
        });
    }
")]
extern "C" {
    fn confirm_solver_start() -> bool;

    #[wasm_bindgen(js_name = schedule_after_render)]
    fn schedule_after_render(callback: &Closure<dyn FnMut()>);
}

/// スクランブルの最小手数
#[allow(dead_code)]
const MIN_SCRAMBLE_MOVES: usize = 5;

/// ファイル読み込みの最大サイズ (1 MB)
const MAX_FILE_SIZE: usize = 1024 * 1024;

// ロジック分割モジュール
mod solution;
mod scanner;
mod file_io;
mod solver_control;

/// スクランブルの最大手数
const MAX_SCRAMBLE_MOVES: usize = 20;

/// デフォルトのアニメーション時間(秒)
const DEFAULT_ANIMATION_DURATION: f32 = 0.3;

/// アニメーション速度の最小値
#[allow(dead_code)]
const MIN_ANIMATION_SPEED: f32 = 0.1;

/// アニメーション速度の最大値
#[allow(dead_code)]
const MAX_ANIMATION_SPEED: f32 = 2.0;

/// ズーム倍率の制限
const ZOOM_MIN: f32 = 0.1;
const ZOOM_MAX: f32 = 5.0;

/// キューブの表示モードを定義します。
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ViewMode {
    /// 2D 展開図のみ表示
    TwoD,
    /// 3D ビューのみ表示
    ThreeD,
    /// 2D と 3D を両方表示
    Both,
}

/// ユーザーの入力状態を定義します。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputState {
    /// 通常の状態
    Normal,
    /// 6面スキャン入力モード
    Scanning {
        /// 現在入力中の面のインデックス (0-5: U, D, L, R, F, B)
        face_index: usize,
    },
}

/// アニメーションのイージング（加減速）モードを定義します。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EasingMode {
    /// 通常 (加速してから減速)
    EaseInOut,
    /// 前半部分 (加速のみ、180度回転の分割前半などで使用)
    EaseIn,
    /// 後半部分 (減速のみ、180度回転の分割後半などで使用)
    EaseOut,
}

/// 現在実行中のアニメーションの状態を管理する構造体。
#[derive(Debug, Clone)]
pub struct AnimationState {
    /// 実行中の操作
    pub current_move: Move,
    /// 進捗率 (0.0 から 1.0)
    pub progress: f32,
    /// アニメーション開始時刻
    pub start_time: Instant,
    /// アニメーションの総時間（秒）
    pub duration: f32,
    /// 使用するイージングモード
    pub easing: EasingMode,
}

impl AnimationState {
    /// 新しいアニメーション状態を作成します（標準の EaseInOut）。
    pub fn new(mv: Move, duration: f32) -> Self {
        Self {
            current_move: mv,
            progress: 0.0,
            start_time: Instant::now(),
            duration,
            easing: EasingMode::EaseInOut,
        }
    }

    /// イージングモードを指定して、新しいアニメーション状態を作成します。
    pub fn with_easing(mv: Move, duration: f32, easing: EasingMode) -> Self {
        Self {
            current_move: mv,
            progress: 0.0,
            start_time: Instant::now(),
            duration,
            easing,
        }
    }

    /// 経過時間に基づいて進捗率を更新します。
    ///
    /// # 戻り値
    ///
    /// アニメーションが完了した（1.0 に達した）場合は `true` を返します。
    pub fn update(&mut self) -> bool {
        if self.duration <= 0.001 {
            self.progress = 1.0;
            return true;
        }
        let elapsed = self.start_time.elapsed().as_secs_f32();
        self.progress = (elapsed / self.duration).min(1.0);
        self.progress >= 1.0
    }

    /// イージングモードに基づいた現在の進捗率（計算値）を取得します。
    pub fn eased_progress(&self) -> f32 {
        let t = self.progress;
        match self.easing {
            EasingMode::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
            EasingMode::EaseIn => t * t,
            EasingMode::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
        }
    }
}

/// アプリケーションのメイン状態を保持する構造体。
pub struct CubeApp {
    /// 現在のキューブの状態
    cube: Cube,
    /// 現在実行中のアニメーション（実行されていない場合は `None`）
    animation: Option<AnimationState>,
    /// 未実行の操作キュー
    move_queue: Vec<(Move, Option<EasingMode>, Option<f32>)>,
    /// アニメーションの速度（1手あたりの秒数）
    pub animation_speed: f32,
    /// ソルバーが見つけた解決手順（見つかっていない場合は `None`）
    pub solution: Option<Vec<Move>>,
    /// 現在ソルバーが探索中かどうか
    pub solving: bool,
    /// ソルバーの現在の探索進捗率 (0.0 - 1.0)
    pub solver_progress: f32,
    /// ソルバーの状態テキスト（「探索中...」など）
    pub solution_text: String,

    /// 現在の表示モード（2D, 3D, Both）
    pub view_mode: ViewMode,
    /// 3Dビューのカメラ・倍率設定
    pub view_3d: View3D,

    /// ソルバーからの結果受信用レシーバ
    solver_receiver: Option<Receiver<solver::Solution>>,
    /// ソルバーからの進捗受信用レシーバ
    progress_receiver: Option<Receiver<f32>>,

    /// 解法手順をたどる際の現在のステップ番号
    pub solution_step: usize,
    /// 解法開始前のキューブの状態（リセット用）
    pub solution_cube_state: Option<Cube>,
    /// アニメーション完了後にステップ番号を更新するための保留値
    pending_solution_update: Option<isize>,

    /// ソルバーがステッカーの向きを無視するかどうか
    pub ignore_orientation: bool,

    /// 探索開始時刻
    pub solving_start_time: Option<Instant>,
    /// 前回の探索にかかった時間（秒）
    pub last_solve_duration: Option<f32>,

    /// モードに応じた入力状態（通常、スキャンモード）
    pub input_state: InputState,
    /// 6面スキャン入力中の色データ一時保持用
    pub input_buffer: [Option<Color>; 54],
    /// 入力パネルで選択されている色
    pub selected_input_color: Color,
    /// 入力中のエラーメッセージ（不正なパリティなど）
    pub input_error_message: String,

    /// 開発者用オプション：パリティチェック（物理的妥当性）をスキップするか
    pub skip_parity_check: bool,

    /// 現在実行中のソルバータスクの種類
    pub solver_task: SolverTask,

    /// 解法時間などの統計情報
    pub statistics: Statistics,

    /// 手動操作の履歴 (Undo/Redo用)
    pub history: History,

    /// WASM環境でのソルバー起動ペンディング状態
    #[cfg(target_arch = "wasm32")]
    pending_solver_start: Option<(SolverTask, bool, u8)>, // (task, ignore_orientation, frames_to_wait)

    /// WASM環境用: インクリメンタルソルバーの状態
    #[cfg(target_arch = "wasm32")]
    solver_state: Option<solver::SolverState>,

    /// ファイル読み込み用のレシーバー
    file_receiver: Option<Receiver<Result<String, String>>>,
}

/// ソルバーのタスク種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolverTask {
    Normal, // 通常の解法探索
}

impl Default for CubeApp {
    fn default() -> Self {
        Self {
            cube: Cube::new(),
            animation: None,
            move_queue: Vec::new(),
            animation_speed: DEFAULT_ANIMATION_DURATION,
            solution: None,
            solving: false,
            solver_progress: 0.0,
            solution_text: String::new(),
            view_mode: ViewMode::Both,
            view_3d: View3D::default(),
            solver_receiver: None,
            progress_receiver: None,
            solution_step: 0,
            solution_cube_state: None,
            pending_solution_update: None,
            ignore_orientation: false,
            solving_start_time: None,
            last_solve_duration: None,
            input_state: InputState::Normal,
            input_buffer: [None; 54],
            selected_input_color: Color::White,
            input_error_message: String::new(),
            skip_parity_check: false,
            solver_task: SolverTask::Normal,
            statistics: Statistics::new(),
            history: History::new(),
            file_receiver: None,
            #[cfg(target_arch = "wasm32")]
            pending_solver_start: None,
            #[cfg(target_arch = "wasm32")]
            solver_state: None,
        }
    }
}

impl CubeApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        tracing::info!("CubeApp::new starting");
        // 日本語フォントを設定
        Self::setup_custom_fonts(&cc.egui_ctx);

        // 必要に応じてフォントサイズを調整
        let mut style = (*cc.egui_ctx.style()).clone();
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(UI_BODY_FONT_SIZE, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(UI_BODY_FONT_SIZE, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::new(UI_HEADING_FONT_SIZE, egui::FontFamily::Proportional),
        );
        cc.egui_ctx.set_style(style);

        tracing::info!("CubeApp::new calling Self::default()");
        let app = Self::default();
        tracing::info!("CubeApp::new finishing");
        app
    }

    /// 回転操作をキューに追加
    pub fn queue_move(&mut self, mv: Move) {
        self.move_queue.push((mv, None, None));
        self.statistics.record_manual_move();
        self.history.push(mv);
    }

    /// 複数の回転操作をキューに追加
    pub fn queue_moves(&mut self, moves: Vec<Move>) {
        for mv in moves {
            self.move_queue.push((mv, None, None));
        }
    }

    /// スクランブル
    pub fn scramble(&mut self) {
        self.cube = Cube::new();
        self.cube.scramble(MAX_SCRAMBLE_MOVES);
        self.solution = None;
        self.solution_text.clear();
        self.move_queue.clear();
        self.history.clear();
        self.animation = None;
        self.pending_solution_update = None;
    }

    /// Undo: 最後の手動操作を取り消す
    pub fn undo(&mut self) {
        if let Some(inverse_move) = self.history.undo() {
            self.move_queue.push((inverse_move, None, None));
        }
    }

    /// Redo: 取り消した操作をやり直す
    pub fn redo(&mut self) {
        if let Some(mv) = self.history.redo() {
            self.move_queue.push((mv, None, None));
        }
    }

    /// リセット
    pub fn reset(&mut self) {
        self.cube = Cube::new();
        self.cancel_solve();
        self.animation = None;
        self.pending_solution_update = None;
    }


    /// アニメーション更新
    fn update_animation(&mut self) {
        if let Some(ref mut anim) = self.animation {
            if anim.update() {
                // アニメーション完了
                self.animation = None;

                // ソルーション再生中の場合、ステップ数を更新
                if let Some(delta) = self.pending_solution_update {
                    if delta > 0 {
                        self.solution_step += delta as usize;
                    } else if delta < 0 {
                        self.solution_step = self.solution_step.saturating_sub((-delta) as usize);
                    }
                    self.pending_solution_update = None;
                }
            }
        } else if let Some((mv, easing_override, duration_override)) =
            self.move_queue.first().copied()
        {
            // 次の操作を開始
            self.move_queue.remove(0);

            if let Some(easing) = easing_override {
                // 指定されたイージングで実行 (分割後半など)
                let duration = duration_override.unwrap_or(self.animation_speed);
                self.cube.apply_move(mv);
                self.animation = Some(AnimationState::with_easing(mv, duration, easing));
            } else if let Some(single_mv) = mv.split_to_single() {
                // 180度回転の場合、90度回転2回に分割する
                // F2をF1枚分の時間で終わらせるため、各アニメーションの長さは半分
                let half_duration = self.animation_speed * ANIMATION_SPLIT_DURATION_FACTOR;

                // 1回目の90度回転 (加速のみ)
                self.cube.apply_move(single_mv);
                self.animation = Some(AnimationState::with_easing(
                    single_mv,
                    half_duration,
                    EasingMode::EaseIn,
                ));
                // 2回目の90度回転 (減速のみ) をキューの先頭に挿入
                self.move_queue.insert(
                    0,
                    (single_mv, Some(EasingMode::EaseOut), Some(half_duration)),
                );
            } else {
                // 通常の90度回転
                self.cube.apply_move(mv);
                self.animation = Some(AnimationState::new(mv, self.animation_speed));
            }
        }
    }

/// キューブの状態を取得
    pub fn cube(&self) -> &Cube {
        &self.cube
    }

    /// 描画に使用するキューブを取得
    ///
    /// スキャンモード中は入力バッファから一時キューブを生成して返し、
    /// 通常モードは実際のキューブを返します。
    pub fn display_cube(&self) -> Cube {
        match &self.input_state {
            InputState::Scanning { .. } => {
                // スキャンモード中: 入力バッファから一時キューブを生成
                // 未入力のステッカーはデフォルトの色（グレー風）にする
                let mut colors = [Color::Gray; 54];

                for (i, maybe_color) in self.input_buffer.iter().enumerate() {
                    if let Some(color) = maybe_color {
                        colors[i] = *color;
                    }
                }

                Cube::from_colors(&colors).unwrap_or_else(|_| Cube::new())
            }
            InputState::Normal => {
                // 通常モード: 実際のキューブを返す
                self.cube.clone()
            }
        }
    }

    /// 編集中の面のインデックスを取得（ハイライト表示用）
    ///
    /// スキャンモード中は現在編集中の面のインデックス（0-5）を返し、
    /// 通常モードはNoneを返します。
    pub fn editing_face_index(&self) -> Option<usize> {
        match &self.input_state {
            InputState::Scanning { face_index } => Some(*face_index),
            InputState::Normal => None,
        }
    }

    /// アニメーション状態を取得
    pub fn animation(&self) -> Option<&AnimationState> {
        self.animation.as_ref()
    }

    /// 解法の次のステップへ進む

    fn setup_custom_fonts(ctx: &egui::Context) {
        tracing::info!("setup_custom_fonts starting");
        let mut fonts = egui::FontDefinitions::default();

        tracing::info!("Loading font data");
        fonts.font_data.insert(
            "NotoSansCJKjp".to_owned(),
            egui::FontData::from_static(include_bytes!(
                "../../../assets/fonts/NotoSansCJKjp-Regular.otf"
            )),
        );

        tracing::info!("Setting font families");
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "NotoSansCJKjp".to_owned());

        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "NotoSansCJKjp".to_owned());

        tracing::info!("Applying fonts to context");
        ctx.set_fonts(fonts);
        tracing::info!("setup_custom_fonts finishing");
    }

    /// 3Dビューの描画処理
    fn show_3d_view(&mut self, ui: &mut egui::Ui) {
        let available = ui.available_size();
        let size = available.x.min(available.y);
        let y_offset = (available.y - size) / 2.0;

        if y_offset > 0.0 {
            ui.add_space(y_offset);
        }

        ui.horizontal(|ui| {
            let x_offset = (ui.available_width() - size) / 2.0;
            if x_offset > 0.0 {
                ui.add_space(x_offset);
            }

            // 描画領域 (正方形)
            let rect_size = egui::vec2(size, size);
            let (rect, response) = ui.allocate_at_least(rect_size, egui::Sense::drag());

            // 3Dビュー操作
            if response.dragged() {
                self.view_3d.yaw += response.drag_delta().x * MOUSE_SENSITIVITY;
                self.view_3d.pitch += response.drag_delta().y * MOUSE_SENSITIVITY;

                // Pitch制限
                self.view_3d.pitch = self.view_3d.pitch.clamp(
                    -std::f32::consts::FRAC_PI_2 + VIEW3D_PITCH_LIMIT_MARGIN,
                    std::f32::consts::FRAC_PI_2 - VIEW3D_PITCH_LIMIT_MARGIN,
                );
            }
            // ズーム操作
            let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
            if scroll_delta != 0.0 {
                self.view_3d.scale *= if scroll_delta > 0.0 {
                    ZOOM_FACTOR
                } else {
                    1.0 / ZOOM_FACTOR
                };
                self.view_3d.scale = self.view_3d.scale.clamp(ZOOM_MIN, ZOOM_MAX);
            }

            let display_cube = self.display_cube();
            let highlight_face = self.editing_face_index();
            draw_cube_3d(
                ui,
                rect,
                &display_cube,
                self.animation.as_ref(),
                &self.view_3d,
                highlight_face,
            );
        });
    }

    /// 2Dビューの描画処理
    fn show_2d_view(&mut self, ui: &mut egui::Ui) {
        let available = ui.available_size();
        let size = available.x.min(available.y);

        let (rect, _response) =
            ui.allocate_exact_size(egui::vec2(available.x, size), egui::Sense::hover());

        let display_cube = self.display_cube();
        let highlight_face = self.editing_face_index();
        crate::gui::renderer::draw_cube(
            ui,
            rect,
            &display_cube,
            self.animation.as_ref(),
            highlight_face,
        );
    }

    /// キーボード入力を処理
    fn handle_input(&mut self, ctx: &egui::Context) {
        // アニメーション中やソルブ中は入力を受け付けない（オプション）
        // ここでは連打できるように許可するが、キューに追加される

        // Shiftキーが押されているか確認
        let shift = ctx.input(|i| i.modifiers.shift);

        // 各キーの処理
        let input = ctx.input(|i| {
            let mut moves = Vec::new();

            if i.key_pressed(egui::Key::R) {
                moves.push(if shift { Move::Rp } else { Move::R });
            }
            if i.key_pressed(egui::Key::L) {
                moves.push(if shift { Move::Lp } else { Move::L });
            }
            if i.key_pressed(egui::Key::U) {
                moves.push(if shift { Move::Up } else { Move::U });
            }
            if i.key_pressed(egui::Key::D) {
                moves.push(if shift { Move::Dp } else { Move::D });
            }
            if i.key_pressed(egui::Key::F) {
                moves.push(if shift { Move::Fp } else { Move::F });
            }
            if i.key_pressed(egui::Key::B) {
                moves.push(if shift { Move::Bp } else { Move::B });
            }
            moves
        });

        for mv in input {
            self.queue_move(mv);
        }

        // 機能キー
        if ctx.input(|i| i.key_pressed(egui::Key::S)) {
            self.scramble();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Space)) && !self.solving {
            self.solve();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.reset();
        }
    }

    /// アプリケーションのコアロジックを更新します（UIレンダリング以外）
    pub fn update_logic(&mut self, ctx: &egui::Context) {
        // WASM環境: ペンディング状態のソルバーを起動
        #[cfg(target_arch = "wasm32")]
        if let Some((_task, ignore_orientation, mut frames_to_wait)) =
            self.pending_solver_start.take()
        {
            if frames_to_wait > 0 {
                // まだ待機中：カウンターをデクリメントして再設定
                frames_to_wait -= 1;
                self.pending_solver_start = Some((_task, ignore_orientation, frames_to_wait));
            } else {
                // 待機完了：インクリメンタルソルバーを初期化
                let max_depth = solver::DEFAULT_MAX_DEPTH;
                self.solver_state = Some(solver::SolverState::new(
                    &self.cube,
                    max_depth,
                    ignore_orientation,
                ));
            }
        }

        // WASM環境: インクリメンタルソルバーのチャンク処理
        #[cfg(target_arch = "wasm32")]
        if let Some(state) = &mut self.solver_state {
            // 100ノード処理（約10-20ms）
            let (_processed, is_complete) = state.process_chunk(100);

            if is_complete {
                // 完了: 結果を取得
                if let Some(solution) = state.get_solution() {
                    // duration がまだ計算されていない場合は計算する
                    if self.last_solve_duration.is_none() {
                        if let Some(start_time) = self.solving_start_time {
                            let duration = start_time.elapsed().as_secs_f32();
                            self.last_solve_duration = Some(duration);
                        }
                    }
                    self.apply_solver_result(&solution);
                }

                self.solving = false;
                self.solver_state = None;
            } else {
                // 進捗更新
                self.solver_progress = state.estimate_progress();
            }
        }

        self.check_solver_result();
        self.check_progress();
        self.update_animation();
        self.handle_input(ctx);

        // ファイル読み込みの結果を確認
        if let Some(rx) = &self.file_receiver {
            if let Ok(result) = rx.try_recv() {
                self.file_receiver = None;
                match result {
                    Ok(content) => match self.load_from_content(&content) {
                        Ok(warning) => {
                            if warning.is_empty() {
                                self.input_error_message = "ファイルを読み込みました".to_string();
                            } else {
                                self.input_error_message = format!("読み込み完了: {}", warning);
                            }
                        }
                        Err(e) => {
                            self.input_error_message = format!("読み込みエラー: {}", e);
                        }
                    },
                    Err(e) => {
                        self.input_error_message = format!("ファイル選択エラー: {}", e);
                    }
                }
            }
        }

        // 継続的な再描画をリクエスト
        ctx.request_repaint();
    }
}

impl eframe::App for CubeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_logic(ctx);

        // 右側のサイドパネル (コントロールパネル)
        egui::SidePanel::right("control_panel")
            .min_width(UI_SIDE_PANEL_WIDTH)
            .default_width(UI_SIDE_PANEL_WIDTH)
            .resizable(false)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("side_panel_scroll")
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.add_space(UI_SPACING_LARGE);
                            crate::gui::controls::draw_controls(self, ui);
                        });
                    });
            });

        // 中央パネル (メインコンテンツ)
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("3x3 ルービックキューブ");
                ui.add_space(20.0);
                ui.hyperlink_to("⬅ 2x2版はこちら", "../");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.selectable_value(&mut self.view_mode, ViewMode::Both, "2D & 3D");
                    ui.selectable_value(&mut self.view_mode, ViewMode::ThreeD, "3Dのみ");
                    ui.selectable_value(&mut self.view_mode, ViewMode::TwoD, "2Dのみ");
                });
            });
            ui.add_space(UI_SPACING_LARGE);

            // キューブ表示領域
            ui.group(|ui| {
                // 利用可能なサイズを計算
                ui.set_min_width(300.0);
                ui.set_min_height(300.0);

                match self.view_mode {
                    ViewMode::TwoD => {
                        self.show_2d_view(ui);
                    }
                    ViewMode::ThreeD => {
                        ui.vertical(|ui| {
                            ui.heading("3Dビュー");
                            ui.label(
                                egui::RichText::new("ドラッグで回転、ホイールでズーム")
                                    .size(UI_HELP_TEXT_SIZE),
                            );
                            self.show_3d_view(ui);
                        });
                    }
                    ViewMode::Both => {
                        ui.columns(2, |columns| {
                            columns[0].vertical(|ui| {
                                ui.heading("3Dビュー");
                                ui.label(
                                    egui::RichText::new("ドラッグで回転、ホイールでズーム")
                                        .size(UI_HELP_TEXT_SIZE),
                                );
                                self.show_3d_view(ui);
                            });
                            columns[1].vertical(|ui| {
                                ui.heading("展開図");
                                self.show_2d_view(ui);
                            });
                        });
                    }
                }
            });
        });
    }
}
