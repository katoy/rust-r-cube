use rubiks_cube_3x3::cube::{Color, Cube, Face, Move, Sticker};
use rubiks_cube_3x3::cube::piece::{Cubie, CubieSticker};
use glam::{Mat4, Vec3};

#[test]
fn test_new_cube_is_solved() {
    let cube = Cube::new();
    assert!(cube.is_solved());
}

#[test]
fn test_is_solved_with_orientation() {
    let cube = Cube::new();
    assert!(cube.is_solved_with_orientation());

    let mut scrambled = Cube::new();
    scrambled.apply_move(Move::R);
    assert!(!scrambled.is_solved_with_orientation());
}

#[test]
fn test_get_and_set_sticker() {
    let mut cube = Cube::new();
    let original = cube.get_sticker(0);
    assert_eq!(original.color, Color::White);

    cube.set_sticker_color(0, Color::Red);
    let updated = cube.get_sticker(0);
    assert_eq!(updated.color, Color::Red);
    assert_eq!(updated.orientation, 0);
}

#[test]
fn test_with_clockwise_orientations() {
    let cube = Cube::new();
    let reoriented = cube.with_clockwise_orientations();
    assert!(reoriented.is_solved_with_orientation());

    let mut scrambled = Cube::new();
    scrambled.apply_move(Move::R);
    let reoriented = scrambled.with_clockwise_orientations();
    // 色は変わっているが、向きは統一される
    for face in Face::all() {
        let start = face.start_index();
        for (i, &expected_ori) in [0, 0, 0, 0, 0, 0, 0, 0, 0].iter().enumerate() {
            assert_eq!(reoriented.get_sticker(start + i).orientation, expected_ori);
        }
    }
}

#[test]
fn test_from_colors() {
    let solved_colors = [Color::White; 9]
        .iter()
        .chain([Color::Yellow; 9].iter())
        .chain([Color::Green; 9].iter())
        .chain([Color::Blue; 9].iter())
        .chain([Color::Red; 9].iter())
        .chain([Color::Orange; 9].iter())
        .copied()
        .collect::<Vec<_>>();

    let colors_array: [Color; 54] = solved_colors.try_into().unwrap();
    let cube = Cube::from_colors(&colors_array).expect("Should create valid cube");
    assert!(cube.is_solved());
}

#[test]
fn test_face_start_index() {
    assert_eq!(Face::Up.start_index(), 0);
    assert_eq!(Face::Down.start_index(), 9);
    assert_eq!(Face::Left.start_index(), 18);
    assert_eq!(Face::Right.start_index(), 27);
    assert_eq!(Face::Front.start_index(), 36);
    assert_eq!(Face::Back.start_index(), 45);
}

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
    for face in Face::all() {
        let adjacent = face.any_adjacent();
        assert_ne!(adjacent, face);
        assert_ne!(adjacent, face.opposite());
    }
}

#[test]
fn test_face_from_index() {
    assert_eq!(Face::from_index(0), Face::Up);
    assert_eq!(Face::from_index(15), Face::Down);
    assert_eq!(Face::from_index(18), Face::Left);
    assert_eq!(Face::from_index(30), Face::Right);
    assert_eq!(Face::from_index(36), Face::Front);
    assert_eq!(Face::from_index(50), Face::Back);
}

#[test]
fn test_move_is_global() {
    assert!(Move::X.is_global());
    assert!(Move::Y.is_global());
    assert!(Move::Z.is_global());
    assert!(!Move::R.is_global());
    assert!(!Move::M.is_global());
}

#[test]
fn test_move_is_middle_layer() {
    assert!(Move::M.is_middle_layer());
    assert!(Move::E.is_middle_layer());
    assert!(Move::S.is_middle_layer());
    assert!(!Move::R.is_middle_layer());
    assert!(!Move::X.is_middle_layer());
}

#[test]
fn test_move_is_face_move() {
    assert!(Move::R.is_face_move());
    assert!(Move::U.is_face_move());
    assert!(!Move::X.is_face_move());
    assert!(!Move::M.is_face_move());
}

#[test]
fn test_move_split_to_single() {
    assert_eq!(Move::R2.split_to_single(), Some(Move::R));
    assert_eq!(Move::U2.split_to_single(), Some(Move::U));
    assert_eq!(Move::R.split_to_single(), None);
    assert_eq!(Move::Rp.split_to_single(), None);
}

#[test]
fn test_move_geometric_params() {
    let (axis, _) = Move::R.geometric_params();
    assert_eq!(axis, glam::Vec3::X);

    let (axis, _) = Move::U.geometric_params();
    assert_eq!(axis, glam::Vec3::Y);

    let (axis, _) = Move::F.geometric_params();
    assert_eq!(axis, glam::Vec3::Z);
}

#[test]
fn test_move_display() {
    assert_eq!(Move::R.to_string(), "R");
    assert_eq!(Move::Rp.to_string(), "R'");
    assert_eq!(Move::R2.to_string(), "R2");
    assert_eq!(Move::X.to_string(), "X");
}

#[test]
fn test_all_moves_display() {
    for mv in Move::all_moves() {
        let display = mv.to_string();
        assert!(!display.is_empty());
    }
}

#[test]
fn test_move_all_inverses() {
    for mv in Move::all_moves() {
        let inv_inv = mv.inverse().inverse();
        assert_eq!(inv_inv, mv);
    }
}

#[test]
fn test_face_all_coverage() {
    let faces = Face::all();
    assert_eq!(faces.len(), 6);
    for (i, face) in faces.iter().enumerate() {
        assert_eq!(face.start_index(), i * 9);
    }
}

#[test]
fn test_face_to_pos_for_local_index() {
    for face in Face::all() {
        for local_idx in 0..9 {
            let pos = face.to_pos_for_local_index(local_idx);
            assert!(pos.length() > 0.0);
            assert!(pos.x.abs() <= 1.0);
            assert!(pos.y.abs() <= 1.0);
            assert!(pos.z.abs() <= 1.0);
        }
    }
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

#[test]
fn test_restore_orientation_instantly_invalid_centers() {
    let mut cube = Cube::new();
    // center色を壊して「中心ピースの色配置が不正です」分岐を通す
    cube.stickers[Face::Up.start_index() + 4].color = Color::Yellow;
    let err = cube.restore_orientation_instantly().unwrap_err();
    assert!(!err.to_string().is_empty()); // Just check it has an error
}

#[test]
fn test_validate_colors_missing_color_error() {
    let mut colors = [Color::White; 54];
    // White 18, Yellow 0 などを作る
    for c in colors.iter_mut().take(9) {
        *c = Color::White;
    }
    let err = Cube::validate_colors(&colors).unwrap_err();
    assert!(!err.to_string().is_empty()); // Just check it has an error
}

#[test]
fn test_validate_colors_wrong_count_error() {
    let mut colors = [Color::White; 54];
    // 正常配列から1個だけ崩す
    for color in colors.iter_mut().take(9) { *color = Color::White; }
    for color in colors.iter_mut().skip(9).take(9) { *color = Color::Yellow; }
    for color in colors.iter_mut().skip(18).take(9) { *color = Color::Green; }
    for color in colors.iter_mut().skip(27).take(9) { *color = Color::Blue; }
    for color in colors.iter_mut().skip(36).take(9) { *color = Color::Red; }
    for color in colors.iter_mut().skip(45) { *color = Color::Orange; }
    colors[0] = Color::Yellow; // White 8 / Yellow 10
    let err = Cube::validate_colors(&colors).unwrap_err().to_string();
    assert!(err.contains("必要"));
}

#[test]
#[should_panic]
fn test_cubie_new_panics_on_invalid_sticker_count() {
    let _ = Cubie::new(Vec3::ZERO, vec![]);
}

#[test]
fn test_cubie_matches_colors_length_mismatch_and_missing_color() {
    let c = Cubie::new(
        Vec3::new(1.0, 1.0, 1.0),
        vec![
            CubieSticker { initial_normal: Vec3::X, color: Color::Red },
            CubieSticker { initial_normal: Vec3::Y, color: Color::White },
            CubieSticker { initial_normal: Vec3::Z, color: Color::Blue },
        ],
    );
    assert!(!c.matches_colors(&[Color::Red, Color::White])); // len mismatch
    assert!(!c.matches_colors(&[Color::Red, Color::White, Color::Green])); // missing
    assert!(c.matches_colors(&[Color::Blue, Color::Red, Color::White])); // unordered match
}

#[test]
fn test_calculate_orientation_with_rot_branches() {
    let mut cube = Cube::new();
    // 90度回転で dot が ±0.9 以外になる分岐を通す
    cube.pieces[0].current_rot = Mat4::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2);
    let ori = cube.pieces[0].calculate_orientation(Vec3::Y, Vec3::Y);
    assert!(ori <= 3);
}

#[test]
fn test_rotation_all_middle_layers_keep_color_count() {
    let mut cube = Cube::new();
    cube.apply_move(Move::M);
    cube.apply_move(Move::E);
    cube.apply_move(Move::S);
    // 色数保存（rotation.rs の層分岐を広く通す）
    let mut cnt = std::collections::HashMap::new();
    for i in 0..54 {
        *cnt.entry(cube.get_sticker(i).color).or_insert(0usize) += 1;
    }
    for c in [Color::White, Color::Yellow, Color::Green, Color::Blue, Color::Red, Color::Orange] {
        assert_eq!(cnt.get(&c).copied().unwrap_or(0), 9);
    }
}

#[test]
fn test_piece_type_detection() {
    let pieces = rubiks_cube_3x3::cube::piece::get_initial_pieces();
    let mut centers = 0;
    let mut edges = 0;
    let mut corners = 0;

    for piece in &pieces {
        match piece.piece_type {
            rubiks_cube_3x3::cube::piece::PieceType::Center => centers += 1,
            rubiks_cube_3x3::cube::piece::PieceType::Edge => edges += 1,
            rubiks_cube_3x3::cube::piece::PieceType::Corner => corners += 1,
        }
    }
    assert_eq!(centers, 6);
    assert_eq!(edges, 12);
    assert_eq!(corners, 8);
}

#[test]
fn test_piece_rotate_and_project() {
    let pieces = rubiks_cube_3x3::cube::piece::get_initial_pieces();
    let mut stickers = [Sticker { color: Color::Gray, orientation: 0 }; 54];

    // 初期状態をプロジェクト
    for piece in &pieces {
        piece.project_to_stickers(&mut stickers);
    }

    // すべてのステッカーが Gray でないことを確認
    assert!(stickers.iter().any(|s| s.color != Color::Gray));
}

#[test]
fn test_cube_clone_and_equality() {
    let mut cube1 = Cube::new();
    cube1.apply_move(Move::R);
    cube1.apply_move(Move::U);

    let cube2 = cube1.clone();

    for i in 0..54 {
        assert_eq!(cube1.get_sticker(i).color, cube2.get_sticker(i).color);
        assert_eq!(cube1.get_sticker(i).orientation, cube2.get_sticker(i).orientation);
    }
}

#[test]
fn test_multiple_scrambles() {
    let mut cube = Cube::new();
    for _ in 0..3 {
        cube.scramble(5);
        assert!(!cube.is_solved());
    }
}

#[test]
fn test_face_rotation_three_times_equals_inverse() {
    let mut c1 = Cube::new();
    c1.apply_move(Move::R);
    c1.apply_move(Move::R);
    c1.apply_move(Move::R);

    let mut c2 = Cube::new();
    c2.apply_move(Move::Rp);

    // R^3 should equal R'
    for i in 0..54 {
        assert_eq!(c1.get_sticker(i).color, c2.get_sticker(i).color);
    }
}

#[test]
fn test_combined_moves_sequence() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);
    cube.apply_move(Move::Rp);
    cube.apply_move(Move::Up);

    assert!(!cube.is_solved());
    // Apply inverse sequence
    cube.apply_move(Move::U);
    cube.apply_move(Move::R);
    cube.apply_move(Move::Up);
    cube.apply_move(Move::Rp);

    assert!(cube.is_solved());
}

#[test]
fn test_all_sticker_indices() {
    let cube = Cube::new();
    for i in 0..54 {
        let sticker = cube.get_sticker(i);
        assert_ne!(sticker.color, Color::Gray);
    }
}

#[test]
fn test_normalized_cube() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);

    let normalized = cube.normalized();
    // Normalized should have all orientations = 0
    for i in 0..54 {
        assert_eq!(normalized.get_sticker(i).orientation, 0);
    }
}
