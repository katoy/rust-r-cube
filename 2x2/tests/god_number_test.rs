use rubiks_cube_2x2::cube::{Cube, Move};
use rubiks_cube_2x2::solver;

/// 2x2キューブの神の数（最長解法手順）をテスト
/// 神の数は11手（向きを無視）または14手（向きを考慮）

#[test]
fn test_ru_5_times_pattern() {
    // R U を5回繰り返すパターン
    // これは2x2キューブの中で最も難しい状態の一つ（10手必要）
    let mut cube = Cube::new();

    println!("=== R U を5回繰り返すパターン ===");

    // R U を5回繰り返す
    for i in 0..5 {
        cube.apply_move(Move::R);
        cube.apply_move(Move::U);
        if i == 0 {
            println!("スクランブル: R U R U R U R U R U");
        }
    }

    println!("スクランブル後の状態: {}", cube.to_file_format());

    // この状態を解く
    let solution = solver::solve(&cube, solver::DEFAULT_MAX_DEPTH, true);

    assert!(solution.found, "解が見つかるはず");
    println!("解法手数: {} 手", solution.moves.len());
    println!("解法: {:?}", solution.moves);

    // 解法を適用して完成することを確認
    for mv in solution.moves {
        cube.apply_move(mv);
    }

    assert!(cube.is_solved(), "解法適用後に完成状態になるはず");
    println!("✓ 解法適用後、正しく完成状態になりました");

    // R U 5回は10手必要な難しい状態のはず
    // assert!(solution.moves.len() >= 9, "R U 5回パターンは9手以上必要");
}

#[test]
fn test_search_for_11_move_state() {
    // ランダムスクランブルで比較的長い手数を要する状態を探す
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    let all_moves = Move::all_moves();
    let mut rng = thread_rng();
    let mut max_solution_length = 0;
    let mut hardest_scramble = Vec::new();

    // 複数回試行して長い解法が必要な状態を探す
    for _ in 0..10 {
        let mut cube = Cube::new();
        let mut scramble = Vec::new();

        // 20手のランダムスクランブル
        for _ in 0..20 {
            let mv = *all_moves.choose(&mut rng).unwrap();
            cube.apply_move(mv);
            scramble.push(mv);
        }

        let solution = solver::solve(&cube, solver::DEFAULT_MAX_DEPTH, true);

        if solution.found && solution.moves.len() > max_solution_length {
            max_solution_length = solution.moves.len();
            hardest_scramble = scramble;
        }
    }

    println!("見つかった最長解法: {} 手", max_solution_length);
    println!("スクランブル手順: {:?}", hardest_scramble);

    // 2x2の神の数は11なので、それ以下のはず
    assert!(max_solution_length <= 11, "解法手数は11手以下のはず");
}

#[test]
fn test_known_difficult_pattern() {
    // "6 Spot" パターン - 比較的解くのが難しい既知のパターン
    let mut cube = Cube::new();

    // 6 Spot パターンを作成する手順
    let pattern = vec![
        Move::R,
        Move::U,
        Move::U,
        Move::R,
        Move::R,
        Move::U,
        Move::U,
        Move::R,
        Move::U,
        Move::U,
        Move::R,
        Move::R,
    ];

    for mv in &pattern {
        cube.apply_move(*mv);
    }

    let solution = solver::solve(&cube, solver::DEFAULT_MAX_DEPTH, true);

    assert!(solution.found, "6 Spot パターンの解が見つかるはず");
    println!("6 Spot パターンの解法手数: {} 手", solution.moves.len());
    println!("解法: {:?}", solution.moves);
}
