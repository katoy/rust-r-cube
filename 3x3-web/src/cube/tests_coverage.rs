use super::*;

#[test]
fn test_move_split_to_single() {
    assert_eq!(Move::R2.split_to_single(), Some(Move::R));
    assert_eq!(Move::L2.split_to_single(), Some(Move::L));
    assert_eq!(Move::U2.split_to_single(), Some(Move::U));
    assert_eq!(Move::D2.split_to_single(), Some(Move::D));
    assert_eq!(Move::F2.split_to_single(), Some(Move::F));
    assert_eq!(Move::B2.split_to_single(), Some(Move::B));
    assert_eq!(Move::R.split_to_single(), None);
}

#[test]
fn test_move_display_all() {
    let moves = [
        (Move::R, "R"),
        (Move::Rp, "R'"),
        (Move::R2, "R2"),
        (Move::L, "L"),
        (Move::Lp, "L'"),
        (Move::L2, "L2"),
        (Move::U, "U"),
        (Move::Up, "U'"),
        (Move::U2, "U2"),
        (Move::D, "D"),
        (Move::Dp, "D'"),
        (Move::D2, "D2"),
        (Move::F, "F"),
        (Move::Fp, "F'"),
        (Move::F2, "F2"),
        (Move::B, "B"),
        (Move::Bp, "B'"),
        (Move::B2, "B2"),
        (Move::M, "M"),
        (Move::Mp, "M'"),
        (Move::M2, "M2"),
        (Move::E, "E"),
        (Move::Ep, "E'"),
        (Move::E2, "E2"),
        (Move::S, "S"),
        (Move::Sp, "S'"),
        (Move::S2, "S2"),
        (Move::X, "X"),
        (Move::Xp, "X'"),
        (Move::X2, "X2"),
        (Move::Y, "Y"),
        (Move::Yp, "Y'"),
        (Move::Y2, "Y2"),
        (Move::Z, "Z"),
        (Move::Zp, "Z'"),
        (Move::Z2, "Z2"),
    ];
    for (mv, s) in moves {
        assert_eq!(format!("{}", mv), s);
    }
}

#[test]
fn test_io_from_file_format_errors() {
    // 行数不足
    assert!(Cube::from_file_format("WWWWWWWWW\nGGGGGGGGG").is_err());

    // 1行目のパーツ数不正
    assert!(
        Cube::from_file_format("WWWWWWWW\nGGGGGGGGG RRRRRRRRR BBBBBBBBB OOOOOOOOO\nYYYYYYYYY")
            .is_err()
    );

    // 2行目のパーツ数不正
    assert!(
        Cube::from_file_format("WWWWWWWWW\nGGGGGGGGG RRRRRRRRR BBBBBBBBB OOOOOOOO\nYYYYYYYYY")
            .is_err()
    );

    // 3行目のパーツ数不正
    assert!(
        Cube::from_file_format("WWWWWWWWW\nGGGGGGGGG RRRRRRRRR BBBBBBBBB OOOOOOOOO\nWWWWWWWW")
            .is_err()
    );

    // 無効な文字
    assert!(Cube::from_file_format(
        "WWWWWWWWW\nGGGGGGGGG RRRRRRRRR BBBBBBBBB OOOOOOOOZ\nYYYYYYYYY"
    )
    .is_err());
}

#[test]
fn test_is_solved_with_orientation_mismatch() {
    let cube = Cube::new();
    assert!(cube.is_solved_with_orientation());

    // 色は合っているが向きが違う
    let mut cube_wrong_orient = cube.clone();
    cube_wrong_orient.stickers[0].orientation = (cube_wrong_orient.stickers[0].orientation + 1) % 4;
    assert!(cube_wrong_orient.is_solved());
    assert!(!cube_wrong_orient.is_solved_with_orientation());

    // 色がそもそも違う
    let mut cube_wrong_color = cube.clone();
    cube_wrong_color.stickers[0].color = Color::Yellow;
    assert!(!cube_wrong_color.is_solved());
    assert!(!cube_wrong_color.is_solved_with_orientation());
}

#[test]
fn test_from_colors_and_restore_orientation() {
    let mut solved_colors = [Color::White; 54];
    let faces = [
        (Color::White, 0..9),
        (Color::Yellow, 9..18),
        (Color::Green, 18..27),
        (Color::Blue, 27..36),
        (Color::Red, 36..45),
        (Color::Orange, 45..54),
    ];
    for (color, range) in faces {
        for i in range {
            solved_colors[i] = color;
        }
    }

    // 正常系
    let cube = Cube::from_colors(&solved_colors).unwrap();
    assert!(cube.is_solved_with_orientation());

    // 物理的に不可能なピース配置（白と黄が隣り合うコーナー）
    let mut invalid_colors = solved_colors;
    invalid_colors[2] = Color::Yellow; // UFLコーナーのU(White)をYellowに
    assert!(Cube::from_colors(&invalid_colors).is_err());
}

#[test]
fn test_check_corner_parity_detailed() {
    let cube = Cube::new();

    // 同一コーナー内に同じ色
    let mut c1 = cube.clone();
    c1.stickers[6].color = Color::Green; // UFL corner: index 6, 36, 20.
    c1.stickers[18].color = Color::White; // 他の場所から色を補填
    assert!(c1.is_valid_state().is_err());

    // 同一コーナー内に対面色 (White-Yellow)
    let mut c2 = cube.clone();
    c2.stickers[6].color = Color::Yellow;
    c2.stickers[36].color = Color::White;
    assert!(c2.is_valid_state().is_err());

    // Twist パリティエラー (1コーナーだけ捻る)
    let mut c4 = cube.clone();
    // UBL: [0, 18, 47]  (U, L, B) -> (W, G, O)
    // 捻る: (W, G, O) -> (G, O, W)
    let t = c4.stickers[0].color;
    c4.stickers[0].color = c4.stickers[18].color;
    c4.stickers[18].color = c4.stickers[47].color;
    c4.stickers[47].color = t;
    assert!(c4.is_valid_state().is_err());
}

#[test]
fn test_apply_orientation_solution() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    let solution = crate::solver::Solution {
        moves: vec![Move::Rp],
        found: true,
        message: "test".to_string(),
    };
    assert!(cube.apply_orientation_solution(&solution).is_ok());
}

#[test]
fn test_restore_orientation_errors() {
    let mut cube = Cube::new();
    // センターピースの色を重複させる
    cube.stickers[4].color = Color::Yellow; // Up を Yellow に (Yellow-Yellow)
    assert!(cube.restore_orientation_instantly().is_err());

    let mut cube2 = Cube::new();
    // 物理的に存在しないエッジ（白-黄）
    cube2.stickers[1].color = Color::Yellow; // UB edge: U(1), B(46) -> Yellow, Orange (OK)
    cube2.stickers[46].color = Color::White; // UB edge: Front(1), Back(46) -> Yellow, White (Invalid)
    assert!(cube2.restore_orientation_instantly().is_err());
}
