use rubiks_cube_2x2::cube::{Color, Face, Move};
use std::collections::HashSet;

#[test]
fn test_enums_extra_coverage() {
    // Moved from src/cube/enums.rs
    // Line 90: Gray color
    assert_eq!(Color::from_u8(99), Color::Gray);

    // Move inverse and split coverage
    for m in Move::all_moves() {
        let inv = m.inverse();
        let _ = inv.inverse();
        let _ = m.split_to_single();
    }
}

#[test]
fn test_color_enum() {
    // Moved from tests/cube_tests.rs (now cube_state.rs)
    let c1 = Color::White;
    let c2 = c1;
    assert_eq!(c1, c2);
    let _ = format!("{:?}", c1);

    let mut set = HashSet::new();
    set.insert(c1);
}

#[test]
fn test_move_display() {
    // Moved from tests/cube_tests.rs (now cube_state.rs)
    let moves = [
        (Move::R, "R"),
        (Move::Rp, "R'"),
        (Move::R2, "R2"),
        (Move::L, "L"),
        (Move::Lp, "L'"),
        (Move::L2, "L2"),
        (Move::U, "U"),
        (Move::Up, "U'"),
        (Move::U2, "U2"),
        (Move::D, "D"),
        (Move::Dp, "D'"),
        (Move::D2, "D2"),
        (Move::F, "F"),
        (Move::Fp, "F'"),
        (Move::F2, "F2"),
        (Move::B, "B"),
        (Move::Bp, "B'"),
        (Move::B2, "B2"),
    ];
    for (mv, s) in moves {
        assert_eq!(format!("{}", mv), s);
    }
}

#[test]
fn test_move_split_to_single() {
    // Moved from tests/cube_tests.rs (now cube_state.rs)
    assert_eq!(Move::R2.split_to_single(), Some(Move::R));
    assert_eq!(Move::L2.split_to_single(), Some(Move::L));
    assert_eq!(Move::U2.split_to_single(), Some(Move::U));
    assert_eq!(Move::D2.split_to_single(), Some(Move::D));
    assert_eq!(Move::F2.split_to_single(), Some(Move::F));
    assert_eq!(Move::B2.split_to_single(), Some(Move::B));
    assert_eq!(Move::R.split_to_single(), None);
}

#[test]
fn test_face_enum_basic() {
    assert_eq!(Face::Up as usize, 0);
    assert_eq!(Face::Down as usize, 1);
    assert_eq!(Face::Left as usize, 2);
    assert_eq!(Face::Right as usize, 3);
    assert_eq!(Face::Front as usize, 4);
    assert_eq!(Face::Back as usize, 5);

    assert_eq!(Face::Up.start_index(), 0);
    assert_eq!(Face::Down.start_index(), 4);

    let all = Face::all();
    assert_eq!(all.len(), 6);
    assert!(all.contains(&Face::Up));
}
