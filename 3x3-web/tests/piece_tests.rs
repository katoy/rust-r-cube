use glam::Vec3;
use rubiks_cube_3x3::cube::piece::{
    face_to_local_index, get_initial_pieces, Cubie, CubieSticker, PieceType,
};
use rubiks_cube_3x3::cube::{Color, Face, Sticker};

#[test]
fn test_cubie_new_and_matches() {
    let s_up = CubieSticker {
        initial_normal: Vec3::Y,
        color: Color::White,
    };
    let s_front = CubieSticker {
        initial_normal: Vec3::Z,
        color: Color::Red,
    };

    let p2 = Cubie::new(Vec3::new(0.0, 1.0, 1.0), vec![s_up, s_front]);
    assert_eq!(p2.piece_type, PieceType::Edge);
    assert!(p2.matches_colors(&[Color::White, Color::Red]));
    assert!(!p2.matches_colors(&[Color::White]));
    assert!(!p2.matches_colors(&[Color::White, Color::Blue]));

    let s_right = CubieSticker {
        initial_normal: Vec3::X,
        color: Color::Blue,
    };
    let p3 = Cubie::new(Vec3::new(1.0, 1.0, 1.0), vec![s_up, s_front, s_right]);
    assert_eq!(p3.piece_type, PieceType::Corner);
    assert!(p3.matches_colors(&[Color::White, Color::Red, Color::Blue]));

    let p1 = Cubie::new(Vec3::new(0.0, 1.0, 0.0), vec![s_up]);
    assert_eq!(p1.piece_type, PieceType::Center);
}

#[test]
fn test_cubie_rotate_and_project() {
    let mut p = Cubie::new(
        Vec3::Y,
        vec![CubieSticker {
            initial_normal: Vec3::Y,
            color: Color::White,
        }],
    );
    p.rotate(Vec3::X, std::f32::consts::PI); // 180 degrees
    assert_eq!(p.current_pos, -Vec3::Y);

    let mut stickers = [Sticker::new(Color::Gray); 54];
    p.project_to_stickers(&mut stickers);

    // Face::Down is index 9. center is +4 = 13.
    assert_eq!(stickers[13].color, Color::White);
}

#[test]
fn test_face_to_local_index() {
    assert_eq!(face_to_local_index(Face::Up, Vec3::new(-1.0, 1.0, -1.0)), 0);
    assert_eq!(
        face_to_local_index(Face::Down, Vec3::new(-1.0, -1.0, 1.0)),
        0
    );
    assert_eq!(
        face_to_local_index(Face::Left, Vec3::new(-1.0, 1.0, -1.0)),
        0
    );
    assert_eq!(
        face_to_local_index(Face::Right, Vec3::new(1.0, 1.0, 1.0)),
        0
    );
    assert_eq!(
        face_to_local_index(Face::Front, Vec3::new(-1.0, 1.0, 1.0)),
        0
    );
    assert_eq!(
        face_to_local_index(Face::Back, Vec3::new(1.0, 1.0, -1.0)),
        0
    );
}

#[test]
fn test_get_initial_pieces() {
    let pieces = get_initial_pieces();
    assert_eq!(pieces.len(), 26);
}
