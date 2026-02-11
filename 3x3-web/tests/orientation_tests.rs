use rubiks_cube_3x3::cube::{Cube, Face, Move};
use rubiks_cube_3x3::solver::is_fully_solved;

#[test]
fn test_restore_orientation_basic() {
    let mut cube = Cube::new();
    cube.scramble(10);
    let original_normalized = cube.normalized();

    let mut color_only = cube.normalized();
    color_only.restore_orientation_instantly().unwrap();

    assert_eq!(color_only.normalized(), original_normalized);
}

#[test]
fn test_center_rotations_parity() {
    let mut cube = Cube::new();
    // センター1つを90度回転 (パリティ違反)
    cube.stickers[Face::Up.start_index() + 4].orientation = 1;
    cube.force_sync_orientation_to_pieces();

    let mut color_only = cube.normalized();
    let res = color_only.restore_orientation_instantly();
    // 物理的に不可能な状態でも、何らかの結果を返す（エラーにならない場合もあるが、is_solved にはならないはず）
    println!("Restore result for single center twist: {:?}", res);
}

#[test]
fn test_double_center_rotation_legal() {
    let mut cube = Cube::new();
    // センター2つを互いに90度、-90度回転 (合法)
    cube.stickers[Face::Up.start_index() + 4].orientation = 1;
    cube.stickers[Face::Front.start_index() + 4].orientation = 3;
    cube.force_sync_orientation_to_pieces();

    let mut color_only = cube.normalized();
    color_only
        .restore_orientation_instantly()
        .expect("Should restore legal center parity");

    assert!(is_fully_solved(&color_only.with_clockwise_orientations()));
}

#[test]
fn test_full_piece_orientation_sync() {
    let mut cube = Cube::new();

    // Up面センターピースを特定して回転させる
    let mut found = false;
    for piece in &mut cube.pieces {
        if piece.current_pos.y.round() as i8 == 1
            && piece.current_pos.x.round() as i8 == 0
            && piece.current_pos.z.round() as i8 == 0
        {
            piece.rotate(glam::Vec3::Y, std::f32::consts::FRAC_PI_2);
            found = true;
            break;
        }
    }
    assert!(found, "Up center piece not found");

    cube.sync_stickers();

    // ステッカー側に反映されているか
    let up_center_ori = cube.get_sticker(Face::Up.start_index() + 4).orientation;
    assert_ne!(
        up_center_ori, 0,
        "Up center orientation should have changed"
    );

    // force_sync_orientation_to_pieces で逆方向に同期
    cube.stickers[Face::Up.start_index() + 4].orientation = 2;
    cube.force_sync_orientation_to_pieces();
    assert_eq!(cube.get_sticker(Face::Up.start_index() + 4).orientation, 2);
}

#[test]
fn test_clockwise_orientations_normalization() {
    let mut cube = Cube::new();
    cube.apply_move(Move::Y); // 全体回転

    let normalized = cube.with_clockwise_orientations();
    // Y回転後の解決済み状態は、正規化（回転を戻す）しても解決済みであるべき
    assert!(is_fully_solved(&normalized));
}
