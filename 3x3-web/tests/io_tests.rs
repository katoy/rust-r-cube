use rubiks_cube_3x3::cube::{Color, Cube, Face, Move, Sticker};

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

#[test]
fn test_x2_y2_z2_split_to_single() {
    assert_eq!(Move::X2.split_to_single(), Some(Move::X));
    assert_eq!(Move::Y2.split_to_single(), Some(Move::Y));
    assert_eq!(Move::Z2.split_to_single(), Some(Move::Z));
}

#[test]
fn test_all_double_moves_split_to_single() {
    let double_moves = vec![
        (Move::U2, Move::U),
        (Move::D2, Move::D),
        (Move::R2, Move::R),
        (Move::L2, Move::L),
        (Move::F2, Move::F),
        (Move::B2, Move::B),
        (Move::M2, Move::M),
        (Move::E2, Move::E),
        (Move::S2, Move::S),
        (Move::X2, Move::X),
        (Move::Y2, Move::Y),
        (Move::Z2, Move::Z),
    ];

    for (double_move, expected_half) in double_moves {
        assert_eq!(
            double_move.split_to_single(),
            Some(expected_half),
            "{:?}.split_to_single() should return Some({:?})",
            double_move,
            expected_half
        );
    }
}

#[test]
fn test_single_moves_split_to_single_none() {
    let single_moves = vec![
        Move::U,
        Move::Up,
        Move::D,
        Move::Dp,
        Move::R,
        Move::Rp,
        Move::L,
        Move::Lp,
        Move::F,
        Move::Fp,
        Move::B,
        Move::Bp,
        Move::M,
        Move::Mp,
        Move::E,
        Move::Ep,
        Move::S,
        Move::Sp,
        Move::X,
        Move::Xp,
        Move::Y,
        Move::Yp,
        Move::Z,
        Move::Zp,
    ];

    for single_move in single_moves {
        assert_eq!(
            single_move.split_to_single(),
            None,
            "{:?}.split_to_single() should return None",
            single_move
        );
    }
}

#[test]
fn test_file_format_round_trip() {
    let cube = Cube::new();
    let format = cube.to_file_format();
    let restored = Cube::from_file_format(&format).unwrap();

    for i in 0..54 {
        assert_eq!(
            cube.get_sticker(i).color,
            restored.get_sticker(i).color,
            "idx {} の色が一致しません",
            i
        );
    }
}

#[test]
fn test_file_format_scrambled() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);
    cube.apply_move(Move::Fp);

    let format = cube.to_file_format();
    let restored = Cube::from_file_format(&format).unwrap();

    for i in 0..54 {
        assert_eq!(cube.get_sticker(i).color, restored.get_sticker(i).color);
    }
}

#[test]
fn test_validate_colors_valid() {
    let mut colors = [Color::White; 54];
    let faces = [
        (Color::White, 0),
        (Color::Yellow, 9),
        (Color::Green, 18),
        (Color::Blue, 27),
        (Color::Red, 36),
        (Color::Orange, 45),
    ];
    for (color, start) in faces {
        for i in 0..9 {
            colors[start + i] = color;
        }
    }
    assert!(Cube::validate_colors(&colors).is_ok());
}

#[test]
fn test_to_file_format_structure() {
    let cube = Cube::new();
    let format = cube.to_file_format();
    let lines: Vec<&str> = format.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines[0].starts_with("          "));
    assert_eq!(lines[0].trim().len(), 9);
}

#[test]
fn test_save_load_consistency() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);
    cube.apply_move(Move::F);

    let saved_str = cube.to_file_format();
    let loaded_cube = Cube::from_file_format(&saved_str).expect("Failed to load cube");

    for i in 0..54 {
        assert_eq!(cube.get_sticker(i).color, loaded_cube.get_sticker(i).color);
    }
}

#[test]
fn test_save_load_orientation_restoration() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::B2);
    cube.apply_move(Move::U);

    let saved_str = cube.to_file_format();
    let loaded_cube = Cube::from_file_format(&saved_str).expect("Failed to load");

    assert!(loaded_cube.is_valid_state().is_ok());
    let solution = rubiks_cube_3x3::solver::solve(&loaded_cube, 24, false);
    assert!(solution.found);
}

#[test]
fn test_parse_gray_state() {
    let input =
        "          .........\n......... ......... ......... .........\n          .........\n";
    let cube = Cube::from_file_format(input).expect("Failed to parse gray state");
    assert_eq!(cube.stickers[0].color, Color::Gray);
}

#[test]
fn test_legacy_format_compatibility() {
    let legacy =
        "          WWWWWWWWW\nGGGGGGGGG RRRRRRRRR BBBBBBBBB OOOOOOOOO\n          YYYYYYYYY\n";
    let cube = Cube::from_file_format(legacy).expect("Failed to load 3x3 format");
    assert_eq!(cube.stickers[0].orientation, 0);
}
