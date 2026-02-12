use rubiks_cube_2x2::cube::Move;
use rubiks_cube_2x2::history::History;

#[test]
fn test_push_and_undo() {
    let mut history = History::new();
    history.push(Move::R);
    history.push(Move::U);

    assert_eq!(history.undo_count(), 2);
    assert_eq!(history.undo(), Some(Move::Up)); // U の逆操作
    assert_eq!(history.undo(), Some(Move::Rp)); // R の逆操作
    assert_eq!(history.undo(), None);
}

#[test]
fn test_redo() {
    let mut history = History::new();
    history.push(Move::R);
    history.undo();

    assert_eq!(history.redo(), Some(Move::R));
    assert_eq!(history.redo(), None);
}

#[test]
fn test_clear_redo_on_new_push() {
    let mut history = History::new();
    history.push(Move::R);
    history.undo();
    assert!(history.can_redo());
    history.push(Move::U);
    assert!(!history.can_redo());
}

#[test]
fn test_history_default_and_clear() {
    let mut history = History::default();
    history.push(Move::R);
    assert_eq!(history.undo_count(), 1);
    history.clear();
    assert_eq!(history.undo_count(), 0);
    assert!(!history.can_undo());
    assert!(!history.can_redo());
}

#[test]
fn test_history_max_size() {
    let mut history = History::with_capacity(2);
    history.push(Move::R);
    history.push(Move::U);
    assert_eq!(history.undo_count(), 2);

    history.push(Move::F); // ここで R が消えるはず
    assert_eq!(history.undo_count(), 2);

    // Undoスタックの中身を確認 (U, F が残っている)
    assert_eq!(history.undo(), Some(Move::Fp));
    assert_eq!(history.undo(), Some(Move::Up));
    assert_eq!(history.undo(), None);
}

#[test]
fn test_history_redo_count() {
    let mut history = History::new();
    history.push(Move::R);
    history.undo();
    assert_eq!(history.undo_count(), 0);
    assert_eq!(history.redo_count(), 1);
}
