use rubiks_cube_3x3::cube::{Color, Cube, Move};
use rubiks_cube_3x3::error::CubeError;

#[test]
fn test_valid_cube_is_valid() {
    let cube = Cube::new();
    assert!(cube.is_valid_state().is_ok());

    let mut scrambled = cube.clone();
    scrambled.scramble(20);
    assert!(scrambled.is_valid_state().is_ok());
}

#[test]
fn test_color_count_invalid() {
    let mut cube = Cube::new();
    // 白を1つ黄色に変える（白8, 黄10になるはず）
    cube.set_sticker_color(0, Color::Yellow);
    let result = cube.is_valid_state();
    assert!(result.is_err());
    if let Err(CubeError::InvalidColors(msg)) = result {
        assert!(msg.contains("White"));
    } else {
        panic!("Should return InvalidColors error");
    }
}

#[test]
fn test_corner_twist_parity_invalid() {
    let mut cube = Cube::new();

    // コーナーを1つだけ捻る (UFR: 8, 38, 27)
    // 8->38, 38->27, 27->8
    let c0 = cube.stickers[8];
    let c1 = cube.stickers[38];
    let c2 = cube.stickers[27];

    cube.stickers[8].color = c1.color;
    cube.stickers[38].color = c2.color;
    cube.stickers[27].color = c0.color;

    let result = cube.is_valid_state();
    assert!(result.is_err());
    println!("Expected error: {:?}", result);
}

#[test]
fn test_edge_flip_parity_invalid() {
    let mut cube = Cube::new();

    // エッジを1つだけ反転させる (UR: 5, 28)
    let c0 = cube.stickers[5];
    let c1 = cube.stickers[28];

    cube.stickers[5].color = c1.color;
    cube.stickers[28].color = c0.color;

    let result = cube.is_valid_state();
    assert!(result.is_err());
    println!("Expected error: {:?}", result);
}

#[test]
fn test_permutation_parity_invalid() {
    let mut cube = Cube::new();

    // 2つのコーナーを入れ替える (UFR: [8, 38, 27] と UFL: [6, 20, 36])
    let s8 = cube.stickers[8];
    let s38 = cube.stickers[38];
    let s27 = cube.stickers[27];

    let s6 = cube.stickers[6];
    let s20 = cube.stickers[20];
    let s36 = cube.stickers[36];

    cube.stickers[8] = s6;
    cube.stickers[38] = s20;
    cube.stickers[27] = s36;

    cube.stickers[6] = s8;
    cube.stickers[20] = s38;
    cube.stickers[36] = s27;

    let result = cube.is_valid_state();
    assert!(result.is_err());
    if let Err(CubeError::InvalidState(msg)) = result {
        assert!(msg.contains("置換パリティ"));
    } else {
        panic!("Should return permutation parity error, got {:?}", result);
    }
}

#[test]
fn test_restore_orientation_instantly() {
    let mut cube = Cube::new();
    cube.scramble(10);

    // ステッカーの向き情報をリセット（色のみの状態にする）
    let mut color_only_cube = cube.normalized();

    // 向きを即座に復元
    color_only_cube
        .restore_orientation_instantly()
        .expect("Should restore orientation successfully");

    // 元のキューブと同じ状態（正規化後）になっているか確認
    assert_eq!(color_only_cube.normalized(), cube.normalized());
    assert!(color_only_cube.is_valid_state().is_ok());
}

#[test]
fn test_force_sync_orientation_to_pieces() {
    let mut cube = Cube::new();
    // センターピースの向きを直接書き換え (U面センター: index 4)
    cube.stickers[4].orientation = 1; // 時計回りに90度

    // 強制同期
    cube.force_sync_orientation_to_pieces();

    // 同期後は stickers 側にも反映され、整合性が保たれるはず
    assert_eq!(cube.get_sticker(4).orientation, 1);

    // 回転操作をしても、その向きが維持されたまま回転するか確認
    cube.apply_move(Move::R);
    assert_eq!(cube.get_sticker(4).orientation, 1);
}
