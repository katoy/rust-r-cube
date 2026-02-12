use rubiks_cube_2x2::cube::{Color, Cube, Face, Move};
use std::collections::HashSet;

/// コーナーキューブの整合性をチェックするヘルパー関数
fn check_corners_integrity(cube: &Cube) -> Result<(), String> {
    let corners = vec![
        ("ULF", vec![2, 9, 16]),  // Up-Left-Front
        ("URF", vec![3, 12, 17]), // Up-Right-Front
        ("ULB", vec![0, 8, 21]),  // Up-Left-Back
        ("URB", vec![1, 13, 20]), // Up-Right-Back
        ("DLF", vec![4, 11, 18]), // Down-Left-Front
        ("DRF", vec![5, 14, 19]), // Down-Right-Front
        ("DLB", vec![6, 10, 23]), // Down-Left-Back
        ("DRB", vec![7, 15, 22]), // Down-Right-Back
    ];

    for (name, indices) in corners {
        let colors: Vec<String> = indices
            .iter()
            .map(|&i| format!("{:?}", cube.get_sticker(i).color))
            .collect();
        let unique: HashSet<&String> = colors.iter().collect();

        if unique.len() != 3 {
            return Err(format!(
                "{}: 異なる色が{}個しかありません {:?} (indices: {:?})",
                name,
                unique.len(),
                colors,
                indices
            ));
        }
    }
    Ok(())
}

#[test]
fn test_all_moves_preserve_corner_integrity() {
    let moves = Move::all_moves();
    for mv in moves {
        let mut cube = Cube::new();
        cube.apply_move(mv);
        if let Err(e) = check_corners_integrity(&cube) {
            panic!("Move {:?} broke corner integrity: {}", mv, e);
        }
    }
}

#[test]
fn test_specific_sequence_corner_integrity() {
    let sequence = vec![
        Move::Bp,
        Move::Lp,
        Move::Bp,
        Move::Lp,
        Move::Fp,
        Move::D,
        Move::F,
        Move::U,
        Move::F,
        Move::R,
        Move::B,
        Move::Up,
    ];
    let mut cube = Cube::new();
    for (i, &mv) in sequence.iter().enumerate() {
        cube.apply_move(mv);
        if let Err(e) = check_corners_integrity(&cube) {
            panic!("Step {} ({:?}) broke corner integrity: {}", i + 1, mv, e);
        }
    }
}

#[test]
fn test_random_scramble_corner_integrity() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let all_moves = Move::all_moves();

    for i in 0..100 {
        let mut cube = Cube::new();
        let num_moves = rng.gen_range(10..30);
        let mut history = Vec::new();

        for _ in 0..num_moves {
            let mv = all_moves[rng.gen_range(0..all_moves.len())];
            cube.apply_move(mv);
            history.push(mv);

            if let Err(e) = check_corners_integrity(&cube) {
                panic!(
                    "Random test failed (trial {}): {}\nHistory: {:?}",
                    i, e, history
                );
            }
        }
    }
}

#[test]
fn test_check_corner_parity_detailed() {
    let cube = Cube::new();

    // 同一コーナー内に同じ色
    let mut c1 = cube.clone();
    c1.stickers[2].color = Color::Green;
    c1.stickers[8].color = Color::White;
    assert!(c1.is_valid_state().is_err());

    // 同一コーナー内に対面色 (White-Yellow)
    let mut c2 = cube.clone();
    c2.stickers[2].color = Color::Yellow;
    c2.stickers[9].color = Color::White;
    assert!(c2.is_valid_state().is_err());

    // Twist パリティエラー
    let mut c4 = cube.clone();
    let t = c4.stickers[0].color;
    c4.stickers[0].color = c4.stickers[8].color;
    c4.stickers[8].color = c4.stickers[21].color;
    c4.stickers[21].color = t;
    assert!(c4.is_valid_state().is_err());
}

#[test]
fn test_restore_orientation_invalid_piece() {
    let mut cube = Cube::new();
    // コーナー 0 (UFL) の色を、物理的に不可能な組み合わせ (White-White-Red) にする
    cube.stickers[2].color = Color::White;
    cube.stickers[16].color = Color::White;
    cube.stickers[9].color = Color::Red;
    assert!(cube.restore_orientation_instantly().is_err());
}

#[test]
fn test_validate_colors_logic() {
    let mut colors = [Color::White; 24];
    for face in Face::all() {
        let start = face.start_index();
        for i in 0..4 {
            colors[start + i] = match face {
                Face::Up => Color::White,
                Face::Down => Color::Yellow,
                Face::Left => Color::Green,
                Face::Right => Color::Blue,
                Face::Front => Color::Red,
                Face::Back => Color::Orange,
            };
        }
    }
    assert!(Cube::validate_colors(&colors).is_ok());

    // 色数不正
    colors[0] = Color::Yellow;
    assert!(Cube::validate_colors(&colors).is_err());
}
