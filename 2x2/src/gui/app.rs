use crate::cube::{Color, Cube, Move};
use crate::gui::constants::*;
use crate::gui::renderer_3d::{draw_cube_3d, View3D};
use crate::history::History;
use crate::solver;
use crate::statistics::Statistics;
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::Instant;

/// スクランブルの最小手数
#[allow(dead_code)]
const MIN_SCRAMBLE_MOVES: usize = 5;

/// スクランブルの最大手数
const MAX_SCRAMBLE_MOVES: usize = 10;

/// デフォルトのアニメーション時間(秒)
const DEFAULT_ANIMATION_DURATION: f32 = 0.3;

/// アニメーション速度の最小値
#[allow(dead_code)]
const MIN_ANIMATION_SPEED: f32 = 0.1;

/// アニメーション速度の最大値
#[allow(dead_code)]
const MAX_ANIMATION_SPEED: f32 = 2.0;

/// ズーム倍率の最小値
const MIN_ZOOM_SCALE: f32 = 0.5;

/// ズーム倍率の最大値
const MAX_ZOOM_SCALE: f32 = 3.0;

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
    pub input_buffer: [Option<Color>; 24],
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
            input_buffer: [None; 24],
            selected_input_color: Color::White,
            input_error_message: String::new(),
            skip_parity_check: false,
            solver_task: SolverTask::Normal,
            statistics: Statistics::new(),
            history: History::new(),
        }
    }
}

impl CubeApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
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

        Self::default()
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

    /// ソルバーの探索を中止
    pub fn cancel_solve(&mut self) {
        self.solving = false;
        self.solution = None;
        self.solution_text.clear();
        self.solver_receiver = None;
        self.progress_receiver = None;
        self.move_queue.clear();
    }

    /// ソルバー実行（通常）
    pub fn solve(&mut self) {
        self.start_solver_internal(SolverTask::Normal, self.ignore_orientation);
    }

    /// 向きの自動復元を開始（即時）
    pub fn start_restore_orientation(&mut self) {
        if let Err(e) = self.cube.restore_orientation_instantly() {
            self.input_error_message = format!("向きの復元に失敗しました: {}", e);
        }
    }

    /// ソルバー実行の内部処理
    fn start_solver_internal(&mut self, task: SolverTask, ignore_orientation: bool) {
        if self.solving {
            return;
        }
        self.solving = true;
        self.solver_task = task;
        self.solver_progress = 0.0;

        match task {
            SolverTask::Normal => self.solution_text = "探索中...".to_string(),
        }

        self.solving_start_time = Some(Instant::now()); // 開始時刻を記録

        // 解法開始時の状態を保存
        self.solution_cube_state = Some(self.cube.clone());
        self.solution_step = 0;

        let cube_clone = self.cube.clone();
        let (tx, rx) = channel();
        let (progress_tx, progress_rx) = channel();
        self.solver_receiver = Some(rx);
        self.progress_receiver = Some(progress_rx);

        thread::spawn(move || {
            // HTM対応により、向きの有無に関わらず最大11手で必ず解ける
            let max_depth = solver::DEFAULT_MAX_DEPTH;
            println!(
                "ソルバー開始: 深度{}まで探索 (タスク: {:?})",
                max_depth, task
            );
            let solution = solver::solve_with_progress(
                &cube_clone,
                max_depth,
                ignore_orientation,
                Some(progress_tx),
            );
            println!(
                "ソルバー完了: 解が{}",
                if solution.found {
                    "見つかりました"
                } else {
                    "見つかりませんでした"
                }
            );
            if solution.found {
                println!("解の手数: {}", solution.moves.len());
            }
            if let Err(e) = tx.send(solution) {
                eprintln!("ソルバー結果の送信に失敗しました: {:?}", e);
            }
        });
    }

    /// アニメーション更新
    fn update_animation(&mut self) {
        if let Some(ref mut anim) = self.animation {
            if anim.update() {
                // アニメーション完了
                self.cube.apply_move(anim.current_move);
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
                self.animation = Some(AnimationState::with_easing(mv, duration, easing));
            } else if let Some(single_mv) = mv.split_to_single() {
                // 180度回転の場合、90度回転2回に分割する
                // F2をF1枚分の時間で終わらせるため、各アニメーションの長さは半分
                let half_duration = self.animation_speed * ANIMATION_SPLIT_DURATION_FACTOR;

                // 1回目の90度回転 (加速のみ)
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
                self.animation = Some(AnimationState::new(mv, self.animation_speed));
            }
        }
    }

    /// ソルバーの結果を確認
    fn check_solver_result(&mut self) {
        if let Some(rx) = &self.solver_receiver {
            if let Ok(solution) = rx.try_recv() {
                self.solving = false;
                self.solver_receiver = None;
                self.progress_receiver = None;

                // 所要時間を計算
                if let Some(start_time) = self.solving_start_time.take() {
                    let duration = start_time.elapsed().as_secs_f32();
                    self.last_solve_duration = Some(duration);
                }

                if solution.found {
                    match self.solver_task {
                        SolverTask::Normal => {
                            self.solution = Some(solution.moves.clone());
                            let duration_text = if let Some(d) = self.last_solve_duration {
                                format!(" ({:.2}秒)", d)
                            } else {
                                String::new()
                            };
                            self.solution_text =
                                format!("解法: {} 手{}", solution.moves.len(), duration_text);
                            self.solution_step = 0;
                            // 自動実行はしない（ステップ操作で手動実行）
                        }
                    }
                } else {
                    self.solution = None;
                    match self.solver_task {
                        SolverTask::Normal => {
                            self.solution_text = "解が見つかりませんでした".to_string()
                        }
                    }
                }
            }
        }
    }

    /// ソルバーの進捗を確認
    fn check_progress(&mut self) {
        if let Some(rx) = &self.progress_receiver {
            while let Ok(progress) = rx.try_recv() {
                self.solver_progress = progress;
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
                let mut colors = [Color::Gray; 24];

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
    pub fn solution_step_forward(&mut self) {
        if self.animation.is_some() || !self.move_queue.is_empty() {
            return;
        }
        if let Some(solution) = &self.solution {
            if self.solution_step < solution.len() {
                let mv = solution[self.solution_step];
                self.move_queue.push((mv, None, None));
                self.pending_solution_update = Some(1);
            }
        }
    }

    /// 解法の前のステップへ戻る
    pub fn solution_step_backward(&mut self) {
        if self.animation.is_some() || !self.move_queue.is_empty() {
            return;
        }
        if let Some(solution) = &self.solution {
            if self.solution_step > 0 {
                let mv = solution[self.solution_step - 1];
                let inverse_mv = mv.inverse();
                self.move_queue.push((inverse_mv, None, None));
                self.pending_solution_update = Some(-1);
            }
        }
    }

    /// 解法の最初へ戻る
    pub fn solution_step_reset(&mut self) {
        if let Some(cube_state) = &self.solution_cube_state {
            self.cube = cube_state.clone();
            self.solution_step = 0;
        }
    }

    /// 解法を最後まで実行
    pub fn solution_step_to_end(&mut self) {
        if let Some(solution) = &self.solution {
            // アニメーション中は実行しない
            if self.animation.is_some() {
                return;
            }

            // 残りの手を全て即座に適用
            while self.solution_step < solution.len() {
                let mv = solution[self.solution_step];
                self.cube.apply_move(mv);
                self.solution_step += 1;
            }
        }
    }

    fn setup_custom_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();

        fonts.font_data.insert(
            "NotoSansCJKjp".to_owned(),
            egui::FontData::from_static(include_bytes!(
                "../../assets/fonts/NotoSansCJKjp-Regular.otf"
            )),
        );

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

        ctx.set_fonts(fonts);
    }

    /// 3Dビューの描画処理
    fn show_3d_view(&mut self, ui: &mut egui::Ui) {
        let available = ui.available_size();
        let size = available.x.min(available.y);
        // 領域確保
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(available.x, size), // 横幅いっぱいに使う
            egui::Sense::drag(),
        );

        // 3Dビュー操作
        if response.dragged() {
            let delta = response.drag_delta();
            self.view_3d.yaw += delta.x * MOUSE_SENSITIVITY;
            self.view_3d.pitch += delta.y * MOUSE_SENSITIVITY;

            // Pitch制限
            self.view_3d.pitch = self.view_3d.pitch.clamp(
                -std::f32::consts::FRAC_PI_2 + VIEW3D_PITCH_LIMIT_MARGIN,
                std::f32::consts::FRAC_PI_2 - VIEW3D_PITCH_LIMIT_MARGIN,
            );
        }
        // ズーム操作
        if response.hovered() {
            let zoom_delta = ui.input(|i| i.raw_scroll_delta.y);
            if zoom_delta != 0.0 {
                self.view_3d.scale *= if zoom_delta > 0.0 {
                    ZOOM_FACTOR
                } else {
                    1.0 / ZOOM_FACTOR
                };
                self.view_3d.scale = self.view_3d.scale.clamp(MIN_ZOOM_SCALE, MAX_ZOOM_SCALE);
            }
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

        // ヘルプテキストを描画
        let help_text = "ドラッグで回転、ホイールでズーム";
        let help_pos = rect.min + egui::vec2(UI_SPACING_LARGE, UI_SPACING_LARGE);
        ui.painter().text(
            help_pos,
            egui::Align2::LEFT_TOP,
            help_text,
            egui::FontId::proportional(UI_HELP_TEXT_SIZE),
            egui::Color32::from_rgba_premultiplied(255, 255, 255, 200),
        );
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

    // ============ 6面スキャン入力モード用メソッド ============

    /// スキャンモードを開始
    pub fn start_scanning_mode(&mut self) {
        self.input_state = InputState::Scanning { face_index: 0 };
        self.input_buffer = [None; 24];
        self.selected_input_color = Color::White;
        self.input_error_message.clear();
    }

    /// スキャンモードをキャンセル
    pub fn cancel_scanning_mode(&mut self) {
        self.input_state = InputState::Normal;
        self.input_buffer = [None; 24];
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
            let global_index = face_index * 4 + position;
            if global_index < 24 {
                self.input_buffer[global_index] = Some(color);
            }
        }
    }

    /// 現在の面の指定位置のステッカー色を取得
    pub fn get_current_face_sticker(&self, position: usize) -> Option<Color> {
        if let InputState::Scanning { face_index } = self.input_state {
            let global_index = face_index * 4 + position;
            if global_index < 24 {
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
            let start = face_index * 4;
            let end = start + 4;
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
        let colors: [Color; 24] = self
            .input_buffer
            .iter()
            .map(|c| c.expect("全ての色が入力されています"))
            .collect::<Vec<_>>()
            .try_into()
            .expect("配列は24要素です");

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
        self.input_buffer = [None; 24];
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

    /// キューブの状態をファイルに保存
    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        // 現在表示されているキューブ（スキャン中なら入力バッファベース）を保存
        let content = self.display_cube().to_file_format();
        std::fs::write(path, content).map_err(|e| format!("ファイルの保存に失敗しました: {}", e))
    }

    /// ファイルからキューブの状態を読み込み
    pub fn load_from_file(&mut self, path: &str) -> Result<String, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("ファイルの読み込みに失敗しました: {}", e))?;

        let loaded_cube = Cube::from_file_format(&content).map_err(|e| e.to_string())?;

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
            self.input_buffer = [None; 24];
        }

        self.solution = None;
        self.solution_text.clear();
        self.animation = None;
        self.move_queue.clear();
        self.input_error_message.clear();

        Ok(warning)
    }

    /// 保存ダイアログを表示して保存
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

    /// 読込ダイアログを表示して読み込み
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
}

impl eframe::App for CubeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.check_solver_result();
        self.check_progress();
        self.update_animation();
        self.handle_input(ctx);

        // 継続的な再描画をリクエスト
        ctx.request_repaint();

        // 右側のサイドパネル (コントロールパネル)
        egui::SidePanel::right("control_panel")
            .min_width(UI_SIDE_PANEL_WIDTH)
            .default_width(UI_SIDE_PANEL_WIDTH)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(UI_SPACING_LARGE);
                    crate::gui::controls::draw_controls(self, ui);
                });
            });

        // 中央パネル (メインコンテンツ)
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("2x2 ルービックキューブ");
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
                        self.show_3d_view(ui);
                    }
                    ViewMode::Both => {
                        ui.columns(2, |columns| {
                            columns[0].vertical(|ui| {
                                ui.heading("3Dビュー");
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
