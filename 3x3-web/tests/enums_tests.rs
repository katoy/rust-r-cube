use rubiks_cube_3x3::cube::{Color, Face, Move, Sticker};

#[test]
fn test_face_opposite() {
    assert_eq!(Face::Up.opposite(), Face::Down);
    assert_eq!(Face::Down.opposite(), Face::Up);
    assert_eq!(Face::Left.opposite(), Face::Right);
    assert_eq!(Face::Right.opposite(), Face::Left);
    assert_eq!(Face::Front.opposite(), Face::Back);
    assert_eq!(Face::Back.opposite(), Face::Front);
}

#[test]
fn test_face_any_adjacent() {
    for f in Face::all() {
        let adj = f.any_adjacent();
        assert_ne!(f, adj);
        assert_ne!(f.opposite(), adj);
    }
}

#[test]
fn test_face_to_pos_for_local_index() {
    for f in Face::all() {
        for i in 0..9 {
            let pos = f.to_pos_for_local_index(i);
            assert!(pos.length() > 0.0);
        }
    }
}

#[test]
fn test_face_from_index() {
    assert_eq!(Face::from_index(0), Face::Up);
    assert_eq!(Face::from_index(9), Face::Down);
    assert_eq!(Face::from_index(18), Face::Left);
    assert_eq!(Face::from_index(27), Face::Right);
    assert_eq!(Face::from_index(36), Face::Front);
    assert_eq!(Face::from_index(45), Face::Back);
    assert_eq!(Face::from_index(54), Face::Up); // Default case
}

#[test]
fn test_move_properties() {
    assert!(!Move::R.is_global());
    assert!(!Move::R.is_middle_layer());
    assert!(Move::R.is_face_move());

    assert!(Move::X.is_global());
    assert!(!Move::X.is_middle_layer());
    assert!(!Move::X.is_face_move());

    assert!(!Move::M.is_global());
    assert!(Move::M.is_middle_layer());
    assert!(!Move::M.is_face_move());
}

#[test]
fn test_move_split() {
    assert_eq!(Move::R2.split_to_single(), Some(Move::R));
    assert_eq!(Move::R.split_to_single(), None);
}

#[test]
fn test_move_geometric_params() {
    for m in Move::all_moves() {
        let (axis, angle) = m.geometric_params();
        assert!(axis.length() > 0.0);
        assert!(angle != 0.0);
    }
}

#[test]
fn test_move_display() {
    assert_eq!(format!("{}", Move::R), "R");
    assert_eq!(format!("{}", Move::Rp), "R'");
    assert_eq!(format!("{}", Move::R2), "R2");
}

#[test]
fn test_sticker_rotate() {
    let mut s = Sticker::new(Color::White);
    s.rotate_cw();
    assert_eq!(s.orientation, 1);
    s.rotate_ccw();
    assert_eq!(s.orientation, 0);
}
