use super::CubeApp;

impl CubeApp {
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
}
