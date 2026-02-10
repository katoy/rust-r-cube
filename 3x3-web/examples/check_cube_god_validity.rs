use rubiks_cube_3x3::cube::{Color, Cube};
use std::fs;

fn main() {
    println!("=== cube_god.txt の向き復元プロセスの詳細調査 ===\n");

    // ファイルから読み込み
    let cube_text =
        fs::read_to_string("cubes/cube_god.txt").expect("cube_god.txt を読み込めませんでした");

    println!("1. ファイル内容（色のみ）:\n{}\n", cube_text);

    // 手動で色配列を作成（restore_orientation_instantly を呼ばない方法を試す）
    let mut colors = [Color::White; 54];

    // ファイルから色をパース
    let lines: Vec<&str> = cube_text.lines().collect();
    let parse_colors = |s: &str| -> Vec<Color> {
        s.chars()
            .filter(|c| !c.is_whitespace())
            .map(|c| match c.to_ascii_uppercase() {
                'W' => Color::White,
                'Y' => Color::Yellow,
                'G' => Color::Green,
                'B' => Color::Blue,
                'R' => Color::Red,
                'O' => Color::Orange,
                _ => Color::Gray,
            })
            .collect()
    };

    let line1_colors = parse_colors(lines[0]);
    let line2_colors = parse_colors(lines[1]);
    let line3_colors = parse_colors(lines[2]);

    // 色配列を構築
    for i in 0..9 {
        colors[i] = line1_colors[i]; // Up
    }
    for i in 0..9 {
        colors[9 + i] = line3_colors[i]; // Down
    }
    for i in 0..9 {
        colors[18 + i] = line2_colors[i]; // Left
    }
    for i in 0..9 {
        colors[27 + i] = line2_colors[9 + i]; // Front
    }
    for i in 0..9 {
        colors[36 + i] = line2_colors[18 + i]; // Right
    }
    for i in 0..9 {
        colors[45 + i] = line2_colors[27 + i]; // Back
    }

    println!("2. 色配列:");
    for (i, color) in colors.iter().enumerate() {
        print!("{:?} ", color);
        if (i + 1) % 9 == 0 {
            println!();
        }
    }
    println!();

    // 通常の方法でキューブを作成（restore_orientation_instantly が呼ばれる）
    let cube = Cube::from_file_format(&cube_text).expect("キューブのパースに失敗しました");

    println!("3. restore_orientation_instantly() 後のキューブ:");
    println!("{}\n", cube.to_file_format());

    println!("4. 設定された向き:");
    for i in 0..54 {
        let sticker = cube.get_sticker(i);
        print!("idx{:2}:ori={} ", i, sticker.orientation);
        if (i + 1) % 6 == 0 {
            println!();
        }
    }
    println!();

    // 妥当性チェック
    println!("\n5. 妥当性チェック:");
    match cube.is_valid_state() {
        Ok(()) => println!("✓ is_valid_state(): OK"),
        Err(e) => println!("✗ is_valid_state(): ERROR - {}", e),
    }

    // 完成状態のキューブと比較
    println!("\n6. 完成状態との比較:");
    let solved = Cube::new();
    println!("完成状態の向き:");
    for i in 0..54 {
        let sticker = solved.get_sticker(i);
        print!("idx{:2}:ori={} ", i, sticker.orientation);
        if (i + 1) % 6 == 0 {
            println!();
        }
    }
    println!();

    println!("\n7. センターピースの色:");
    println!("Up(idx 4): {:?}", cube.get_sticker(4).color);
    println!("Down(idx 13): {:?}", cube.get_sticker(13).color);
    println!("Left(idx 22): {:?}", cube.get_sticker(22).color);
    println!("Front(idx 31): {:?}", cube.get_sticker(31).color);
    println!("Right(idx 40): {:?}", cube.get_sticker(40).color);
    println!("Back(idx 49): {:?}", cube.get_sticker(49).color);
}
