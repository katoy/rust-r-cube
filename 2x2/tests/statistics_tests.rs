use rubiks_cube_2x2::statistics::Statistics;
use std::time::Duration;

#[test]
fn test_statistics_new() {
    let s = Statistics::new();
    assert_eq!(s.total_solves, 0);
    assert_eq!(s.successful_solves, 0);
    assert_eq!(s.total_solve_time, Duration::ZERO);
    assert!(s.best_solve_time.is_none());
    assert_eq!(s.total_manual_moves, 0);
}

#[test]
fn test_statistics_default() {
    let s = Statistics::default();
    assert_eq!(s.total_solves, 0);
    assert_eq!(s.successful_solves, 0);
}

#[test]
fn test_record_solve_first() {
    let mut s = Statistics::new();
    s.record_solve(Duration::from_secs(10));
    assert_eq!(s.total_solves, 1);
    assert_eq!(s.successful_solves, 1);
    assert_eq!(s.best_solve_time, Some(Duration::from_secs(10)));
    assert_eq!(s.total_solve_time, Duration::from_secs(10));
}

#[test]
fn test_record_solve_updates_best_time() {
    let mut s = Statistics::new();
    s.record_solve(Duration::from_secs(10));
    s.record_solve(Duration::from_secs(5)); // より速い → ベスト更新
    assert_eq!(s.best_solve_time, Some(Duration::from_secs(5)));
}

#[test]
fn test_record_solve_keeps_best_time() {
    let mut s = Statistics::new();
    s.record_solve(Duration::from_secs(5));
    s.record_solve(Duration::from_secs(10)); // より遅い → ベスト維持
    assert_eq!(s.best_solve_time, Some(Duration::from_secs(5)));
}

#[test]
fn test_record_solve_failure() {
    let mut s = Statistics::new();
    s.record_solve_failure();
    assert_eq!(s.total_solves, 1);
    assert_eq!(s.successful_solves, 0);
}

#[test]
fn test_record_manual_move() {
    let mut s = Statistics::new();
    s.record_manual_move();
    s.record_manual_move();
    assert_eq!(s.total_manual_moves, 2);
}

#[test]
fn test_avg_solve_time_none() {
    let s = Statistics::new();
    assert!(s.avg_solve_time().is_none());
}

#[test]
fn test_avg_solve_time_some() {
    let mut s = Statistics::new();
    s.record_solve(Duration::from_secs(10));
    s.record_solve(Duration::from_secs(20));
    assert_eq!(s.avg_solve_time(), Some(Duration::from_secs(15)));
}

#[test]
fn test_success_rate_zero_solves() {
    let s = Statistics::new();
    assert_eq!(s.success_rate(), 0.0);
}

#[test]
fn test_success_rate() {
    let mut s = Statistics::new();
    s.record_solve(Duration::from_secs(5));
    s.record_solve_failure();
    let rate = s.success_rate();
    assert!((rate - 0.5).abs() < 1e-10);
}

#[test]
fn test_session_duration() {
    let s = Statistics::new();
    let dur = s.session_duration();
    assert!(dur < Duration::from_secs(60));
}
