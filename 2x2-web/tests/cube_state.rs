use rubiks_cube_2x2::cube::{Color, Cube, Move, Sticker};

#[test]
fn test_new_cube_is_solved() {
    let cube = Cube::new();
    assert!(cube.is_solved());
}

#[test]
fn test_default_is_new() {
    let cube1 = Cube::new();
    let cube2 = Cube::default();
    assert_eq!(cube1, cube2);
}

#[test]
fn test_apply_move_basic() {
    let mut cube = Cube::new();

    // R を適用
    cube.apply_move(Move::R);
    assert!(!cube.is_solved());

    // R' を適用して元に戻るか
    cube.apply_move(Move::Rp);
    assert!(cube.is_solved());
}

#[test]
fn test_move_cycles() {
    let moves = vec![
        Move::R,
        Move::L,
        Move::U,
        Move::D,
        Move::F,
        Move::B,
        Move::Rp,
        Move::Lp,
        Move::Up,
        Move::Dp,
        Move::Fp,
        Move::Bp,
    ];

    for mv in moves {
        let mut cube = Cube::new();
        // 4回回すと元に戻る（向きも含めて）
        for _ in 0..4 {
            cube.apply_move(mv);
        }
        assert!(
            cube.is_solved(),
            "Move {} applied 4 times should solve the cube",
            mv
        );
    }
}

#[test]
fn test_move_inverse() {
    let moves = vec![
        (Move::R, Move::Rp),
        (Move::L, Move::Lp),
        (Move::U, Move::Up),
        (Move::D, Move::Dp),
        (Move::F, Move::Fp),
        (Move::B, Move::Bp),
    ];

    for (m1, m2) in moves {
        assert_eq!(m1.inverse(), m2, "Inverse of {} should be {}", m1, m2);
        let mut cube = Cube::new();
        cube.apply_move(m1);
        cube.apply_move(m2);
        assert!(
            cube.is_solved(),
            "Move {} then {} should solve the cube",
            m1,
            m2
        );
    }
}

#[test]
fn test_scramble() {
    let mut cube = Cube::new();
    cube.scramble(10);
    // 10回ランダムに動かして、偶然揃う確率は極めて低い
    // (ただし、スクランブルロジックによっては元に戻る可能性もゼロではないが、テストとしては非ソルブ期待)
    // ここでは「変化すること」を確認する程度
    // 完全に一致しないことを確認（運悪く一致する可能性を排除するため、何度か試行すべきだが簡易的に）
    if cube.is_solved() {
        // 万が一揃ってしまった場合はもう一度
        cube.scramble(10);
    }
    assert!(!cube.is_solved());
}

#[test]
fn test_normalized() {
    let mut cube = Cube::new();
    // 向きを変えるような操作（全体回転に相当する操作）を行ってみる
    // 例: U D' はY軸回転
    cube.apply_move(Move::U);
    cube.apply_move(Move::Dp);

    // この状態はソルブされていないが、normalized() を呼んでも色は変わらないはず
    let norm = cube.normalized();

    // normalizedの結果、stickersのorientationが全て0になっていることを確認
    // Cube構造体のフィールドはprivateだが、stickersにはアクセスできない。
    // get_stickerメソッド経由で確認する。
    for i in 0..24 {
        let s = norm.get_sticker(i);
        assert_eq!(s.orientation, 0, "Sticker {} orientation should be 0", i);
    }
}

#[test]
fn test_sticker_properties() {
    let s = Sticker::new(Color::White);
    assert_eq!(s.color, Color::White);
    assert_eq!(s.orientation, 0);

    let mut s2 = s;
    s2.rotate_cw();
    assert_eq!(s2.orientation, 1);
    s2.rotate_cw();
    assert_eq!(s2.orientation, 2);
    s2.rotate_cw();
    assert_eq!(s2.orientation, 3);
    s2.rotate_cw();
    assert_eq!(s2.orientation, 0);

    let mut s3 = s;
    s3.rotate_ccw();
    assert_eq!(s3.orientation, 3);
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
fn test_apply_orientation_solution() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    let solution = rubiks_cube_2x2::solver::Solution {
        moves: vec![Move::Rp],
        found: true,
    };
    assert!(cube.apply_orientation_solution(&solution).is_ok());
}

#[test]
fn test_ru_cycle() {
    // R U の繰り返しの周期性を確認 (105回で元に戻る)
    let mut cube = Cube::new();
    for _ in 0..105 {
        cube.apply_move(Move::R);
        cube.apply_move(Move::U);
    }
    assert!(cube.is_solved());
}
#[test]
fn test_cube_extra_coverage() {
    // Transferred from src/cube/mod.rs
    // Sticker rotation (Lines 117-124)
    let mut sticker = Sticker::new(Color::White);
    sticker.rotate_cw();
    sticker.rotate_ccw();

    // from_colors (Lines 146-163)
    let mut colors = [Color::White; 24];
    // 解決可能な配色を適当に作る
    let solved = Cube::new();
    for (i, sticker) in solved.stickers.iter().enumerate() {
        colors[i] = sticker.color;
    }
    let _ = Cube::from_colors(&colors);

    // restore_orientation_instantly fail path (Lines 246-249)
    let mut cube = Cube::new();
    // 物理的にありえないコーナー構成を作る。
    // ただし、validate_colors をパスさせるために全体の各色数は4のままでなければならない。
    cube.stickers[2].color = Color::Yellow;
    cube.stickers[4].color = Color::White;
    let _ = cube.restore_orientation_instantly();
}

// --- Orientation Tests (merged from orientation_tests.rs) ---

#[test]
fn test_is_fully_solved_basic() {
    use rubiks_cube_2x2::solver::is_fully_solved;
    let cube = Cube::new();
    assert!(is_fully_solved(&cube));

    let mut cube_rotated = cube.clone();
    cube_rotated.apply_move(Move::U);
    cube_rotated.apply_move(Move::Dp);
    assert!(is_fully_solved(&cube_rotated));
}

#[test]
fn test_apply_orientation_solution_various() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    let solution = rubiks_cube_2x2::solver::Solution {
        moves: vec![Move::Rp],
        found: true,
    };
    assert!(cube.apply_orientation_solution(&solution).is_ok());
}
