use rubiks_cube_2x2::statistics::Statistics;
use std::thread;
use std::time::Duration;

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
