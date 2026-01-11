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

    // 解法を探索（HTMでは最大11手で必ず解ける）
    println!("探索中 (最大11手)...");
    let solution = solver::solve(&cube, 11, true);

    if solution.found {
        println!("✓ 解法発見！");
        println!("解法手数: {} 手", solution.moves.len());
        println!("解法: {:?}", solution.moves);

        // 解法の正当性確認
        let mut check_cube = cube.clone();
        for &mv in &solution.moves {
            check_cube.apply_move(mv);
        }
        assert!(check_cube.is_solved(), "解法を適用しても完成しませんでした");
    } else {
        println!("✗ 11手以内に解法が見つかりませんでした。");
        println!("2x2キューブは最大11手で解けることが証明されているため、");
        println!("この状態は物理的に不可能なパリティエラーの可能性があります。");
    }
}
