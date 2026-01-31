use rubiks_cube_3x3::cube::{Cube, Move};
use rubiks_cube_3x3::gui::mapping::get_oris_delta;

/// アニメーション開始・終了時の向きの連続性を検証する
///
/// 連続性の条件:
/// (Target_Orientation - oris_delta) % 4 == Source_Orientation
/// かつ、スライス移動では oris_delta = 0 であること。

#[test]
fn test_m_operations_oris_delta() {
    // M 操作は直線的なスライドなので oris_delta = 0
    assert_eq!(
        get_oris_delta(Move::M, 46),
        0,
        "B->U should have oris_delta=0"
    );
    assert_eq!(
        get_oris_delta(Move::M, 10),
        0,
        "D->B should have oris_delta=0"
    );
    assert_eq!(
        get_oris_delta(Move::M, 37),
        0,
        "U->F should have oris_delta=0"
    );
}

#[test]
fn test_l_operations_oris_delta() {
    // L 操作のスライス移動も oris_delta = 0
    assert_eq!(
        get_oris_delta(Move::L, 53),
        0,
        "B->U should have oris_delta=0"
    );
    assert_eq!(
        get_oris_delta(Move::L, 9),
        0,
        "D->B should have oris_delta=0"
    );
}

#[test]
fn test_face_rotation_oris_delta() {
    // 面回転の面内ステッカーは oris_delta = 1
    assert_eq!(
        get_oris_delta(Move::U, 4),
        1,
        "U-center should have oris_delta=1"
    );
    assert_eq!(
        get_oris_delta(Move::F, 40),
        1,
        "F-center should have oris_delta=1"
    );
    assert_eq!(
        get_oris_delta(Move::B, 49),
        1,
        "B-center should have oris_delta=1"
    );
}

#[test]
fn test_frontal_adjacent_oris_delta() {
    // F move adjacent stickers も自転しない仕様（0）に変更
    assert_eq!(
        get_oris_delta(Move::F, 7),
        0,
        "U-Front-Mid should have oris_delta=0"
    );

    // B move adjacent stickers も自転しない仕様（0）に変更
    assert_eq!(
        get_oris_delta(Move::B, 1),
        0,
        "U-Back-Mid should have oris_delta=0"
    );
}
