use rubiks_cube_2x2::cube::{Cube, Move};

fn main() {
    let mut cube = Cube::new();

    // B' L' B' L' F' D F U F R B U'
    let moves = vec![
        Move::Bp, // B'
        Move::Lp, // L'
        Move::Bp, // B'
        Move::Lp, // L'
        Move::Fp, // F'
        Move::D,  // D (assuming [D] means D)
        Move::F,  // F
        Move::U,  // U
        Move::F,  // F
        Move::R,  // R
        Move::B,  // B
        Move::Up, // U'
    ];

    println!("初期状態（完成状態）:");
    print_cube_summary(&cube);

    for (i, &mv) in moves.iter().enumerate() {
        cube.apply_move(mv);
        println!("\nステップ {}: {:?} 実行後:", i + 1, mv);
        print_cube_summary(&cube);

        // 色の重複をチェック
        check_color_duplicates(&cube, i + 1);
    }

    println!("\n最終状態:");
    print_all_colors(&cube);
}

fn print_cube_summary(cube: &Cube) {
    let names = ["Up", "Down", "Left", "Right", "Front", "Back"];
    for (face_idx, name) in names.iter().enumerate() {
        let start = face_idx * 4;
        let colors: Vec<_> = (0..4)
            .map(|i| format!("{:?}", cube.get_sticker(start + i).color))
            .collect();
        println!("  {}: {}", name, colors.join(", "));
    }
}

fn print_all_colors(cube: &Cube) {
    use std::collections::HashMap;
    let mut color_count = HashMap::new();

    for i in 0..24 {
        let color = cube.get_sticker(i).color;
        *color_count.entry(format!("{:?}", color)).or_insert(0) += 1;
    }

    println!("色の分布:");
    for (color, count) in color_count.iter() {
        println!("  {}: {} 個", color, count);
    }
}

fn check_color_duplicates(cube: &Cube, step: usize) {
    use std::collections::HashMap;

    // 各面ごとに色の重複をチェック
    let names = ["Up", "Down", "Left", "Right", "Front", "Back"];
    for (face_idx, name) in names.iter().enumerate() {
        let start = face_idx * 4;
        let mut color_count = HashMap::new();

        for i in 0..4 {
            let color = cube.get_sticker(start + i).color;
            *color_count.entry(format!("{:?}", color)).or_insert(0) += 1;
        }

        // 2つ以上の同じ色があるかチェック
        for (color, count) in color_count.iter() {
            if *count >= 2 {
                println!(
                    "  ⚠️  ステップ{}: {} 面に {} が {} 個!",
                    step, name, color, count
                );
            }
        }
    }
}
