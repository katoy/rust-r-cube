use rubiks_cube_3x3::cube::Move;
use rubiks_cube_3x3::history::History;

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
fn test_capacity_limit() {
    let mut history = History::with_capacity(2);
    history.push(Move::R);
    history.push(Move::U);
    history.push(Move::F);

    assert_eq!(history.undo_count(), 2);
    assert_eq!(history.undo(), Some(Move::Fp));
    assert_eq!(history.undo(), Some(Move::Up));
    assert_eq!(history.undo(), None);
}

#[test]
fn test_clear() {
    let mut history = History::new();
    history.push(Move::R);
    history.undo();
    history.clear();
    assert!(!history.can_undo());
    assert!(!history.can_redo());
    assert_eq!(history.undo_count(), 0);
    assert_eq!(history.redo_count(), 0);
}

#[test]
fn test_default() {
    let history = History::default();
    assert_eq!(history.undo_count(), 0);
}
