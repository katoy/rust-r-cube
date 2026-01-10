use rubiks_cube_2x2::cube::{Cube, Move};

fn main() {
    // 初期状態（時計回りパターン）
    let base = Cube::new();
    println!("Base cube (Cube::new()):");
    for i in 0..24 {
        let s = base.get_sticker(i);
        print!("idx{}: o{}, ", i, s.orientation);
        if (i + 1) % 4 == 0 {
            println!();
        }
    }
    println!();

    // Y軸回転（U D'）を適用
    let mut rotated = base.clone();
    rotated.apply_move(Move::U);
    rotated.apply_move(Move::Dp);
    println!("After Y-rotation (U D'):");
    for i in 0..24 {
        let s = rotated.get_sticker(i);
        print!("idx{}: o{}, ", i, s.orientation);
        if (i + 1) % 4 == 0 {
            println!();
        }
    }
}
