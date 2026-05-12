use super::CubeApp;
use super::SolverTask;
use crate::solver;

#[cfg(not(target_arch = "wasm32"))]
use std::sync::mpsc::channel;
#[cfg(not(target_arch = "wasm32"))]
use std::thread;

#[cfg(target_arch = "wasm32")]
use super::confirm_solver_start;

impl CubeApp {
    /// ソルバーの探索を中止
    pub fn cancel_solve(&mut self) {
        self.solving = false;
        self.solution = None;
        self.solution_text.clear();
        self.solver_receiver = None;
        self.progress_receiver = None;
        self.move_queue.clear();

        #[cfg(target_arch = "wasm32")]
        {
            self.solver_state = None;
        }
    }

    /// ソルバー実行（通常）
    pub fn solve(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            // WASM環境では、状態を変更する前に確認ダイアログを表示
            if !confirm_solver_start() {
                // ユーザーがキャンセルした場合は何もせずに終了
                return;
            }
            self.solve_without_confirm();
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.start_solver_internal(SolverTask::Normal, self.ignore_orientation);
        }
    }

    /// 確認ダイアログなしでソルバーを開始（テスト用および内部用）
    pub fn solve_without_confirm(&mut self) {
        self.start_solver_internal(SolverTask::Normal, self.ignore_orientation);
    }

    /// 向きの自動復元を開始（即時）
    pub fn start_restore_orientation(&mut self) {
        if let Err(e) = self.cube.restore_orientation_instantly() {
            self.input_error_message = format!("向きの復元に失敗しました: {}", e);
        }
    }

    /// ソルバー実行の内部処理
    pub(super) fn start_solver_internal(&mut self, task: SolverTask, ignore_orientation: bool) {
        if self.solving {
            return;
        }

        // WASM環境での処理
        #[cfg(target_arch = "wasm32")]
        {
            // WASM環境: ペンディング状態を設定（2フレーム後に起動）
            // これによりUIが確実に更新される時間を確保
            self.pending_solver_start = Some((task, ignore_orientation, 2));
            self.solving = true;
            self.solver_task = task;
            self.solver_progress = 0.0;
            match task {
                SolverTask::Normal => self.solution_text = "探索中...".to_string(),
            }
            self.solving_start_time = Some(web_time::Instant::now());
            self.solution_cube_state = Some(self.cube.clone());
            self.solution_step = 0;
            return;
        }

        // デスクトップ環境: ここから実際のソルバー処理を開始
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.solving = true;
            self.solver_task = task;
            self.solver_progress = 0.0;

            match task {
                SolverTask::Normal => self.solution_text = "探索中...".to_string(),
            }

            self.solving_start_time = Some(web_time::Instant::now()); // 開始時刻を記録

            // 解法開始時の状態を保存
            self.solution_cube_state = Some(self.cube.clone());
            self.solution_step = 0;

            let cube_clone = self.cube.clone();
            let (tx, rx) = channel();
            let (progress_tx, progress_rx) = channel();
            self.solver_receiver = Some(rx);
            self.progress_receiver = Some(progress_rx);

            // デスクトップ環境: 別スレッドで実行（UIをブロックしない）
            thread::spawn(move || {
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
    }

    /// ソルバーの結果を確認
    pub(super) fn check_solver_result(&mut self) {
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

                match self.solver_task {
                    SolverTask::Normal => {
                        self.apply_solver_result(&solution);
                    }
                }
            }
        }
    }

    /// ソルバーの進捗を確認
    pub(super) fn check_progress(&mut self) {
        if let Some(rx) = &self.progress_receiver {
            while let Ok(progress) = rx.try_recv() {
                self.solver_progress = progress;
            }
        }
    }

    /// ソルバー結果を画面に反映する
    pub(super) fn apply_solver_result(&mut self, solution: &solver::Solution) {
        if solution.found {
            self.solution = Some(solution.moves.clone());
            let duration_text = if let Some(d) = self.last_solve_duration {
                format!(" ({:.2}秒)", d)
            } else {
                String::new()
            };
            self.solution_text = format!("解法: {} 手{}", solution.moves.len(), duration_text);
            self.solution_step = 0;
        } else {
            self.solution = None;
            self.solution_text = "解が見つかりませんでした".to_string();
        }
    }
}
