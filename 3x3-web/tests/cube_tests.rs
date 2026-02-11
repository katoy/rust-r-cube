use rubiks_cube_3x3::cube::{Color, Cube, Move, Sticker};

#[test]
fn test_new_cube_is_solved() {
    let cube = Cube::new();
    assert!(cube.is_solved());
}

#[test]
fn test_move_inverse_all() {
    for mv in Move::all_moves() {
        let mut cube = Cube::new();
        cube.apply_move(mv);
        cube.apply_move(mv.inverse());
        assert!(cube.is_solved(), "Inverse of {:?} failed", mv);
    }
}

#[test]
fn test_move_cycles_four() {
    // 基本的な面回転と全体回転は4回で元に戻る
    let moves = vec![
        Move::U,
        Move::D,
        Move::L,
        Move::R,
        Move::F,
        Move::B,
        Move::M,
        Move::E,
        Move::S,
        Move::X,
        Move::Y,
        Move::Z,
    ];

    for mv in moves {
        let mut cube = Cube::new();
        for _ in 0..4 {
            cube.apply_move(mv);
        }
        assert!(cube.is_solved(), "{:?} applied 4 times should solve", mv);
    }
}

#[test]
fn test_normalization_invariants() {
    let mut cube = Cube::new();
    cube.apply_move(Move::Y);
    assert!(cube.normalized().is_solved());
    cube.apply_move(Move::X);
    assert!(cube.normalized().is_solved());
}

#[test]
fn test_specific_color_shifts() {
    let mut cube = Cube::new();

    // R move: Up -> Back -> Down -> Front -> Up
    cube.apply_move(Move::R);
    assert_eq!(cube.get_sticker(2).color, Color::Red); // U2 was White, now Red (from F)
    assert_eq!(cube.get_sticker(45).color, Color::White); // B0 was Orange, now White (from U)
    assert_eq!(cube.get_sticker(11).color, Color::Orange); // D2 was Yellow, now Orange (from B)
    assert_eq!(cube.get_sticker(38).color, Color::Yellow); // F2 was Red, now Yellow (from D)
}

#[test]
fn test_all_moves_available_count() {
    let moves = Move::all_moves();
    assert_eq!(moves.len(), 36);
}

#[test]
fn test_ru_cycle_105() {
    let mut cube = Cube::new();
    for _ in 0..105 {
        cube.apply_move(Move::R);
        cube.apply_move(Move::U);
    }
    assert!(cube.is_solved());
}

#[test]
fn test_scramble_changes_state() {
    let mut cube = Cube::new();
    cube.scramble(20);
    assert!(!cube.is_solved());
}

#[test]
fn test_sticker_rotation() {
    let mut s = Sticker::new(Color::White);
    s.rotate_cw();
    assert_eq!(s.orientation, 1);
    s.rotate_ccw();
    assert_eq!(s.orientation, 0);
}

#[test]
fn test_check_seq_macro_like_logic() {
    // 以前の check_seq.rs のロジックをテスト
    let mut cube = Cube::new();
    let seq = [Move::R, Move::U, Move::Rp, Move::Up]; // Sexy move
    for _ in 0..6 {
        for &m in &seq {
            cube.apply_move(m);
        }
    }
    assert!(cube.is_solved());
}

#[test]
fn test_comm_property() {
    // 遠い面は交換可能 (R L == L R)
    let mut c1 = Cube::new();
    c1.apply_move(Move::R);
    c1.apply_move(Move::L);

    let mut c2 = Cube::new();
    c2.apply_move(Move::L);
    c2.apply_move(Move::R);

    assert_eq!(c1, c2);
}
