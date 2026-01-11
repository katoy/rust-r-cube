use rubiks_cube_2x2::cube::Cube;
use rubiks_cube_2x2::solver;

#[test]
fn test_user_specified_state() {
    // ユーザー指定の状態:
    //      WWWW
    // OOOO GGGR RRBG BBRB
    //      YYYY

    let state = "     WWWW\nOOOO GGGR RRBG BBRB\n     YYYY";

    println!("=== ユーザー指定の状態 ===");
    println!("{}", state);
    println!();

    let cube = Cube::from_file_format(state).expect("状態の読み込みに失敗");

    // 状態の有効性をチェック
    match cube.is_valid_state() {
        Ok(_) => println!("✓ 有効な状態です"),
        Err(e) => {
            println!("✗ 無効な状態: {}", e);
            println!("この状態は物理的に不可能です");
            return;
        }
    }

    // まず11手で探索
    println!("探索中 (最大11手)...");
    let solution = solver::solve(&cube, 11, true);

    if solution.found {
        println!("✓ 11手以内で解法発見！");
        println!("解法手数: {} 手", solution.moves.len());
        println!("解法: {:?}", solution.moves);
        return;
    }

    // 解けない場合は14手まで拡張（QTMの限界値や保険として）
    println!("11手で見つかりませんでした。14手まで拡張して再探索中...");
    let solution = solver::solve(&cube, 14, true);

    if solution.found {
        println!("✓ 解法発見（11手を超過）！");
        println!("解法手数: {} 手", solution.moves.len());
        println!("解法: {:?}", solution.moves);
        println!("⚠️ 注意: 2x2の神の数は11手ですが、この状態は{}手必要でした。状態の記述かパリティを確認してください。", solution.moves.len());
    } else {
        println!("✗ 14手以内でも解法が見つかりませんでした。");
        println!(
            "この状態は有効と判定されましたが、到達不能（パリティエラー）の可能性があります。"
        );
    }
}
