use rubiks_cube_3x3::cube::{Cube, Move};
use rubiks_cube_3x3::gui::mapping::get_oris_delta;

/// 現状の実装における不具合（M 操作で矢印が反転し、アニメーション中に自転する）を検出するテスト
#[test]
fn test_detect_m_move_logic_bug() {
    let mut cube = Cube::new();

    // M 操作を実行
    cube.apply_move(Move::M);

    // U 面中央 (インデックス 4) のステッカーを取得
    let sticker = cube.get_sticker(4);

    // [不具合検出 1] ステッカーの向きが 2 になるのが物理的に正しい（B面はひっくり返るため）
    assert_eq!(
        sticker.orientation, 2,
        "ERROR: M-center orientation should be 2 (due to B-face flip), but got {}.",
        sticker.orientation
    );
}

#[test]
fn test_detect_m_move_animation_bug() {
    // [不具合検出 2] アニメーションの自転（oris_delta）も 2 である必要がある
    // M 操作で 4 に来たもの (src_idx = 49) の delta
    let delta = get_oris_delta(Move::M, 49);

    assert_eq!(
        delta, 2,
        "ERROR: oris_delta for M move onto B-face should be 2 to match physical flip, but got {}.",
        delta
    );
}
