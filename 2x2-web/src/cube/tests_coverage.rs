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
    ];
    for (mv, s) in moves {
        assert_eq!(format!("{}", mv), s);
    }
}

#[test]
fn test_io_from_file_format_errors() {
    // 行数不足
    assert!(Cube::from_file_format("WWWW\nGGGG").is_err());

    // 1行目のパーツ数不正
    assert!(Cube::from_file_format("WWW\nGGGG RRRR BBBB OOOO\nYYYY").is_err());

    // 2行目のパーツ数不正
    assert!(Cube::from_file_format("WWWW\nGGGG RRRR BBBB OOO\nYYYY").is_err());

    // 3行目のパーツ数不正
    assert!(Cube::from_file_format("WWWW\nGGGG RRRR BBBB OOOO\nYYY").is_err());

    // 無効な文字
    assert!(Cube::from_file_format("WWWW\nGGGG RRRR BBBB OOOZ\nYYYY").is_err());
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
    let mut solved_colors = [Color::White; 24];
    let faces = [
        (Color::White, 0..4),
        (Color::Yellow, 4..8),
        (Color::Green, 8..12),
        (Color::Blue, 12..16),
        (Color::Red, 16..20),
        (Color::Orange, 20..24),
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

    // 同一コーナー内に同じ色（通常のキューブではありえないが、色数チェックはパスするよう調整）
    let mut c1 = cube.clone();
    c1.stickers[2].color = Color::Green; // UFL corner: U(W), L(G), F(R) -> U(G), L(G), F(R)
    c1.stickers[8].color = Color::White; // 他の場所から色を補填して合計数を合わせる
    assert!(c1.is_valid_state().is_err());

    // 同一コーナー内に対面色 (White-Yellow)
    let mut c2 = cube.clone();
    c2.stickers[2].color = Color::Yellow; // UFL corner: index 2, 9, 16.
    c2.stickers[9].color = Color::White; // Y and W in the same corner!
    assert!(c2.is_valid_state().is_err());

    // コーナーピースの重複 (UFLピースが2つある状態)
    // UFL: W, G, R.  UFR: W, R, B.
    // UFRをUFLと同じ色構成にする
    let mut c3 = cube.clone();
    c3.stickers[3].color = Color::White;
    c3.stickers[17].color = Color::Green;
    c3.stickers[12].color = Color::Red;
    // 不足した色を補う
    c3.stickers[11].color = Color::Blue;
    c3.stickers[20].color = Color::Red; // ... 整合性をとるのが大変なので、適当に重複させる
    assert!(c3.is_valid_state().is_err());

    // Twist パリティエラー (1コーナーだけ捻る = 色を循環させる)
    let mut c4 = cube.clone();
    // UBL: [0, 21, 8]  (U, B, L) -> (W, O, G)
    // 捻る: (W, O, G) -> (G, W, O)
    let t = c4.stickers[0].color;
    c4.stickers[0].color = c4.stickers[8].color;
    c4.stickers[8].color = c4.stickers[21].color;
    c4.stickers[21].color = t;
    assert!(c4.is_valid_state().is_err());
}

#[test]
fn test_apply_orientation_solution() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    let solution = crate::solver::Solution {
        moves: vec![Move::Rp],
        found: true,
    };
    assert!(cube.apply_orientation_solution(&solution).is_ok());
}
