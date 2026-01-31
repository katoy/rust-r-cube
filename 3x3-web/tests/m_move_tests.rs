use rubiks_cube_3x3::cube::{Color, Cube, Move};

#[test]
fn test_m_move_physical_correctness() {
    // 解決済みキューブから開始
    let mut cube = Cube::new();

    // M 操作前の状態を記録（中央列のステッカー）
    // U面: インデックス 1, 4, 7 (白)
    // F面: インデックス 37, 40, 43 (赤)
    // D面: インデックス 10, 13, 16 (黄)
    // B面: インデックス 52, 49, 46 (橙)

    assert_eq!(cube.get_sticker(1).color, Color::White);
    assert_eq!(cube.get_sticker(37).color, Color::Red);
    assert_eq!(cube.get_sticker(10).color, Color::Yellow);
    assert_eq!(cube.get_sticker(52).color, Color::Orange);

    // すべての向きは 0 のはず
    assert_eq!(cube.get_sticker(1).orientation, 0);
    assert_eq!(cube.get_sticker(37).orientation, 0);
    assert_eq!(cube.get_sticker(10).orientation, 0);
    assert_eq!(cube.get_sticker(52).orientation, 0);

    // M 操作を適用
    cube.apply_move(Move::M);

    // M 操作後の期待される色の配置:
    // U面の中央列(1,4,7)には、B面から来た橙が入るべき
    assert_eq!(
        cube.get_sticker(1).color,
        Color::Orange,
        "U[1] should have Orange from B[52]"
    );
    assert_eq!(
        cube.get_sticker(4).color,
        Color::Orange,
        "U[4] should have Orange from B[49]"
    );
    assert_eq!(
        cube.get_sticker(7).color,
        Color::Orange,
        "U[7] should have Orange from B[46]"
    );

    // F面の中央列(37,40,43)には、U面から来た白が入るべき
    assert_eq!(
        cube.get_sticker(37).color,
        Color::White,
        "F[37] should have White from U[1]"
    );
    assert_eq!(
        cube.get_sticker(40).color,
        Color::White,
        "F[40] should have White from U[4]"
    );
    assert_eq!(
        cube.get_sticker(43).color,
        Color::White,
        "F[43] should have White from U[7]"
    );

    // D面の中央列(10,13,16)には、F面から来た赤が入るべき
    assert_eq!(
        cube.get_sticker(10).color,
        Color::Red,
        "D[10] should have Red from F[37]"
    );
    assert_eq!(
        cube.get_sticker(13).color,
        Color::Red,
        "D[13] should have Red from F[40]"
    );
    assert_eq!(
        cube.get_sticker(16).color,
        Color::Red,
        "D[16] should have Red from F[43]"
    );

    // B面の中央列(52,49,46)には、D面から来た黄が入るべき
    assert_eq!(
        cube.get_sticker(52).color,
        Color::Yellow,
        "B[52] should have Yellow from D[10]"
    );
    assert_eq!(
        cube.get_sticker(49).color,
        Color::Yellow,
        "B[49] should have Yellow from D[13]"
    );
    assert_eq!(
        cube.get_sticker(46).color,
        Color::Yellow,
        "B[46] should have Yellow from D[16]"
    );
}

#[test]
fn test_m_move_orientation() {
    // 解決済みキューブから開始
    let mut cube = Cube::new();

    // M 操作を適用
    cube.apply_move(Move::M);

    // 2D展開図での向きを考慮:
    // - U, F, D は同じ「上」方向を共有
    // - B は展開図上で逆さま
    //
    // 物理的な動き:
    // B -> U: B面は展開図で逆さまなので、U面に来たとき180度回転が必要 (orientation = 2)
    // U -> F: 同じ向きなので回転不要 (orientation = 0)
    // F -> D: 同じ向きなので回転不要 (orientation = 0)
    // D -> B: B面は展開図で逆さまなので、180度回転が必要 (orientation = 2)

    // U面に来た橙ステッカー（元B面）の向き: B面は展開図で逆さまなので 2
    assert_eq!(
        cube.get_sticker(1).orientation,
        2,
        "Orange sticker from B to U should be 2"
    );
    assert_eq!(
        cube.get_sticker(4).orientation,
        2,
        "Orange sticker from B to U should be 2"
    );
    assert_eq!(
        cube.get_sticker(7).orientation,
        2,
        "Orange sticker from B to U should be 2"
    );

    // F面に来た白ステッカー（元U面）の向き: 0
    assert_eq!(
        cube.get_sticker(37).orientation,
        0,
        "White sticker from U to F should keep 0"
    );
    assert_eq!(
        cube.get_sticker(40).orientation,
        0,
        "White sticker from U to F should keep 0"
    );
    assert_eq!(
        cube.get_sticker(43).orientation,
        0,
        "White sticker from U to F should keep 0"
    );

    // D面に来た赤ステッカー（元F面）の向き: 0
    assert_eq!(
        cube.get_sticker(10).orientation,
        0,
        "Red sticker from F to D should keep 0"
    );
    assert_eq!(
        cube.get_sticker(13).orientation,
        0,
        "Red sticker from F to D should keep 0"
    );
    assert_eq!(
        cube.get_sticker(16).orientation,
        0,
        "Red sticker from F to D should keep 0"
    );

    // B面に来た黄ステッカー（元D面）の向き: B面は展開図で逆さまなので 2
    assert_eq!(
        cube.get_sticker(52).orientation,
        2,
        "Yellow sticker from D to B should be 2"
    );
    assert_eq!(
        cube.get_sticker(49).orientation,
        2,
        "Yellow sticker from D to B should be 2"
    );
    assert_eq!(
        cube.get_sticker(46).orientation,
        2,
        "Yellow sticker from D to B should be 2"
    );
}

#[test]
fn test_m_four_times_identity() {
    let mut cube = Cube::new();
    let original = cube.clone();

    // M を4回適用すると元に戻るべき
    cube.apply_move(Move::M);
    cube.apply_move(Move::M);
    cube.apply_move(Move::M);
    cube.apply_move(Move::M);

    // すべてのステッカーが元の位置と向きに戻っているか確認
    for i in 0..54 {
        assert_eq!(
            cube.get_sticker(i).color,
            original.get_sticker(i).color,
            "Sticker {} color should return to原状 after M4",
            i
        );
        assert_eq!(
            cube.get_sticker(i).orientation,
            original.get_sticker(i).orientation,
            "Sticker {} orientation should return to original after M4",
            i
        );
    }
}
