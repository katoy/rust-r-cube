use std::time::Duration;

use web_time::Instant;

/// アプリケーションの統計情報
#[derive(Debug, Clone)]
pub struct Statistics {
    /// 総解法試行回数
    pub total_solves: usize,

    /// 成功した解法回数
    pub successful_solves: usize,

    /// 総解法時間の累積
    pub total_solve_time: Duration,

    /// 最速解法時間
    pub best_solve_time: Option<Duration>,

    /// 手動操作の総回数
    pub total_manual_moves: usize,

    /// セッション開始時刻
    pub session_start: Instant,
}

impl Default for Statistics {
    fn default() -> Self {
        Self {
            total_solves: 0,
            successful_solves: 0,
            total_solve_time: Duration::ZERO,
            best_solve_time: None,
            total_manual_moves: 0,
            session_start: Instant::now(),
        }
    }
}

impl Statistics {
    /// 新しい統計情報を作成
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 解法成功時に統計を更新
    pub fn record_solve(&mut self, solve_time: Duration) {
        self.total_solves += 1;
        self.successful_solves += 1;
        self.total_solve_time += solve_time;

        if let Some(best) = self.best_solve_time {
            if solve_time < best {
                self.best_solve_time = Some(solve_time);
            }
        } else {
            self.best_solve_time = Some(solve_time);
        }
    }

    /// 解法失敗時に統計を更新
    pub fn record_solve_failure(&mut self) {
        self.total_solves += 1;
    }

    /// 手動操作を記録
    pub fn record_manual_move(&mut self) {
        self.total_manual_moves += 1;
    }

    /// 平均解法時間を計算
    #[must_use]
    pub fn avg_solve_time(&self) -> Option<Duration> {
        if self.successful_solves > 0 {
            // successful_solvesは実用上数百程度なu32へのキャストは安全
            #[allow(clippy::cast_possible_truncation)]
            let count_u32 = self.successful_solves as u32;
            Some(self.total_solve_time / count_u32)
        } else {
            None
        }
    }

    /// 成功率を計算（0.0 ~ 1.0）
    #[must_use]
    pub fn success_rate(&self) -> f64 {
        if self.total_solves > 0 {
            // 統計値は実用上f64の52bit精度で十分
            #[allow(clippy::cast_precision_loss)]
            {
                self.successful_solves as f64 / self.total_solves as f64
            }
        } else {
            0.0
        }
    }

    /// セッション時間を取得
    #[must_use]
    pub fn session_duration(&self) -> Duration {
        self.session_start.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_statistics_default() {
        let stats = Statistics::default();
        assert_eq!(stats.total_solves, 0);
        assert_eq!(stats.successful_solves, 0);
        assert_eq!(stats.total_solve_time, Duration::ZERO);
        assert_eq!(stats.best_solve_time, None);
        assert_eq!(stats.total_manual_moves, 0);
    }

    #[test]
    fn test_statistics_new() {
        let stats = Statistics::new();
        assert_eq!(stats.total_solves, 0);
    }

    #[test]
    fn test_record_solve() {
        let mut stats = Statistics::new();

        let t1 = Duration::from_secs(10);
        stats.record_solve(t1);
        assert_eq!(stats.total_solves, 1);
        assert_eq!(stats.successful_solves, 1);
        assert_eq!(stats.total_solve_time, t1);
        assert_eq!(stats.best_solve_time, Some(t1));

        let t2 = Duration::from_secs(5);
        stats.record_solve(t2);
        assert_eq!(stats.total_solves, 2);
        assert_eq!(stats.successful_solves, 2);
        assert_eq!(stats.total_solve_time, t1 + t2);
        assert_eq!(stats.best_solve_time, Some(t2));

        let t3 = Duration::from_secs(15);
        stats.record_solve(t3);
        assert_eq!(stats.best_solve_time, Some(t2));
    }

    #[test]
    fn test_record_solve_failure() {
        let mut stats = Statistics::new();
        stats.record_solve_failure();
        assert_eq!(stats.total_solves, 1);
        assert_eq!(stats.successful_solves, 0);
    }

    #[test]
    fn test_record_manual_move() {
        let mut stats = Statistics::new();
        stats.record_manual_move();
        assert_eq!(stats.total_manual_moves, 1);
    }

    #[test]
    fn test_avg_solve_time() {
        let mut stats = Statistics::new();
        assert_eq!(stats.avg_solve_time(), None);

        stats.record_solve(Duration::from_secs(10));
        stats.record_solve(Duration::from_secs(20));
        assert_eq!(stats.avg_solve_time(), Some(Duration::from_secs(15)));
    }

    #[test]
    fn test_success_rate() {
        let mut stats = Statistics::new();
        assert_eq!(stats.success_rate(), 0.0);

        stats.record_solve(Duration::from_secs(1));
        assert_eq!(stats.success_rate(), 1.0);

        stats.record_solve_failure();
        assert_eq!(stats.success_rate(), 0.5);
    }

    #[test]
    fn test_session_duration() {
        let stats = Statistics::new();
        thread::sleep(Duration::from_millis(10));
        assert!(stats.session_duration() >= Duration::from_millis(10));
    }
}
