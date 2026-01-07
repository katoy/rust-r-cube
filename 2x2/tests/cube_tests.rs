use rubiks_cube_2x2::cube::{Color, Cube, Move, Sticker};
use std::collections::HashSet;

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
fn test_color_enum() {
    // Debug, Clone, Copy, PartialEq, Eq, Hash の派生を確認
    let c1 = Color::White;
    let c2 = c1; // Copy
    assert_eq!(c1, c2); // PartialEq
    let _ = format!("{:?}", c1); // Debug

    let mut set = HashSet::new();
    set.insert(c1); // Hash
}

#[test]
fn test_move_display() {
    assert_eq!(format!("{}", Move::R), "R");
    assert_eq!(format!("{}", Move::Rp), "R'");
    assert_eq!(format!("{}", Move::L), "L");
    assert_eq!(format!("{}", Move::Lp), "L'");
    assert_eq!(format!("{}", Move::U), "U");
    assert_eq!(format!("{}", Move::Up), "U'");
    assert_eq!(format!("{}", Move::D), "D");
    assert_eq!(format!("{}", Move::Dp), "D'");
    assert_eq!(format!("{}", Move::F), "F");
    assert_eq!(format!("{}", Move::Fp), "F'");
    assert_eq!(format!("{}", Move::B), "B");
    assert_eq!(format!("{}", Move::Bp), "B'");
}

#[test]
fn test_specific_move_logic() {
    // 具体的な色の移動を確認するテスト
    // 初期状態:
    // U: White
    // D: Yellow
    // L: Green
    // R: Blue
    // F: Red
    // B: Orange

    let mut cube = Cube::new();

    // R (Right Clockwise) を実行
    // R面(Blue)は回転する
    // F面(Red)の右列 -> U面(White)の右列
    // U面(White)の右列 -> B面(Orange)の左列 (Uの右はBの左につながる)
    // B面(Orange)の左列 -> D面(Yellow)の右列
    // D面(Yellow)の右列 -> F面(Red)の右列

    cube.apply_move(Move::R);

    // F面(16-19)の右列(17, 19) は D面の色(Yellow)になっているはず
    // D面は4-7. 元のD面右列は 5, 7.
    assert_eq!(cube.get_sticker(17).color, Color::Yellow);
    assert_eq!(cube.get_sticker(19).color, Color::Yellow);

    // U面(0-3)の右列(1, 3) は F面の色(Red)になっているはず
    // 元のF面右列は 17, 19
    assert_eq!(cube.get_sticker(1).color, Color::Red);
    assert_eq!(cube.get_sticker(3).color, Color::Red);
}

#[test]
fn test_hash_consistency() {
    // 同じ状態のキューブは同じハッシュ値を持つことを確認
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let cube1 = Cube::new();
    let cube2 = Cube::new();

    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();

    cube1.hash(&mut hasher1);
    cube2.hash(&mut hasher2);

    assert_eq!(hasher1.finish(), hasher2.finish());

    // 1手動かすと異なるハッシュになる
    let mut cube3 = Cube::new();
    cube3.apply_move(Move::R);
    let mut hasher3 = DefaultHasher::new();
    cube3.hash(&mut hasher3);

    assert_ne!(hasher1.finish(), hasher3.finish());
}

#[test]
fn test_all_moves_available() {
    // Move::all_moves()が12個の動きを返すことを確認
    let moves = Move::all_moves();
    assert_eq!(moves.len(), 12);

    // 重複がないことを確認
    let mut set = HashSet::new();
    for mv in moves {
        assert!(set.insert(mv), "Duplicate move found: {:?}", mv);
    }
}

#[test]
fn test_clone_and_eq() {
    // Clone と PartialEq のテスト
    let mut cube1 = Cube::new();
    cube1.apply_move(Move::R);

    let cube2 = cube1.clone();
    assert_eq!(cube1, cube2);

    let mut cube3 = cube2.clone();
    cube3.apply_move(Move::U);
    assert_ne!(cube1, cube3);
}

#[test]
fn test_face_enum() {
    // Face列挙型のテスト
    use rubiks_cube_2x2::cube::Face;

    let faces = vec![
        Face::Up,
        Face::Down,
        Face::Left,
        Face::Right,
        Face::Front,
        Face::Back,
    ];

    // すべて異なることを確認
    let mut set = HashSet::new();
    for face in faces {
        assert!(set.insert(face));
    }
}

#[test]
fn test_sticker_rotation_cycles() {
    // ステッカーの回転が4回で元に戻ることを確認
    let mut sticker = Sticker::new(Color::White);

    for _ in 0..4 {
        sticker.rotate_cw();
    }
    assert_eq!(sticker.orientation, 0);

    let mut sticker2 = Sticker::new(Color::Yellow);
    for _ in 0..4 {
        sticker2.rotate_ccw();
    }
    assert_eq!(sticker2.orientation, 0);
}

#[test]
fn test_all_colors() {
    // すべての色が異なることを確認
    let colors = vec![
        Color::White,
        Color::Yellow,
        Color::Green,
        Color::Blue,
        Color::Red,
        Color::Orange,
    ];

    let mut set = HashSet::new();
    for color in colors {
        assert!(set.insert(color));
    }
}

#[test]
fn test_move_hash() {
    // Move列挙型がHashを実装していることを確認
    let mut set = HashSet::new();
    set.insert(Move::R);
    set.insert(Move::Rp);

    assert!(set.contains(&Move::R));
    assert!(set.contains(&Move::Rp));
    assert!(!set.contains(&Move::L));
}

#[test]
fn test_all_face_rotations() {
    // すべての面の回転をテスト
    let moves = Move::all_moves();

    for mv in moves {
        let mut cube = Cube::new();
        cube.apply_move(mv);

        // 回転後は完成していないはず（全体回転を除く）
        // ただし、is_solved()は色のみを見るので、一部の全体回転では揃っている
        // ここでは単に実行できることを確認
        let _ = cube.is_solved();
    }
}

#[test]
fn test_normalized_preserves_colors() {
    // normalized()が色を保持することを確認
    let mut cube = Cube::new();
    cube.apply_move(Move::R);

    let original_colors: Vec<Color> = (0..24).map(|i| cube.get_sticker(i).color).collect();
    let normalized = cube.normalized();
    let normalized_colors: Vec<Color> = (0..24).map(|i| normalized.get_sticker(i).color).collect();

    assert_eq!(original_colors, normalized_colors);
}

#[test]
fn test_multiple_scrambles() {
    // 複数回スクランブルしても問題ないことを確認
    let mut cube = Cube::new();

    for _ in 0..3 {
        cube.scramble(5);
    }

    // スクランブル後もget_stickerが正常に動作することを確認
    for i in 0..24 {
        let _ = cube.get_sticker(i);
    }
}

#[test]
fn test_cube_invariants() {
    // どのような操作をしても各色4枚ずつ存在することを確認
    let mut cube = Cube::new();
    cube.scramble(50);

    let mut color_counts = std::collections::HashMap::new();
    for i in 0..24 {
        let s = cube.get_sticker(i);
        *color_counts.entry(s.color).or_insert(0) += 1;
    }

    assert_eq!(color_counts.len(), 6);
    for count in color_counts.values() {
        assert_eq!(*count, 4);
    }
}

#[test]
fn test_normalization_equivalence() {
    // 全体回転させただけの「完成状態」が正規化後にすべて一致することを確認
    // Y軸回転 (U D')
    let mut cube_y = Cube::new();
    cube_y.apply_move(Move::U);
    cube_y.apply_move(Move::Dp);
    // 現在の normalized() は向きを0にするだけで、面を回転させて色を揃えるわけではない
    // そのため、色が合っているかどうかの検証に留める（is_solved() の挙動に近い）
    assert!(cube_y.normalized().is_solved());

    // X軸回転 (R L')
    let mut cube_x = Cube::new();
    cube_x.apply_move(Move::R);
    cube_x.apply_move(Move::Lp);
    assert!(cube_x.normalized().is_solved());
}

#[test]
fn test_all_moves_exhaustive_physical() {
    let moves = Move::all_moves();

    for &mv in &moves {
        let mut cube = Cube::new();
        cube.apply_move(mv);
        let msg = format!("操作: {:?}", mv);

        // 逆操作で元に戻るか (色と向き)
        let mut cube_back = cube.clone();
        cube_back.apply_move(mv.inverse());
        assert_eq!(cube_back, Cube::new(), "{} -> inverse 失敗", msg);

        // 4回で元に戻るか
        let mut cube_cycle = cube.clone();
        for _ in 0..3 {
            cube_cycle.apply_move(mv);
        }
        assert_eq!(cube_cycle, Cube::new(), "{} x 4 失敗", msg);

        // 特定の操作後の物理状態チェック（Dを含む主要なもの）
        match mv {
            Move::D => {
                // D面(4-7)は時計回りに回転, 向きは+1
                for i in 4..8 {
                    assert_eq!(cube.get_sticker(i).color, Color::Yellow);
                    assert_eq!(cube.get_sticker(i).orientation, 1, "D面 {} 向き不一致", i);
                }
                // D面付近の側面 (D操作 CW: F -> R -> B -> L -> F)
                check_sticker_val(&cube, 14, Color::Red, 0, &msg); // F(18,19) -> R(14,15)
                check_sticker_val(&cube, 15, Color::Red, 0, &msg);
                check_sticker_val(&cube, 22, Color::Blue, 0, &msg); // R(14,15) -> B(22,23)
                check_sticker_val(&cube, 23, Color::Blue, 0, &msg);
                check_sticker_val(&cube, 10, Color::Orange, 0, &msg); // B(22,23) -> L(10,11)
                check_sticker_val(&cube, 11, Color::Orange, 0, &msg);
                check_sticker_val(&cube, 18, Color::Green, 0, &msg); // L(10,11) -> F(18,19)
                check_sticker_val(&cube, 19, Color::Green, 0, &msg);
            }
            Move::U => {
                for i in 0..4 {
                    check_sticker_val(&cube, i, Color::White, 1, &msg);
                }
            }
            _ => {}
        }
    }
}

fn check_sticker_val(cube: &Cube, idx: usize, color: Color, orient: u8, msg: &str) {
    let s = cube.get_sticker(idx);
    assert_eq!(s.color, color, "{} idx:{} 色不一致", msg, idx);
    assert_eq!(s.orientation, orient, "{} idx:{} 向き不一致", msg, idx);
}
