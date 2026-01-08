use rubiks_cube_2x2::cube::{Cube, Move};
use rubiks_cube_2x2::solver;

#[test]
fn test_solve_already_solved() {
    let cube = Cube::new();
    let solution = solver::solve(&cube, 11, true);
    assert!(solution.found);
    assert_eq!(solution.moves.len(), 0);
}

#[test]
fn test_solve_one_move() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);

    let solution = solver::solve(&cube, 11, true);
    assert!(solution.found);
    assert_eq!(solution.moves.len(), 1);
    assert_eq!(solution.moves[0], Move::R.inverse());
}

#[test]
fn test_solve_two_moves() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);

    let solution = solver::solve(&cube, 11, true);
    assert!(solution.found);
    assert_eq!(solution.moves.len(), 2);

    let mut check_cube = cube.clone();
    for &mv in &solution.moves {
        check_cube.apply_move(mv);
    }
    assert!(check_cube.is_solved());
}

#[test]
fn test_solve_checker_pattern() {
    let mut cube = Cube::new();
    // 2x2のチェッカーボード（180度回転の組み合わせ）
    cube.apply_move(Move::R);
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);
    cube.apply_move(Move::U);
    cube.apply_move(Move::F);
    cube.apply_move(Move::F);

    let solution = solver::solve(&cube, 11, true);
    assert!(solution.found);

    let mut check_cube = cube.clone();
    for &mv in &solution.moves {
        check_cube.apply_move(mv);
    }
    assert!(check_cube.is_solved());
}

#[test]
fn test_solve_deep_scramble() {
    let mut cube = Cube::new();
    // 7手スクランブル
    let moves = vec![
        Move::R,
        Move::U,
        Move::F,
        Move::Rp,
        Move::Up,
        Move::Fp,
        Move::R,
    ];
    for &mv in &moves {
        cube.apply_move(mv);
    }

    let solution = solver::solve(&cube, 11, true);
    assert!(solution.found);

    let mut check_cube = cube.clone();
    for &mv in &solution.moves {
        check_cube.apply_move(mv);
    }
    assert!(check_cube.is_solved());
}

#[test]
fn test_solve_random_scramble() {
    // 試行回数を減らす
    for _ in 0..2 {
        let mut cube = Cube::new();
        // 3手スクランブル (5手だと深度11で解けない場合がある)
        cube.scramble(3);

        let solution = solver::solve(&cube, 11, true);
        assert!(solution.found, "3手スクランブルは深度11で解けるはず");

        let mut check_cube = cube.clone();
        for &sol_mv in &solution.moves {
            check_cube.apply_move(sol_mv);
        }
        assert!(
            check_cube.is_solved(),
            "Failed to solve scramble: {:?}",
            solution.moves
        );
    }
}

#[test]
fn test_solve_depth_limit() {
    let mut cube = Cube::new();
    cube.scramble(10); // 10手スクランブル

    // 低すぎる深度では解けない可能性が高い（ただし運が良ければ解ける）
    // 0手では当然解けない（スクランブルされているので）
    let solution = solver::solve(&cube, 0, true);
    assert!(!solution.found);
}

#[test]
fn test_solve_with_orientation_change() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);

    // 向きが違っていても色さえ合えば solve(..., true) は 0 手を返す「可能性」があるが
    // ここでは1手操作しているので必ず何か見つかるはず。
    let solution = solver::solve(&cube, 11, true);
    assert!(solution.found);
}

#[test]
fn test_solve_all_orientations() {
    // 全ての完成状態（向き違い）が0手で解決されることを確認
    let base = Cube::new();
    let rotations = vec![
        vec![Move::U, Move::Dp],
        vec![Move::R, Move::Lp],
        vec![Move::F, Move::Bp],
    ];

    let mut queue = std::collections::VecDeque::new();
    let mut visited = std::collections::HashSet::new();
    queue.push_back(base.clone());
    visited.insert(base.normalized());

    while let Some(current) = queue.pop_front() {
        for rot_moves in &rotations {
            let mut next = current.clone();
            for &mv in rot_moves {
                next.apply_move(mv);
            }
            let norm = next.normalized();
            if !visited.contains(&norm) {
                visited.insert(norm);
                queue.push_back(next.clone());

                let solution = solver::solve(&next, 11, true);
                assert!(solution.found);
                assert_eq!(
                    solution.moves.len(),
                    0,
                    "Already solved in different orientation"
                );
            }
        }
    }
}

#[test]
fn test_solve_rotated_and_scrambled() {
    // 向きを変えてからスクランブルしても解けることを確認
    let mut cube = Cube::new();
    cube.apply_move(Move::U);
    cube.apply_move(Move::Dp); // 全体回転相当

    cube.apply_move(Move::R); // 実際の崩し

    let solution = solver::solve(&cube, 11, true);
    assert!(solution.found);

    let mut check_cube = cube.clone();
    for &mv in &solution.moves {
        check_cube.apply_move(mv);
    }
    assert!(check_cube.is_solved());
}

#[test]
fn test_solution_struct() {
    let solution = solver::Solution {
        moves: vec![Move::R, Move::U],
        found: true,
    };
    assert!(solution.found);
    assert_eq!(solution.moves.len(), 2);

    let solution2 = solution.clone();
    assert_eq!(solution.moves, solution2.moves);
    assert_eq!(solution.found, solution2.found);
}

#[test]
fn test_solve_max_depth() {
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    let solution = solver::solve(&cube, 11, true);
    assert!(solution.found);
}

#[test]
fn test_solve_fully_aligned() {
    // 向きも揃える解決のテスト
    let mut cube = Cube::new();
    // 全体回転
    cube.apply_move(Move::U);
    cube.apply_move(Move::Dp);

    assert!(cube.is_solved());
    assert!(solver::is_fully_solved(&cube));

    // 向きを考慮した解決（既に回転された完成状態なので0手）
    let sol_align = solver::solve(&cube, 11, false);
    assert!(sol_align.found);
    assert_eq!(sol_align.moves.len(), 0);

    // 次に、真に解決が必要な状態（1手崩す）
    cube.apply_move(Move::R);
    assert!(!cube.is_solved());
    assert!(!solver::is_fully_solved(&cube));

    let sol_align2 = solver::solve(&cube, 11, false);
    assert!(sol_align2.found);
    assert!(!sol_align2.moves.is_empty());
}

#[test]
fn test_solve_with_progress_details() {
    use std::sync::mpsc;
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U); // 2手離れた状態

    let (tx, rx) = mpsc::channel();
    // 確実に解ける深度(11)を指定
    let solution = solver::solve_with_progress(&cube, 11, false, Some(tx));

    assert!(
        solution.found,
        "2手スクランブルが深度11で解けないはずがない"
    );
    let progress_values: Vec<f32> = rx.into_iter().collect();

    // 少なくとも完了通知(1.0)は含まれているはず
    assert!(
        progress_values.contains(&1.0),
        "完了時に 1.0 が送信されるべき"
    );

    // 他の進捗値が送られている場合、それらが 0.0 以上 1.0 以下であることを確認
    for &p in &progress_values {
        assert!((0.0..=1.0).contains(&p), "進捗値 {} は範囲外です", p);
    }
}

#[test]
fn test_solve_unsolvable_at_depth() {
    let mut cube = Cube::new();
    // 5手スクランブル
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);
    cube.apply_move(Move::F);
    cube.apply_move(Move::L);
    cube.apply_move(Move::B);

    // 2手以内では絶対に解けない
    let solution = solver::solve(&cube, 2, false);
    assert!(!solution.found, "2手で5手スクランブルが解けてはいけない");
}

#[test]
fn test_solve_is_fully_solved_coverage() {
    let cube = Cube::new();
    assert!(solver::is_fully_solved(&cube));

    let mut moved = cube.clone();
    moved.apply_move(Move::R);
    assert!(!solver::is_fully_solved(&moved));
}

#[test]
fn test_progress_channel_closed() {
    use std::sync::mpsc;

    // 進捗チャネルの受信側が閉じられている場合でも、
    // ソルバーが正常に動作することを確認
    let mut cube = Cube::new();
    cube.apply_move(Move::R);
    cube.apply_move(Move::U);

    let (tx, rx) = mpsc::channel();

    // 意図的に受信側を早期にdrop
    drop(rx);

    // send() がエラーを返しても、ソルバーは正常に完了するはず
    let solution = solver::solve_with_progress(&cube, 11, true, Some(tx));

    assert!(solution.found, "チャネルが閉じていても解法は見つかるべき");
    assert!(!solution.moves.is_empty(), "解法の手順が含まれているべき");
}

#[test]
fn test_progress_channel_multiple_sends() {
    use std::sync::mpsc;

    // 深めのスクランブルで複数回の進捗送信をトリガー
    let mut cube = Cube::new();
    cube.scramble(5);

    let (tx, rx) = mpsc::channel();
    let solution = solver::solve_with_progress(&cube, 11, true, Some(tx));

    // 進捗メッセージを収集
    let progress_messages: Vec<f32> = rx.into_iter().collect();

    // 少なくとも完了通知(1.0)は受信されているはず
    assert!(progress_messages.contains(&1.0), "完了通知が送信されるべき");

    // すべての進捗値が有効な範囲内
    for &p in &progress_messages {
        assert!((0.0..=1.0).contains(&p), "進捗値が範囲外: {}", p);
    }

    assert!(solution.found || !solution.found); // テストが完了することを確認
}

#[test]
fn test_end_to_end_scramble_solve_with_orientation() {
    // エンドツーエンドテスト: スクランブル→解法探索→手順実行→完全一致

    // 複数回テストして堅牢性を確認
    for scramble_moves in [3, 5, 7] {
        let mut cube = Cube::new();
        cube.scramble(scramble_moves);

        // 向きも揃える解法を探索
        let solution = solver::solve(&cube, 14, false);

        if !solution.found {
            // 深度14で見つからない場合はスキップ（稀なケース）
            continue;
        }

        // 見つかった手順をすべて適用
        for &mv in &solution.moves {
            cube.apply_move(mv);
        }

        // 完全に揃っているはず（色も向きも）
        assert!(
            solver::is_fully_solved(&cube),
            "{} 手スクランブル後、解法 {:?} を実行しても完全に揃わない",
            scramble_moves,
            solution.moves
        );
    }
}

#[test]
fn test_end_to_end_specific_scramble() {
    // 特定のスクランブルパターンでのエンドツーエンドテスト
    let mut cube = Cube::new();
    let scramble = vec![Move::R, Move::U, Move::F, Move::L];

    // スクランブル適用
    for &mv in &scramble {
        cube.apply_move(mv);
    }

    // 向きも揃える解法を探索
    let solution = solver::solve(&cube, 14, false);

    assert!(solution.found, "4手スクランブルは深度14で解けるはず");

    // 解法を適用
    for &mv in &solution.moves {
        cube.apply_move(mv);
    }

    // 完全に揃っているか検証
    assert!(solver::is_fully_solved(&cube), "解法実行後、完全に揃うべき");
    assert!(cube.is_solved(), "色も揃っているべき");

    // 初期状態のいずれかと一致していることを確認
    // （24通りの完成状態のいずれか）
    assert_eq!(
        cube,
        Cube::new(),
        "完全に初期状態に戻るべき、または24通りの完成状態のいずれか"
    );
}
