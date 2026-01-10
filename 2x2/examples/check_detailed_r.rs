use rubiks_cube_2x2::cube::{Color, Cube, Move};

fn main() {
    // 完成状態を作成
    let solved = Cube::new();

    // R操作を実行
    let mut cube = solved.clone();
    cube.apply_move(Move::R);

    println!("R操作後の色配置:");
    print_all_stickers(&cube);

    // Rp操作で戻す
    cube.apply_move(Move::Rp);

    println!("\nRp操作で完成状態に戻った:");
    print_all_stickers(&cube);

    // 色が一致するか確認
    let is_same = (0..24).all(|i| cube.get_sticker(i).color == solved.get_sticker(i).color);

    if is_same {
        println!("\n✅ 色の配置は完全に一致");
    } else {
        println!("\n❌ 色の配置が異なる:");
        for i in 0..24 {
            let c1 = cube.get_sticker(i).color;
            let c2 = solved.get_sticker(i).color;
            if c1 != c2 {
                println!("  位置{}: {:?} → {:?}", i, c2, c1);
            }
        }
    }

    // orientationも確認
    println!("\n完成状態のorientation:");
    print_orientations(&solved);

    println!("\nR→Rp後のorientation:");
    print_orientations(&cube);
}

fn print_all_stickers(cube: &Cube) {
    let names = ["Up", "Down", "Left", "Right", "Front", "Back"];
    for (face_idx, name) in names.iter().enumerate() {
        let start = face_idx * 4;
        print!("  {}: ", name);
        for i in 0..4 {
            let s = cube.get_sticker(start + i);
            print!("{:?}({}) ", s.color, s.orientation);
        }
        println!();
    }
}

fn print_orientations(cube: &Cube) {
    for face_idx in 0..6 {
        let start = face_idx * 4;
        let orientations: Vec<u8> = (0..4)
            .map(|i| cube.get_sticker(start + i).orientation)
            .collect();
        println!("  面{}: {:?}", face_idx, orientations);
    }
}
