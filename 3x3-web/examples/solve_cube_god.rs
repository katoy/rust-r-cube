use rubiks_cube_3x3::cube::Cube;
use rubiks_cube_3x3::solver::{solve, DEFAULT_MAX_DEPTH};
use std::fs;

fn main() {
    let test_files = vec!["cubes/cube_normal.txt", "cubes/cube_ex001.txt"];

    for file_path in test_files {
        println!("\n{}", "=".repeat(60));
        println!("Testing: {}", file_path);
        println!("{}\n", "=".repeat(60));

        // キューブファイルを読み込む
        let cube_text = match fs::read_to_string(file_path) {
            Ok(text) => text,
            Err(e) => {
                println!("✗ ファイルの読み込みに失敗: {}", e);
                continue;
            }
        };

        // テキストをパースしてCubeを作成
        let cube = match Cube::from_file_format(&cube_text) {
            Ok(c) => c,
            Err(e) => {
                println!("✗ キューブのパースに失敗: {}", e);
                continue;
            }
        };

        println!("キューブの状態:\n{}\n", cube.to_file_format());

        // 1. 向きを揃えない解（色のみ）
        println!("--- 1. 向きを揃えない解（色のみ）---");
        let solution_color_only = solve(&cube, DEFAULT_MAX_DEPTH, true);

        if solution_color_only.found {
            println!("✓ 解が見つかりました！");
            println!("  手数: {}", solution_color_only.moves.len());

            // 検証
            let mut test_cube = cube.clone();
            for mv in &solution_color_only.moves {
                test_cube.apply_move(*mv);
            }
            if test_cube.is_solved() {
                println!("  ✓ 検証成功: 色が揃っています");
            }
            if test_cube.is_solved_with_orientation() {
                println!("  ✓ 向きも揃っています");
            } else {
                println!("  ℹ 色のみ揃っています（向きは揃っていません）");
            }
        } else {
            println!("✗ 解が見つかりませんでした");
            if !solution_color_only.message.is_empty() {
                println!("  メッセージ: {}", solution_color_only.message);
            }
        }

        println!();

        // 2. 向きを揃える解（supercube）
        println!("--- 2. 向きを揃える解（向きも含む）---");
        let solution_with_orientation = solve(&cube, DEFAULT_MAX_DEPTH, false);

        if solution_with_orientation.found {
            println!("✓ 解が見つかりました！");
            println!("  手数: {}", solution_with_orientation.moves.len());

            // 検証
            let mut test_cube = cube.clone();
            for mv in &solution_with_orientation.moves {
                test_cube.apply_move(*mv);
            }
            if test_cube.is_solved() {
                println!("  ✓ 検証成功: 色が揃っています");
            }
            if test_cube.is_solved_with_orientation() {
                println!("  ✓ 向きも揃っています");
            } else {
                println!("  ⚠ 注意: 向きが揃っていません");
            }
        } else {
            println!("✗ 解が見つかりませんでした");
            if !solution_with_orientation.message.is_empty() {
                println!("  メッセージ: {}", solution_with_orientation.message);
            }
        }
    }

    println!("\n{}", "=".repeat(60));
    println!("テスト完了");
    println!("{}", "=".repeat(60));
}
