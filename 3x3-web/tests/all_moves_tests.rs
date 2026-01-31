use rubiks_cube_3x3::cube::{Color, Cube, Move};

// ========== L 操作のテスト ==========

#[test]
fn test_l_move_colors() {
    let mut cube = Cube::new();
    cube.apply_move(Move::L);

    // L面自体は回転（詳細は省略）
    // 隣接面の確認:
    // U面左列(0,3,6) <- B面右列(53,50,47) の逆順
    assert_eq!(cube.get_sticker(0).color, Color::Blue);
    assert_eq!(cube.get_sticker(3).color, Color::Blue);
    assert_eq!(cube.get_sticker(6).color, Color::Blue);

    // F面左列(36,39,42) <- U面左列(0,3,6)
    assert_eq!(cube.get_sticker(36).color, Color::White);
    assert_eq!(cube.get_sticker(39).color, Color::White);
    assert_eq!(cube.get_sticker(42).color, Color::White);

    // D面左列(9,12,15) <- F面左列(36,39,42)
    assert_eq!(cube.get_sticker(9).color, Color::Green);
    assert_eq!(cube.get_sticker(12).color, Color::Green);
    assert_eq!(cube.get_sticker(15).color, Color::Green);

    // B面右列(53,50,47) <- D面左列(9,12,15) の逆順
    assert_eq!(cube.get_sticker(53).color, Color::Yellow);
    assert_eq!(cube.get_sticker(50).color, Color::Yellow);
    assert_eq!(cube.get_sticker(47).color, Color::Yellow);
}

#[test]
fn test_l_move_orientation() {
    let mut cube = Cube::new();
    cube.apply_move(Move::L);

    // B -> U: 180度反転 (2)
    assert_eq!(cube.get_sticker(0).orientation, 2);
    assert_eq!(cube.get_sticker(3).orientation, 2);
    assert_eq!(cube.get_sticker(6).orientation, 2);

    // U -> F: 回転なし (0)
    assert_eq!(cube.get_sticker(36).orientation, 0);
    assert_eq!(cube.get_sticker(39).orientation, 0);
    assert_eq!(cube.get_sticker(42).orientation, 0);

    // F -> D: 回転なし (0)
    assert_eq!(cube.get_sticker(9).orientation, 0);
    assert_eq!(cube.get_sticker(12).orientation, 0);
    assert_eq!(cube.get_sticker(15).orientation, 0);

    // D -> B: 180度反転 (2)
    assert_eq!(cube.get_sticker(53).orientation, 2);
    assert_eq!(cube.get_sticker(50).orientation, 2);
    assert_eq!(cube.get_sticker(47).orientation, 2);
}

#[test]
fn test_l_four_times_identity() {
    let mut cube = Cube::new();
    let original = cube.clone();

    for _ in 0..4 {
        cube.apply_move(Move::L);
    }

    for i in 0..54 {
        assert_eq!(cube.get_sticker(i).color, original.get_sticker(i).color);
        assert_eq!(
            cube.get_sticker(i).orientation,
            original.get_sticker(i).orientation
        );
    }
}

// ========== R 操作のテスト ==========

#[test]
fn test_r_move_colors() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);

    // U面右列(2,5,8) <- F面右列(38,41,44)
    assert_eq!(cube.get_sticker(2).color, Color::Green);
    assert_eq!(cube.get_sticker(5).color, Color::Green);
    assert_eq!(cube.get_sticker(8).color, Color::Green);

    // B面左列(45,48,51) <- U面右列(2,5,8) の逆順
    assert_eq!(cube.get_sticker(45).color, Color::White);
    assert_eq!(cube.get_sticker(48).color, Color::White);
    assert_eq!(cube.get_sticker(51).color, Color::White);

    // D面右列(11,14,17) <- B面左列(45,48,51) の逆順
    assert_eq!(cube.get_sticker(11).color, Color::Blue);
    assert_eq!(cube.get_sticker(14).color, Color::Blue);
    assert_eq!(cube.get_sticker(17).color, Color::Blue);

    // F面右列(38,41,44) <- D面右列(11,14,17)
    assert_eq!(cube.get_sticker(38).color, Color::Yellow);
    assert_eq!(cube.get_sticker(41).color, Color::Yellow);
    assert_eq!(cube.get_sticker(44).color, Color::Yellow);
}

#[test]
fn test_r_move_orientation() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);

    // F -> U: 回転なし (0)
    assert_eq!(cube.get_sticker(2).orientation, 0);
    assert_eq!(cube.get_sticker(5).orientation, 0);
    assert_eq!(cube.get_sticker(8).orientation, 0);

    // U -> B: 180度反転 (2)
    assert_eq!(cube.get_sticker(45).orientation, 2);
    assert_eq!(cube.get_sticker(48).orientation, 2);
    assert_eq!(cube.get_sticker(51).orientation, 2);

    // B -> D: 180度反転 (2)
    assert_eq!(cube.get_sticker(11).orientation, 2);
    assert_eq!(cube.get_sticker(14).orientation, 2);
    assert_eq!(cube.get_sticker(17).orientation, 2);

    // D -> F: 回転なし (0)
    assert_eq!(cube.get_sticker(38).orientation, 0);
    assert_eq!(cube.get_sticker(41).orientation, 0);
    assert_eq!(cube.get_sticker(44).orientation, 0);
}

#[test]
fn test_r_four_times_identity() {
    let mut cube = Cube::new();
    let original = cube.clone();

    for _ in 0..4 {
        cube.apply_move(Move::R);
    }

    for i in 0..54 {
        assert_eq!(cube.get_sticker(i).color, original.get_sticker(i).color);
        assert_eq!(
            cube.get_sticker(i).orientation,
            original.get_sticker(i).orientation
        );
    }
}

// ========== U, D, F, B 操作のテスト ==========

#[test]
fn test_u_four_times_identity() {
    let mut cube = Cube::new();
    let original = cube.clone();

    for _ in 0..4 {
        cube.apply_move(Move::U);
    }

    for i in 0..54 {
        assert_eq!(cube.get_sticker(i).color, original.get_sticker(i).color);
        assert_eq!(
            cube.get_sticker(i).orientation,
            original.get_sticker(i).orientation
        );
    }
}

#[test]
fn test_d_four_times_identity() {
    let mut cube = Cube::new();
    let original = cube.clone();

    for _ in 0..4 {
        cube.apply_move(Move::D);
    }

    for i in 0..54 {
        assert_eq!(cube.get_sticker(i).color, original.get_sticker(i).color);
        assert_eq!(
            cube.get_sticker(i).orientation,
            original.get_sticker(i).orientation
        );
    }
}

#[test]
fn test_f_four_times_identity() {
    let mut cube = Cube::new();
    let original = cube.clone();

    for _ in 0..4 {
        cube.apply_move(Move::F);
    }

    for i in 0..54 {
        assert_eq!(cube.get_sticker(i).color, original.get_sticker(i).color);
        assert_eq!(
            cube.get_sticker(i).orientation,
            original.get_sticker(i).orientation
        );
    }
}

#[test]
fn test_b_four_times_identity() {
    let mut cube = Cube::new();
    let original = cube.clone();

    for _ in 0..4 {
        cube.apply_move(Move::B);
    }

    for i in 0..54 {
        assert_eq!(cube.get_sticker(i).color, original.get_sticker(i).color);
        assert_eq!(
            cube.get_sticker(i).orientation,
            original.get_sticker(i).orientation
        );
    }
}

// ========== E, S 操作のテスト ==========

#[test]
fn test_e_four_times_identity() {
    let mut cube = Cube::new();
    let original = cube.clone();

    for _ in 0..4 {
        cube.apply_move(Move::E);
    }

    for i in 0..54 {
        assert_eq!(cube.get_sticker(i).color, original.get_sticker(i).color);
        assert_eq!(
            cube.get_sticker(i).orientation,
            original.get_sticker(i).orientation
        );
    }
}

#[test]
fn test_s_four_times_identity() {
    let mut cube = Cube::new();
    let original = cube.clone();

    for _ in 0..4 {
        cube.apply_move(Move::S);
    }

    for i in 0..54 {
        assert_eq!(cube.get_sticker(i).color, original.get_sticker(i).color);
        assert_eq!(
            cube.get_sticker(i).orientation,
            original.get_sticker(i).orientation
        );
    }
}

// ========== X, Y, Z 操作のテスト ==========

#[test]
fn test_x_four_times_identity() {
    let mut cube = Cube::new();
    let original = cube.clone();

    for _ in 0..4 {
        cube.apply_move(Move::X);
    }

    for i in 0..54 {
        assert_eq!(cube.get_sticker(i).color, original.get_sticker(i).color);
        assert_eq!(
            cube.get_sticker(i).orientation,
            original.get_sticker(i).orientation
        );
    }
}

#[test]
fn test_y_four_times_identity() {
    let mut cube = Cube::new();
    let original = cube.clone();

    for _ in 0..4 {
        cube.apply_move(Move::Y);
    }

    for i in 0..54 {
        assert_eq!(cube.get_sticker(i).color, original.get_sticker(i).color);
        assert_eq!(
            cube.get_sticker(i).orientation,
            original.get_sticker(i).orientation
        );
    }
}

#[test]
fn test_z_four_times_identity() {
    let mut cube = Cube::new();
    let original = cube.clone();

    for _ in 0..4 {
        cube.apply_move(Move::Z);
    }

    for i in 0..54 {
        assert_eq!(cube.get_sticker(i).color, original.get_sticker(i).color);
        assert_eq!(
            cube.get_sticker(i).orientation,
            original.get_sticker(i).orientation
        );
    }
}
