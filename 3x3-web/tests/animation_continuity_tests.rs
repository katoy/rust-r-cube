use rubiks_cube_3x3::cube::Move;
use rubiks_cube_3x3::gui::mapping::get_oris_delta;

/// アニメーション開始・終了時の向きの連続性を検証する
///
/// 連続性の条件:
/// (Target_Orientation - oris_delta) % 4 == Source_Orientation
/// かつ、スライス移動では oris_delta = 0 であること。

#[test]
fn test_m_operations_oris_delta() {
    // M 操作: B->U は 180度回転 (2)
    assert_eq!(
        get_oris_delta(Move::M, 46),
        2,
        "B->U should have oris_delta=2"
    );
    assert_eq!(
        get_oris_delta(Move::M, 10),
        2,
        "D->B should have oris_delta=2"
    );
    assert_eq!(
        get_oris_delta(Move::M, 37),
        0,
        "U->F should have oris_delta=0"
    );
}

#[test]
fn test_l_operations_oris_delta() {
    // L 操作: B->U は 180度回転 (2)
    assert_eq!(
        get_oris_delta(Move::L, 53),
        2,
        "B->U should have oris_delta=2"
    );
    assert_eq!(
        get_oris_delta(Move::L, 9),
        2,
        "D->B should have oris_delta=2"
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
    // F move adjacent stickers (Up -> Right) は 90度回転 (1)
    assert_eq!(
        get_oris_delta(Move::F, 7),
        1,
        "U-Front-Mid should have oris_delta=1"
    );

    // B move adjacent stickers も 90度回転 (3)
    assert_eq!(
        get_oris_delta(Move::B, 1),
        3,
        "U-Back-Mid should have oris_delta=3"
    );
}
