use rubiks_cube_3x3::cube::{Cube, Move};
use rubiks_cube_3x3::solver::{solve, DEFAULT_MAX_DEPTH};
use std::fs;

fn main() {
    println!("=== cube_ex001.txt の向き復元テスト（修正後）===\n");

    // ファイルから読み込み
    let cube_text =
        fs::read_to_string("cubes/cube_ex001.txt").expect("cube_ex001.txt を読み込めませんでした");

    println!("ファイル内容:\n{}\n", cube_text);

    // キューブを作成
    let mut cube = Cube::from_file_format(&cube_text).expect("キューブのパースに失敗しました");

    println!("パース後のキューブ状態:\n{}\n", cube.to_file_format());

    // 状態チェック
    println!("--- 初期状態チェック ---");
    println!("is_solved(): {}", cube.is_solved());
    println!(
        "is_solved_with_orientation(): {}",
        cube.is_solved_with_orientation()
    );

    // 完成状態をスクランブル
    println!("\n--- スクランブル適用 ---");
    let scramble_moves = vec![Move::R, Move::U, Move::Rp, Move::Up, Move::F, Move::D];
    println!("スクランブル: {:?}", scramble_moves);

    for mv in &scramble_moves {
        cube.apply_move(*mv);
    }

    println!(
        "\nスクランブル後のキューブ状態:\n{}\n",
        cube.to_file_format()
    );
    println!("is_solved(): {}", cube.is_solved());
    println!(
        "is_solved_with_orientation(): {}",
        cube.is_solved_with_orientation()
    );

    // ファイルに保存
    let scrambled_text = cube.to_file_format();
    fs::write("cubes/cube_ex001_scrambled.txt", &scrambled_text)
        .expect("スクランブル後の状態を保存できませんでした");
    println!("スクランブル後の状態を cubes/cube_ex001_scrambled.txt に保存しました");

    // 1. 向きを揃えない解（色のみ）
    println!("\n--- 1. 向きを揃えない解（色のみ）---");
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

    // 2. 向きを揃える解（supercube）
    println!("\n--- 2. 向きを揃える解（向きも含む）---");
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

    println!("\n=== テスト完了 ===");
}
