use rubiks_cube_3x3::cube::{Color, Cube, Move};
use rubiks_cube_3x3::solver::get_orientations_vec;

#[test]
fn test_valid_cube_is_valid() {
    let cube = Cube::new();
    assert!(cube.is_valid_state().is_ok());

    let mut scrambled = cube.clone();
    scrambled.scramble(20);
    assert!(scrambled.is_valid_state().is_ok());
}

#[test]
fn test_invalid_color_count() {
    let mut cube = Cube::new();
    cube.stickers[0].color = Color::Yellow; // White -> Yellow. White=8, Yellow=10
    assert!(cube.is_valid_state().is_err());
}

#[test]
fn test_corner_twist_parity_validation() {
    let mut cube = Cube::new();
    // コーナー1つを捻る (UFR: 8, 27, 38)
    let c8 = cube.stickers[8].color;
    let c27 = cube.stickers[27].color;
    let c38 = cube.stickers[38].color;

    cube.stickers[8].color = c27;
    cube.stickers[27].color = c38;
    cube.stickers[38].color = c8;

    // この状態はコーナー捻りパリティが 1 (or 2) になり、エラーになるはず
    assert!(cube.is_valid_state().is_err());
}

#[test]
fn test_edge_flip_parity_validation() {
    let mut cube = Cube::new();
    // エッジ1つを反転 (UR: 5, 28)
    let c5 = cube.stickers[5].color;
    let c28 = cube.stickers[28].color;
    cube.stickers[5].color = c28;
    cube.stickers[28].color = c5;

    // エッジ反転パリティが 1 になり、エラーになるはず
    assert!(cube.is_valid_state().is_err());
}

#[test]
fn test_move_parity_toggle() {
    let base = Cube::new();
    let moves = vec![Move::U, Move::R, Move::F];

    for mv in moves {
        let mut c = base.clone();
        c.apply_move(mv);
        let oris = get_orientations_vec(&c);
        let sum: u32 = oris.iter().map(|&o| o as u32).sum();
        assert!(!sum.is_multiple_of(2));
    }
}

#[test]
fn test_restore_orientation_instantly() {
    let mut cube = Cube::new();
    cube.scramble(5);
    // 向き情報を消去
    let mut test_cube = cube.normalized();
    // 消去された状態でも restore できることを確認（エラーにならないこと）
    test_cube.restore_orientation_instantly().unwrap();

    // restore後は、色の配置に対して矛盾のない方位になっているはず
    // (全センター方位の和が偶数になるなどの基本条件をチェック)
    let oris = get_orientations_vec(&test_cube);
    let sum: u32 = oris.iter().map(|&o| o as u32).sum();
    assert!(
        sum.is_multiple_of(2),
        "Restored orientation sum should be even for a solvable state"
    );
}
