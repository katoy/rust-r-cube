use rubiks_cube_2x2::cube::{Cube, Move};

/// L操作とLp操作で元に戻ることを確認
#[test]
fn test_l_then_lp_returns_to_initial() {
    let initial_cube = Cube::new();
    let mut cube = Cube::new();

    // L -> Lp で元に戻るはず
    cube.apply_move(Move::L);
    cube.apply_move(Move::Lp);

    // Cube全体が等しいか確認
    assert_eq!(
        cube, initial_cube,
        "L->Lp後、キューブ全体が初期状態と異なる"
    );
}

/// U操作とUp操作で元に戻ることを確認
#[test]
fn test_u_then_up_returns_to_initial() {
    let initial_cube = Cube::new();
    let mut cube = Cube::new();

    cube.apply_move(Move::U);
    cube.apply_move(Move::Up);

    assert_eq!(
        cube, initial_cube,
        "U->Up後、キューブ全体が初期状態と異なる"
    );
}

/// D操作とDp操作で元に戻ることを確認
#[test]
fn test_d_then_dp_returns_to_initial() {
    let initial_cube = Cube::new();
    let mut cube = Cube::new();

    cube.apply_move(Move::D);
    cube.apply_move(Move::Dp);

    assert_eq!(
        cube, initial_cube,
        "D->Dp後、キューブ全体が初期状態と異なる"
    );
}

/// B操作とBp操作で元に戻ることを確認
#[test]
fn test_b_then_bp_returns_to_initial() {
    let initial_cube = Cube::new();
    let mut cube = Cube::new();

    cube.apply_move(Move::B);
    cube.apply_move(Move::Bp);

    assert_eq!(
        cube, initial_cube,
        "B->Bp後、キューブ全体が初期状態と異なる"
    );
}
