use rubiks_cube_3x3::statistics::Statistics;
use std::thread;
use std::time::Duration;

#[test]
fn test_statistics_new() {
    let stats = Statistics::new();
    assert_eq!(stats.total_solves, 0);
    assert_eq!(stats.successful_solves, 0);
    assert_eq!(stats.total_solve_time, Duration::ZERO);
    assert_eq!(stats.best_solve_time, None);
    assert_eq!(stats.total_manual_moves, 0);
}

#[test]
fn test_record_solve() {
    let mut stats = Statistics::new();
    let time1 = Duration::from_millis(100);
    stats.record_solve(time1);
    assert_eq!(stats.total_solves, 1);
    assert_eq!(stats.successful_solves, 1);
    assert_eq!(stats.total_solve_time, time1);
    assert_eq!(stats.best_solve_time, Some(time1));

    let time2 = Duration::from_millis(50);
    stats.record_solve(time2);
    assert_eq!(stats.total_solves, 2);
    assert_eq!(stats.successful_solves, 2);
    assert_eq!(stats.best_solve_time, Some(time2));

    let time3 = Duration::from_millis(150);
    stats.record_solve(time3);
    assert_eq!(stats.best_solve_time, Some(time2));
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

    stats.record_solve(Duration::from_millis(100));
    stats.record_solve(Duration::from_millis(200));
    assert_eq!(stats.avg_solve_time(), Some(Duration::from_millis(150)));
}

#[test]
fn test_success_rate() {
    let mut stats = Statistics::new();
    assert_eq!(stats.success_rate(), 0.0);

    stats.record_solve(Duration::from_millis(100));
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

#[test]
fn test_statistics_clone_debug() {
    let stats = Statistics::new();
    let _ = stats.clone();
    let debug_str = format!("{:?}", stats);
    assert!(debug_str.contains("Statistics"));
}
