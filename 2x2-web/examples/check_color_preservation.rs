use rubiks_cube_2x2::cube::{Cube, Move};

fn main() {
    // 完成状態から操作を実行
    let mut cube = Cube::new();

    println!("初期状態（完成状態）:");
    print_cube_colors(&cube);

    // R操作を実行
    cube.apply_move(Move::R);
    println!("\nR操作後:");
    print_cube_colors(&cube);

    // Rp操作で戻す
    cube.apply_move(Move::Rp);
    println!("\nRp操作後（元に戻るはず）:");
    print_cube_colors(&cube);

    // 確認
    let initial = Cube::new();
    let is_same = (0..24).all(|i| cube.get_sticker(i).color == initial.get_sticker(i).color);

    if is_same {
        println!("\n✅ 色の配置は正しく戻りました");
    } else {
        println!("\n❌ 色の配置が壊れています！");

        // 違いを表示
        for i in 0..24 {
            let c1 = cube.get_sticker(i).color;
            let c2 = initial.get_sticker(i).color;
            if c1 != c2 {
                println!("  位置{}: {:?} → {:?}", i, c2, c1);
            }
        }
    }
}

fn print_cube_colors(cube: &Cube) {
    for face in 0..6 {
        let start = face * 4;
        print!("  面{}: ", face);
        for i in 0..4 {
            print!("{:?} ", cube.get_sticker(start + i).color);
        }
        println!();
    }
}
