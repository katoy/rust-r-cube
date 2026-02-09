use rubiks_cube_3x3::cube::{Color, Cube, Move};

#[test]
fn test_apply_move_all_cycles() {
    let mut cube = Cube::new();
    let moves = [Move::U, Move::R, Move::F, Move::D, Move::L, Move::B];
    for &mv in &moves {
        for _ in 0..4 {
            cube.apply_move(mv);
        }
    }
    assert!(cube.is_solved());
}

#[test]
fn test_m_rotation() {
    let mut cube = Cube::new();
    cube.apply_move(Move::M);
    // M 操作後、センターピースが移動していることを期待
    // U center (4) -> F (40), F (40) -> D (13), D (13) -> B (49), B (49) -> U (4)
    // 色を確認
    assert_eq!(cube.stickers[40].color, Color::White);
    assert_eq!(cube.stickers[13].color, Color::Red);
    assert_eq!(cube.stickers[49].color, Color::Yellow);
    assert_eq!(cube.stickers[4].color, Color::Orange);

    cube.apply_move(Move::M);
    cube.apply_move(Move::M);
    cube.apply_move(Move::M);
    assert!(cube.is_solved());
}
