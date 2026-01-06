use crate::cube::{Cube, Move};
use crate::gui::renderer_3d::{draw_cube_3d, View3D};
use crate::solver;
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

/// ズーム変化率
const ZOOM_FACTOR: f32 = 1.1;

/// 表示モード
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ViewMode {
    TwoD,
    ThreeD,
    Both,
}

/// アニメーション状態
#[derive(Debug, Clone)]
pub struct AnimationState {
    pub current_move: Move,
    pub progress: f32, // 0.0 to 1.0
    pub start_time: Instant,
    pub duration: f32, // seconds
}

impl AnimationState {
    pub fn new(mv: Move, duration: f32) -> Self {
        Self {
            current_move: mv,
            progress: 0.0,
            start_time: Instant::now(),
            duration,
        }
    }

    pub fn update(&mut self) -> bool {
        if self.duration <= 0.001 {
            self.progress = 1.0;
            return true;
        }
        let elapsed = self.start_time.elapsed().as_secs_f32();
        self.progress = (elapsed / self.duration).min(1.0);
        self.progress >= 1.0
    }

    /// イージング関数 (ease-in-out)
    pub fn eased_progress(&self) -> f32 {
        let t = self.progress;
        if t < 0.5 {
            2.0 * t * t
        } else {
            -1.0 + (4.0 - 2.0 * t) * t
        }
    }
}

/// メインアプリケーション
pub struct CubeApp {
    cube: Cube,
    animation: Option<AnimationState>,
    move_queue: Vec<Move>,
    pub animation_speed: f32, // seconds per move
    pub is_paused: bool,
    pub solution: Option<Vec<Move>>,
    solving: bool,
    pub solution_text: String,

    // 表示設定
    pub view_mode: ViewMode,
    pub view_3d: View3D,

    // ソルバー通信用
    solver_receiver: Option<Receiver<solver::Solution>>,

    // 解法ステップ管理
    pub solution_step: usize,
    pub solution_cube_state: Option<Cube>,
    // アニメーション完了後にsolution_stepを更新するための保留値 (+1 or -1)
    pending_solution_update: Option<isize>,

    // 解決設定
    pub ignore_orientation: bool,
}

impl Default for CubeApp {
    fn default() -> Self {
        Self {
            cube: Cube::new(),
            animation: None,
            move_queue: Vec::new(),
            animation_speed: DEFAULT_ANIMATION_DURATION,
            is_paused: false,
            solution: None,
            solving: false,
            solution_text: String::new(),
            view_mode: ViewMode::Both,
            view_3d: View3D::default(),
            solver_receiver: None,
            solution_step: 0,
            solution_cube_state: None,
            pending_solution_update: None,
            ignore_orientation: true,
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
            egui::FontId::new(14.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(14.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::new(18.0, egui::FontFamily::Proportional),
        );
        cc.egui_ctx.set_style(style);

        Self::default()
    }

    /// 回転操作をキューに追加
    pub fn queue_move(&mut self, mv: Move) {
        self.move_queue.push(mv);
    }

    /// 複数の回転操作をキューに追加
    pub fn queue_moves(&mut self, moves: Vec<Move>) {
        self.move_queue.extend(moves);
    }

    /// スクランブル
    pub fn scramble(&mut self) {
        self.cube = Cube::new();
        self.cube.scramble(MAX_SCRAMBLE_MOVES);
        self.solution = None;
        self.solution_text.clear();
        self.move_queue.clear();
        self.animation = None;
        self.pending_solution_update = None;
    }

    /// リセット
    pub fn reset(&mut self) {
        self.cube = Cube::new();
        self.solution = None;
        self.solution_text.clear();
        self.move_queue.clear();
        self.animation = None;
        self.pending_solution_update = None;
        // ソルバー実行中ならキャンセルできないが、結果を無視するようにフラグをクリアする
        self.solving = false;
        self.solver_receiver = None;
    }

    /// ソルバー実行
    pub fn solve(&mut self) {
        if self.solving {
            return;
        }
        self.solving = true;
        self.solution_text = "探索中...".to_string();

        // 解法開始時の状態を保存
        self.solution_cube_state = Some(self.cube.clone());
        self.solution_step = 0;

        let cube_clone = self.cube.clone();
        let (tx, rx) = channel();
        self.solver_receiver = Some(rx);
        let ignore_orientation = self.ignore_orientation;

        thread::spawn(move || {
            println!("ソルバー開始: 深度11まで探索");
            let max_depth = 11; // 2x2のGod's Numberは11（向き無視）だが、向きアリだと増える可能性がある。今回は11を据え置き。
            let solution = solver::solve(&cube_clone, max_depth, ignore_orientation);
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
        if self.is_paused {
            return;
        }

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
        } else if let Some(mv) = self.move_queue.first().copied() {
            // 次の操作を開始
            self.move_queue.remove(0);
            self.animation = Some(AnimationState::new(mv, self.animation_speed));
        }
    }

    /// ソルバーの結果を確認
    fn check_solver_result(&mut self) {
        if let Some(rx) = &self.solver_receiver {
            if let Ok(solution) = rx.try_recv() {
                self.solving = false;
                self.solver_receiver = None;

                if solution.found {
                    self.solution = Some(solution.moves.clone());
                    self.solution_text = format!("解法: {} 手", solution.moves.len());
                    self.solution_step = 0;
                    // 自動実行はしない（ステップ操作で手動実行）
                } else {
                    self.solution_text = "解が見つかりませんでした".to_string();
                }
            }
        }
    }

    /// キューブの状態を取得
    pub fn cube(&self) -> &Cube {
        &self.cube
    }

    /// アニメーション状態を取得
    pub fn animation(&self) -> Option<&AnimationState> {
        self.animation.as_ref()
    }

    /// 解法の次のステップへ進む
    pub fn solution_step_forward(&mut self) {
        if self.animation.is_some() {
            return;
        }
        if let Some(solution) = &self.solution {
            if self.solution_step < solution.len() {
                let mv = solution[self.solution_step];
                self.animation = Some(AnimationState::new(mv, self.animation_speed));
                self.pending_solution_update = Some(1);
            }
        }
    }

    /// 解法の前のステップへ戻る
    pub fn solution_step_backward(&mut self) {
        if self.animation.is_some() {
            return;
        }
        if let Some(solution) = &self.solution {
            if self.solution_step > 0 {
                let mv = solution[self.solution_step - 1];
                let inverse_mv = mv.inverse();
                self.animation = Some(AnimationState::new(inverse_mv, self.animation_speed));
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
        let len = self.solution.as_ref().map(|s| s.len()).unwrap_or(0);
        while self.solution_step < len {
            self.solution_step_forward();
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
            self.view_3d.yaw += delta.x * 0.01;
            self.view_3d.pitch += delta.y * 0.01;

            // Pitch制限
            self.view_3d.pitch = self.view_3d.pitch.clamp(
                -std::f32::consts::FRAC_PI_2 + 0.1,
                std::f32::consts::FRAC_PI_2 - 0.1,
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

        draw_cube_3d(ui, rect, &self.cube, self.animation.as_ref(), &self.view_3d);

        // ヘルプテキストを描画
        let help_text = "ドラッグで回転、ホイールでズーム";
        let help_pos = rect.min + egui::vec2(10.0, 10.0);
        ui.painter().text(
            help_pos,
            egui::Align2::LEFT_TOP,
            help_text,
            egui::FontId::proportional(12.0),
            egui::Color32::from_rgba_premultiplied(255, 255, 255, 200),
        );
    }

    /// 2Dビューの描画処理
    fn show_2d_view(&mut self, ui: &mut egui::Ui) {
        let available = ui.available_size();
        let size = available.x.min(available.y);

        let (rect, _response) =
            ui.allocate_exact_size(egui::vec2(available.x, size), egui::Sense::hover());

        crate::gui::renderer::draw_cube(ui, rect, &self.cube, self.animation.as_ref());
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
        // ソルバーの結果を確認
        self.check_solver_result();

        // キーボード入力処理
        self.handle_input(ctx);

        // アニメーション更新
        self.update_animation();

        // 継続的な再描画をリクエスト
        ctx.request_repaint();

        // 右側のサイドパネル (コントロールパネル)
        egui::SidePanel::right("control_panel")
            .min_width(250.0)
            .default_width(250.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(10.0);
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
            ui.add_space(10.0);

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
