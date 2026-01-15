use rubiks_cube_2x2::cube::{Cube, Move};

fn main() {
    let mut cube = Cube::new();

    println!("初期状態（Right面）:");
    for i in 12..16 {
        let s = cube.get_sticker(i);
        println!("  位置{}: orientation={}", i, s.orientation);
    }
    println!(
        "  パターン: [{}, {}, {}, {}]",
        cube.get_sticker(12).orientation,
        cube.get_sticker(13).orientation,
        cube.get_sticker(14).orientation,
        cube.get_sticker(15).orientation
    );

    // R操作を実行
    cube.apply_move(Move::R);

    println!("\nR操作後（Right面）:");
    for i in 12..16 {
        let s = cube.get_sticker(i);
        println!("  位置{}: orientation={}", i, s.orientation);
    }
    println!(
        "  パターン: [{}, {}, {}, {}]",
        cube.get_sticker(12).orientation,
        cube.get_sticker(13).orientation,
        cube.get_sticker(14).orientation,
        cube.get_sticker(15).orientation
    );

    let clockwise = [1, 2, 0, 3];
    let is_clockwise = (0..4).all(|i| cube.get_sticker(12 + i).orientation == clockwise[i]);

    if is_clockwise {
        println!("\n✅ 時計回りパターンが維持されています");
    } else {
        println!("\n❌ 時計回りパターンが崩れています！");
    }
}
