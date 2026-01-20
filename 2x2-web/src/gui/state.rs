use crate::cube::{Color, Cube, Move};
use crate::gui::app::{AnimationState, EasingMode, SolverTask};
use crate::gui::renderer_3d::View3D;
use crate::history::History;
use crate::solver;
use crate::statistics::Statistics;
use std::sync::mpsc::Receiver;

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

#[cfg(target_arch = "wasm32")]
use instant::Instant;

/// キューブのコア状態（論理的な状態とビジネスロジック）
#[derive(Debug, Clone)]
pub struct CoreState {
    /// 現在のキューブの状態
    pub cube: Cube,
    /// 手動操作の履歴 (Undo/Redo用)
    pub history: History,
    /// 解法時間などの統計情報
    pub statistics: Statistics,
    /// ソルバーが見つけた解決手順（見つかっていない場合は `None`）
    pub solution: Option<Vec<Move>>,
    /// 解法開始前のキューブの状態（リセット用）
    pub solution_cube_state: Option<Cube>,
    /// 解法手順をたどる際の現在のステップ番号
    pub solution_step: usize,
}

impl Default for CoreState {
    fn default() -> Self {
        Self {
            cube: Cube::new(),
            history: History::new(),
            statistics: Statistics::new(),
            solution: None,
            solution_cube_state: None,
            solution_step: 0,
        }
    }
}

impl CoreState {
    /// 新しいコア状態を作成
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 回転操作を適用
    pub fn apply_move(&mut self, mv: Move) {
        self.cube.apply_move(mv);
        self.history.push(mv);
        self.statistics.record_manual_move();
    }

    /// スクランブル
    pub fn scramble(&mut self, moves: usize) {
        self.cube.scramble(moves);
    }

    /// リセット
    pub fn reset(&mut self) {
        self.cube = Cube::new();
        self.history.clear();
        self.solution = None;
        self.solution_cube_state = None;
        self.solution_step = 0;
    }

    /// Undo
    pub fn undo(&mut self) -> Option<Move> {
        if let Some(inverse_mv) = self.history.undo() {
            self.cube.apply_move(inverse_mv);
            Some(inverse_mv)
        } else {
            None
        }
    }

    /// Redo
    pub fn redo(&mut self) -> Option<Move> {
        if let Some(mv) = self.history.redo() {
            self.cube.apply_move(mv);
            Some(mv)
        } else {
            None
        }
    }
}

/// ソルバー実行状態
pub struct SolverStateManager {
    /// 現在ソルバーが探索中かどうか
    pub solving: bool,
    /// 現在実行中のソルバータスクの種類
    pub solver_task: SolverTask,
    /// ソルバーの現在の探索進捗率 (0.0 - 1.0)
    pub solver_progress: f32,
    /// ソルバーの状態テキスト（「探索中...」など）
    pub solution_text: String,
    /// 探索開始時刻
    pub solving_start_time: Option<Instant>,
    /// 前回の探索にかかった時間（秒）
    pub last_solve_duration: Option<f32>,
    /// ソルバーがステッカーの向きを無視するかどうか
    pub ignore_orientation: bool,

    /// ソルバーからの結果受信用レシーバ
    pub solver_receiver: Option<Receiver<solver::Solution>>,
    /// ソルバーからの進捗受信用レシーバ
    pub progress_receiver: Option<Receiver<f32>>,

    /// WASM環境でのソルバー起動ペンディング状態
    #[cfg(target_arch = "wasm32")]
    pub pending_solver_start: Option<(SolverTask, bool, u8)>,

    /// WASM環境用: インクリメンタルソルバーの状態
    #[cfg(target_arch = "wasm32")]
    pub solver_state: Option<solver::SolverState>,
}

impl Default for SolverStateManager {
    fn default() -> Self {
        Self {
            solving: false,
            solver_task: SolverTask::Normal,
            solver_progress: 0.0,
            solution_text: String::new(),
            solving_start_time: None,
            last_solve_duration: None,
            ignore_orientation: false,
            solver_receiver: None,
            progress_receiver: None,
            #[cfg(target_arch = "wasm32")]
            pending_solver_start: None,
            #[cfg(target_arch = "wasm32")]
            solver_state: None,
        }
    }
}

impl SolverStateManager {
    /// 新しいソルバー状態を作成
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// ソルバーをキャンセル
    pub fn cancel(&mut self) {
        self.solving = false;
        self.solver_receiver = None;
        self.progress_receiver = None;
        self.solution_text.clear();
        #[cfg(target_arch = "wasm32")]
        {
            self.solver_state = None;
            self.pending_solver_start = None;
        }
    }
}

/// アニメーション状態の管理
pub struct AnimationStateManager {
    /// 現在実行中のアニメーション（実行されていない場合は `None`）
    pub animation: Option<AnimationState>,
    /// 未実行の操作キュー
    pub move_queue: Vec<(Move, Option<EasingMode>, Option<f32>)>,
    /// アニメーションの速度（1手あたりの秒数）
    pub animation_speed: f32,
    /// アニメーション完了後にステップ番号を更新するための保留値
    pub pending_solution_update: Option<isize>,
}

impl Default for AnimationStateManager {
    fn default() -> Self {
        Self {
            animation: None,
            move_queue: Vec::new(),
            animation_speed: 0.3,
            pending_solution_update: None,
        }
    }
}

impl AnimationStateManager {
    /// 新しいアニメーション状態を作成
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 回転操作をキューに追加
    pub fn queue_move(&mut self, mv: Move) {
        self.move_queue.push((mv, None, None));
    }

    /// 複数の回転操作をキューに追加
    pub fn queue_moves(&mut self, moves: Vec<Move>) {
        for mv in moves {
            self.queue_move(mv);
        }
    }

    /// キューをクリア
    pub fn clear_queue(&mut self) {
        self.move_queue.clear();
        self.animation = None;
    }
}

/// UI表示状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    TwoD,
    ThreeD,
    Both,
}

/// 入力状態（通常モード、スキャンモード）
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputState {
    Normal,
    Scanning { face_index: usize },
}

/// UI状態
pub struct UiState {
    /// 現在の表示モード（2D, 3D, Both）
    pub view_mode: ViewMode,
    /// 3Dビューのカメラ・倍率設定
    pub view_3d: View3D,
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
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            view_mode: ViewMode::Both,
            view_3d: View3D::default(),
            input_state: InputState::Normal,
            input_buffer: [None; 24],
            selected_input_color: Color::White,
            input_error_message: String::new(),
            skip_parity_check: false,
        }
    }
}

impl UiState {
    /// 新しいUI状態を作成
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

/// ファイルI/O状態
#[derive(Default)]
pub struct FileIoState {
    /// ファイル読み込み用のレシーバー
    pub file_receiver: Option<Receiver<Result<String, String>>>,
}

impl FileIoState {
    /// 新しいファイルI/O状態を作成
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}
