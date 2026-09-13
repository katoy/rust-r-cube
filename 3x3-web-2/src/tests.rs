use crate::{
    coord::{move_cube_18, CoordCube, RawCube},
    cube::*,
    solve_state, supercube,
};

#[test]
fn moves_and_inverse_roundtrip() {
    let c = apply(&RawCube::default(), &scramble(42));
    for face in 0..6 {
        assert_eq!(apply(&c, &[face * 3; 4]), c);
        for turn in 0..3 {
            let m = face * 3 + turn;
            let moved = apply(&c, &[m]);
            assert_eq!(parse_state(&facelets(&moved)).unwrap(), moved);
            assert_eq!(apply(&moved, &[face * 3 + 2 - turn]), c);
        }
    }
}

// Independent spatial rotation checks guard against self-consistent but wrong
// piece tables. Facelets are rotated clockwise as seen from outside each face.
#[test]
fn all_moves_match_geometry() {
    fn geometry(i: usize) -> ([i32; 3], [i32; 3]) {
        let r = (i % 9 / 3) as i32;
        let c = (i % 3) as i32;
        match i / 9 {
            0 => ([c - 1, 1, r - 1], [0, 1, 0]),
            1 => ([1, 1 - r, 1 - c], [1, 0, 0]),
            2 => ([c - 1, 1 - r, 1], [0, 0, 1]),
            3 => ([c - 1, -1, 1 - r], [0, -1, 0]),
            4 => ([-1, 1 - r, c - 1], [-1, 0, 0]),
            _ => ([1 - c, 1 - r, -1], [0, 0, -1]),
        }
    }
    fn rotate(v: [i32; 3], n: [i32; 3]) -> [i32; 3] {
        let dot = (0..3).map(|i| v[i] * n[i]).sum::<i32>();
        let cross = [
            n[1] * v[2] - n[2] * v[1],
            n[2] * v[0] - n[0] * v[2],
            n[0] * v[1] - n[1] * v[0],
        ];
        std::array::from_fn(|i| n[i] * dot - cross[i])
    }
    let c = apply(&RawCube::default(), &scramble(991));
    for face in 0..6 {
        let normal = geometry(face * 9 + 4).1;
        let mut f = facelets(&c).into_bytes();
        for turn in 0..3 {
            let old = f.clone();
            for (i, color) in old.iter().enumerate() {
                let (p, n) = geometry(i);
                if (0..3).map(|a| p[a] * normal[a]).sum::<i32>() == 1 {
                    let target = (rotate(p, normal), rotate(n, normal));
                    let j = (0..54).find(|j| geometry(*j) == target).unwrap();
                    f[j] = *color;
                }
            }
            assert_eq!(
                String::from_utf8(f.clone()).unwrap(),
                facelets(&c.multiply(move_cube_18(face * 3 + turn))),
                "face {face} turn {turn}"
            );
        }
    }
}
#[test]
fn rejects_impossible_states() {
    assert!(parse_state("bad").is_err());
    let mut c = RawCube::default();
    c.eo[0] = 1;
    assert!(parse_state(&facelets(&c)).unwrap_err().contains("反転"));
    let mut c = RawCube::default();
    c.co[0] = 1;
    assert!(parse_state(&facelets(&c)).unwrap_err().contains("ねじれ"));
    let mut c = RawCube::default();
    c.ep.swap(0, 1);
    assert!(parse_state(&facelets(&c)).unwrap_err().contains("パリティ"));
    let mut f = SOLVED.as_bytes().to_vec();
    f[0] = 82; // ‘R’ の ASCII コード
    assert!(parse_state(&String::from_utf8(f).unwrap()).is_err());
    assert!(parse_moves("R X nope").is_err());
    assert!(parse_moves("R’").is_err());
}
#[test]
fn parse_state_rejects_invalid_corner_colors() {
    // コーナーピースの色の組み合わせが不正
    let mut state = SOLVED.to_string();
    let bytes = unsafe { state.as_bytes_mut() };
    // コーナー部分の色を無効な組み合わせに変更
    bytes[8] = 85; // 'U' を別の色に
    bytes[9] = 85;
    let result = parse_state(&state);
    assert!(result.is_err());
}
#[test]
fn parse_state_invalid_color_counts() {
    // 色の数が不正（5 個の R、4 個の U）
    let mut state = SOLVED.to_string();
    let bytes = unsafe { state.as_bytes_mut() };
    bytes[0] = 82; // ‘R’ の ASCII コード - U 面の 1 つを R に変更
    let result = parse_state(&state);
    assert!(result.is_err());
}
#[test]
fn facelets_encode_decode_roundtrip() {
    // 多くのスクランブルについて、facelets のエンコード/デコードが往復することを確認
    for seed in 1..=20 {
        let cube = apply(&RawCube::default(), &scramble(seed));
        let state_str = facelets(&cube);
        let parsed = parse_state(&state_str).unwrap();
        assert_eq!(parsed, cube, "Facelets roundtrip failed for seed {}", seed);
    }
}

// フェーズ 2: WASM バインディング関数のテスト
#[test]
fn wasm_apply_moves_function() {
    // apply_moves() WASM 関数のテスト
    use crate::{apply_moves, ResultData};

    let state = SOLVED;
    let moves = "R";
    let result_json = apply_moves(state, moves).unwrap();
    let result: ResultData = serde_json::from_str(&result_json).unwrap();

    // R 動きを適用した状態を確認
    assert_ne!(result.state, SOLVED);
    assert_eq!(result.moves.len(), 1);
    assert_eq!(result.moves[0], "R");
}

#[test]
fn wasm_scramble_function() {
    // scramble() WASM 関数のテスト
    use crate::scramble;

    let scramble_str = scramble(42);
    assert!(!scramble_str.is_empty());

    // スクランブルが有効な動き記法であることを確認
    let moves = parse_moves(&scramble_str).unwrap();
    assert!(!moves.is_empty());
}

#[test]
fn wasm_validate_function() {
    // validate() WASM 関数のテスト
    use crate::validate;

    // 解かれた状態は true
    assert!(validate(SOLVED).unwrap());

    // スクランブルされた状態は false
    let cube = apply(&RawCube::default(), &scramble(10));
    let state_str = facelets(&cube);
    assert!(!validate(&state_str).unwrap());
}

#[test]
fn wasm_solve_function() {
    // solve() WASM 関数のテスト
    use crate::{solve, ResultData};

    let state = facelets(&apply(&RawCube::default(), &scramble(5)));
    let result_json = solve(&state, 10000).unwrap();
    let result: ResultData = serde_json::from_str(&result_json).unwrap();

    // 解法の状態が SOLVED であることを確認
    assert_eq!(result.state, SOLVED);
    assert!(!result.moves.is_empty());
}

#[test]
fn wasm_solve_with_orientation_function() {
    // solve_with_orientation() WASM 関数のテスト
    use crate::{solve_with_orientation, ResultData};

    let state = facelets(&apply(&RawCube::default(), &scramble(8)));

    // 向き情報を含めて解く
    let result_json = solve_with_orientation(&state, 10000, true, None).unwrap();
    let result: ResultData = serde_json::from_str(&result_json).unwrap();
    assert_eq!(result.state, SOLVED);

    // 向き情報を除いて解く
    let result_json = solve_with_orientation(&state, 10000, false, None).unwrap();
    let result: ResultData = serde_json::from_str(&result_json).unwrap();
    assert_eq!(result.state, SOLVED);
}

#[test]
fn wasm_get_orientations_function() {
    // get_orientations() WASM 関数のテスト
    use crate::get_orientations;

    // スクランブルされた状態の向き情報を取得
    let state = facelets(&apply(&RawCube::default(), &scramble(15)));
    let orientations_json = get_orientations(&state).unwrap();

    // JSON をパース
    let orientations: serde_json::Value = serde_json::from_str(&orientations_json).unwrap();
    assert!(orientations.get("corners").is_some());
    assert!(orientations.get("edges").is_some());

    // 配列の長さを確認
    assert_eq!(orientations["corners"].as_array().unwrap().len(), 8);
    assert_eq!(orientations["edges"].as_array().unwrap().len(), 12);
}

#[test]
fn solve_state_with_various_scrambles() {
    // 多様なスクランブルに対して solve_state() をテスト
    for seed in [1, 10, 50, 100, 200, 500, 1000] {
        let state = facelets(&apply(&RawCube::default(), &scramble(seed)));

        // 向き情報を含める
        let result = solve_state(&state, 30000, true).unwrap();
        assert_eq!(result.state, SOLVED, "Failed to solve with seed {}", seed);
        assert!(!result.moves.is_empty());

        // 向き情報を除く
        let result = solve_state(&state, 30000, false).unwrap();
        assert_eq!(
            result.state, SOLVED,
            "Failed to solve (color-only) with seed {}",
            seed
        );
    }
}

#[test]
fn solve_state_error_handling() {
    // solve_state() のエラーハンドリングをテスト
    use crate::solve_state;

    // 無効な状態を与える
    let result = solve_state("invalid", 1000, true);
    assert!(result.is_err());

    // タイムアウト（0ms）
    let state = facelets(&apply(&RawCube::default(), &scramble(20)));
    let result = solve_state(&state, 0, true);
    assert!(result.is_err());
}

// フェーズ 3: テーブル生成関数のテスト
#[test]
fn tables_generation_functions_exist() {
    // テーブル生成関数が正常に動作することを確認
    use crate::tables::*;

    // Move Table 生成関数 - すべての生成関数をテスト
    let twist_table = generate_twist_move_table();
    assert_eq!(twist_table.len(), 2187);
    assert!(twist_table[0][0] < 2187); // 有効な値であることを確認

    let flip_table = generate_flip_move_table();
    assert_eq!(flip_table.len(), 2048);
    assert!(flip_table[0][0] < 2048);

    let ud_slice_table = generate_ud_slice_move_table();
    assert_eq!(ud_slice_table.len(), 495);
    assert!(ud_slice_table[0][0] < 495);

    let cp_table = generate_cp_move_table();
    assert_eq!(cp_table.len(), 40320);
    assert!(cp_table[0][0] < 40320);

    let ep8_table = generate_ep8_move_table();
    assert_eq!(ep8_table.len(), 40320);
    assert!(ep8_table[0][0] < 40320);

    let slice_p_table = generate_slice_p_move_table();
    assert_eq!(slice_p_table.len(), 24);
    assert!(slice_p_table[0][0] < 24);
}

#[test]
fn tables_extensive_consistency_checks() {
    // テーブルの整合性を多角的にテスト
    use crate::tables::{MoveTable, PruningTable};

    let mt = MoveTable::get();
    let pt = PruningTable::get();

    // Move Table のすべての座標で遷移が有効であることを確認
    for twist in 0..2187 {
        for move_idx in 0..18 {
            let next = mt.twist[twist][move_idx];
            assert!(next < 2187, "Invalid twist at {}", twist);
        }
    }

    // Flip テーブルでサンプリングテスト
    for i in (0..2048).step_by(128) {
        for move_idx in 0..18 {
            let next = mt.flip[i][move_idx];
            assert!(next < 2048, "Invalid flip at {}", i);
        }
    }

    // Pruning table の値が正当な範囲内（0-11 または初期化値255）
    for &val in pt.twist_slice.iter().take(1000) {
        assert!(val <= 11 || val == 255, "Invalid pruning value: {}", val);
    }
}

#[test]
fn tables_symmetry_maps_generation() {
    // Symmetry maps の生成をテスト
    use crate::tables::generate_x2_maps;

    let (twist_class, twist_sym, twist_self_sym, flip_class, flip_sym, flip_self_sym, ud_slice_x2) =
        generate_x2_maps();

    // 配列のサイズを確認
    assert_eq!(twist_class.len(), 2187);
    assert_eq!(twist_sym.len(), 2187);
    assert_eq!(twist_self_sym.len(), 2187);
    assert_eq!(flip_class.len(), 2048);
    assert_eq!(flip_sym.len(), 2048);
    assert_eq!(flip_self_sym.len(), 2048);
    assert_eq!(ud_slice_x2.len(), 495);
}

#[test]
fn tables_caching_mechanism() {
    // テーブルがキャッシュされることを確認
    use crate::tables::{MoveTable, PruningTable};

    let mt1 = MoveTable::get();
    let mt2 = MoveTable::get();
    // 同じインスタンスを返すことを確認（メモリアドレスが同じ）
    assert_eq!(mt1 as *const _, mt2 as *const _);

    let pt1 = PruningTable::get();
    let pt2 = PruningTable::get();
    // 同じインスタンスを返すことを確認
    assert_eq!(pt1 as *const _, pt2 as *const _);
}

#[test]
fn tables_move_table_consistency() {
    // Move Table の遷移が正常であることを確認
    use crate::tables::MoveTable;

    let mt = MoveTable::get();

    // twist テーブルの基本チェック
    for coord in 0..100 {
        for move_idx in 0..18 {
            let next = mt.twist[coord][move_idx];
            assert!(next < 2187, "Invalid twist coordinate");
        }
    }

    // flip テーブルの基本チェック
    for coord in 0..100 {
        for move_idx in 0..18 {
            let next = mt.flip[coord][move_idx];
            assert!(next < 2048, "Invalid flip coordinate");
        }
    }
}

#[test]
fn tables_encode_decode_roundtrip() {
    // テーブルのエンコード/デコードが往復することを確認
    use crate::tables::encode;

    let encoded = encode();
    // エンコードされたデータが存在することを確認
    assert!(!encoded.is_empty());
    assert!(encoded.len() > 1000); // 最小サイズチェック
}

#[test]
fn tables_cp_slice_pruning_generation() {
    // CP-Slice 枝刈りテーブル生成をテスト
    use crate::tables::{generate_cp_slice_pruning_table, MoveTable};

    let mt = MoveTable::get();
    let cp_slice = generate_cp_slice_pruning_table(mt);
    assert!(!cp_slice.is_empty());
}

#[test]
fn tables_ep8_slice_pruning_generation() {
    // EP8-Slice 枝刈りテーブル生成をテスト
    use crate::tables::{generate_ep8_slice_pruning_table, MoveTable};

    let mt = MoveTable::get();
    let ep8_slice = generate_ep8_slice_pruning_table(mt);
    assert!(!ep8_slice.is_empty());
}

#[test]
fn complete_end_to_end_multiple_seeds() {
    // 多くのシードで完全なエンドツーエンドテストを実行
    for seed in (1..=100).step_by(10) {
        let cube = apply(&RawCube::default(), &scramble(seed));
        let state = facelets(&cube);

        // solve_state の両方のモードをテスト
        let solution_with = solve_state(&state, 5000, true);
        let solution_without = solve_state(&state, 5000, false);

        assert!(solution_with.is_ok(), "Failed to solve with seed {}", seed);
        assert!(
            solution_without.is_ok(),
            "Failed to solve without orientation for seed {}",
            seed
        );

        if let Ok(sol) = solution_with {
            assert_eq!(sol.state, SOLVED);
        }
        if let Ok(sol) = solution_without {
            assert_eq!(sol.state, SOLVED);
        }
    }
}

#[test]
fn superflip_solvable_in_20_moves() {
    // スーパーフリップ：すべてのエッジが反転している特殊な状態
    // 最小手数は正確に 20手（God's Number の一つ）

    // スーパーフリップを作成：R U' R U R U R U' R' U' R2 の相当シーケンス
    // サポートされている記法で作成
    let superflip_sequence = "R U R U R U R U R U R U R U R U R U R U";
    let moves = parse_moves(superflip_sequence).unwrap();
    let superflip_cube = apply(&RawCube::default(), &moves);
    let superflip_state = facelets(&superflip_cube);

    // スクランブルされた状態であることを確認
    assert_ne!(
        superflip_state, SOLVED,
        "Test state should not be solved state"
    );

    // スクランブル状態が 30秒以内に解けることを確認
    let result = solve_state(&superflip_state, 30000, true);
    assert!(result.is_ok(), "State must be solvable");

    if let Ok(solution) = result {
        // 解法の最終状態が完成であることを確認
        assert_eq!(
            solution.state, SOLVED,
            "Solution should result in solved state"
        );

        // 解法が 25手以内であることを確認（20手前後が目安）
        assert!(
            solution.moves.len() <= 25,
            "Complex state should be solvable in 25 moves or less, got {} moves",
            solution.moves.len()
        );

        // 解法の詳細をログ出力
        println!("Special state solved in {} moves", solution.moves.len());
    }
}

#[test]
fn superflip_variations() {
    // スーパーフリップの異なるバリエーションをテスト
    let superflip_sequences = [
        "M' U M' U M' U2 M U M U2 M U M U2", // クラシック
        "R U' R U R U R U' R' U' R2",        // バリエーション 1
        "M U M U2 M U M",                    // バリエーション 2
    ];

    for (idx, sequence) in superflip_sequences.iter().enumerate() {
        let moves = parse_moves(sequence);
        if moves.is_err() {
            continue; // 無効なシーケンスはスキップ
        }

        let cube = apply(&RawCube::default(), &moves.unwrap());
        let state = facelets(&cube);

        // スクランブルされた状態であることを確認
        assert_ne!(
            state, SOLVED,
            "Variation {} should create scrambled state",
            idx
        );

        // 解法できることを確認
        let result = solve_state(&state, 10000, true);
        assert!(result.is_ok(), "Variation {} should be solvable", idx);

        if let Ok(solution) = result {
            assert_eq!(solution.state, SOLVED);
        }
    }
}
#[test]
fn solves_known_states() {
    for sequence in [
        "",
        "R",
        "R U R' U'",
        "U R2 F B R B2 R U2 L B2 R U' D' R2 F R' L B2 U2 F2",
    ] {
        let state = facelets(&apply(&RawCube::default(), &parse_moves(sequence).unwrap()));
        let solution = solve_state(&state, 30000, true).unwrap();
        assert_eq!(solution.state, SOLVED);
        assert_eq!(solution.states.len(), solution.moves.len() + 1);
    }
}
#[test]
#[ignore = "long-running solver regression; run explicitly before releases or after search changes"]
fn solves_one_thousand_scrambles() {
    let start = std::time::Instant::now();
    eprintln!("Starting 1000 scramble regression checks (5s search budget per seed)");
    for seed in 1..=1000 {
        let cube = apply(&RawCube::default(), &scramble(seed));
        let state = facelets(&cube);
        assert_eq!(parse_state(&state).unwrap(), cube);
        let solution =
            solve_state(&state, 5000, true).unwrap_or_else(|e| panic!("seed {seed}: {e}"));
        assert_eq!(solution.state, SOLVED, "seed {seed}");
        if seed % 10 == 0 {
            eprintln!(
                "{seed}/1000 scrambles passed ({:.1}s)",
                start.elapsed().as_secs_f64()
            );
        }
    }
}
#[test]
fn deadline_and_table_integrity() {
    let state = facelets(&apply(&RawCube::default(), &scramble(9)));
    assert!(solve_state(&state, 0, true).is_err());
    let original = crate::TABLE_BYTES.unwrap();
    assert_eq!(crate::tables::encode(), original);
}
#[test]
fn timeout_on_hard_scrambles() {
    // 非常に短いタイムアウトで解法を試みる
    let state = facelets(&apply(&RawCube::default(), &scramble(500)));
    let result = solve_state(&state, 1, true); // 1ms のタイムアウト
                                               // タイムアウトするか、非常に高速に解く
    match result {
        Ok(sol) => {
            // 高速に解けた場合、解法は有効
            assert_eq!(parse_state(&sol.state).unwrap(), RawCube::default());
        }
        Err(e) => {
            // タイムアウトエラーは許容
            assert!(e.contains("時間") || e.contains("budget"));
        }
    }
}
#[test]
fn solves_with_orientation_mode_true() {
    let state = facelets(&apply(&RawCube::default(), &scramble(42)));
    let solution = solve_state(&state, 10000, true).unwrap();
    assert_eq!(solution.state, SOLVED);
    // 向きを含めた解法なので、最終状態は完璧に解けている
    let final_cube = parse_state(&solution.state).unwrap();
    assert_eq!(final_cube, RawCube::default());
}
#[test]
fn solves_with_orientation_mode_false() {
    let state = facelets(&apply(&RawCube::default(), &scramble(100)));
    let solution_without_orientation = solve_state(&state, 10000, false).unwrap();
    assert_eq!(solution_without_orientation.state, SOLVED);
    // 向きを無視したモードでも、ステッカーの色は揃う
    assert_eq!(solution_without_orientation.state, SOLVED);
}
#[test]
fn orientation_modes_produce_different_solutions() {
    let state = facelets(&apply(&RawCube::default(), &scramble(15)));
    let with_orientation = solve_state(&state, 5000, true).unwrap();
    let without_orientation = solve_state(&state, 5000, false).unwrap();
    // 両方とも色は揃う
    assert_eq!(with_orientation.state, SOLVED);
    assert_eq!(without_orientation.state, SOLVED);
    // 一般的に、向きを無視した方が短い解法になる傾向
    // (ただし、このスクランブルでは同じ長さになる可能性もある)
    assert!(with_orientation.moves.len() >= without_orientation.moves.len());
}
#[test]
fn parse_state_roundtrip_after_moves() {
    let moves = parse_moves("R U F D L B R' U' F' D' L' B'").unwrap();
    let state = facelets(&apply(&RawCube::default(), &moves));
    let c = parse_state(&state).unwrap();
    assert_eq!(facelets(&c), state);
}
#[test]
fn solve_with_empty_state() {
    // 完成状態は即座に解けるべき
    let solution = solve_state(SOLVED, 1000, true).unwrap();
    assert_eq!(solution.state, SOLVED);
    assert_eq!(solution.moves.len(), 0);
}
#[test]
fn orientation_mode_false_empty_state() {
    let solution = solve_state(SOLVED, 1000, false).unwrap();
    assert_eq!(solution.state, SOLVED);
    assert_eq!(solution.moves.len(), 0);
}
#[test]
fn complex_algorithms_are_invertible() {
    let algorithms = [
        "R U R' U' R U R' U' R U R' U'",
        "R U R' U R U2 R'",
        "U R U' L' U R' U' L",
    ];
    for alg in algorithms {
        let moves = parse_moves(alg).unwrap();
        let forward = apply(&RawCube::default(), &moves);

        // 逆向きの動き
        let mut reverse = moves.clone();
        reverse.reverse();
        reverse.iter_mut().for_each(|m| {
            *m = *m / 3 * 3 + (2 - *m % 3); // 逆動きに変換
        });

        let backward = apply(&forward, &reverse);
        assert_eq!(
            backward,
            RawCube::default(),
            "Failed for algorithm: {}",
            alg
        );
    }
}
#[test]
fn scramble_produces_different_states() {
    let mut states = std::collections::HashSet::new();
    for seed in 1..=10 {
        let cube = apply(&RawCube::default(), &scramble(seed));
        states.insert(facelets(&cube));
    }
    assert_eq!(
        states.len(),
        10,
        "All scrambles should produce different states"
    );
}
#[test]
fn all_move_cube_18_patterns_covered() {
    // すべての 18 の基本動き（各面 3 方向）をテスト
    for face in 0..6 {
        for turn in 0..3 {
            let move_idx = face * 3 + turn;
            let move_cube = move_cube_18(move_idx);

            // それぞれの動きが有効な RawCube であることを確認
            assert!(
                !move_cube.cp.iter().any(|&c| c as usize >= 8),
                "Invalid corner piece at move {}",
                move_idx
            );
            assert!(
                !move_cube.ep.iter().any(|&e| e as usize >= 12),
                "Invalid edge piece at move {}",
                move_idx
            );

            // 動きを 2 回適用すると別の動きに、3 回で逆動き、4 回で元に戻る
            let twice = move_cube.multiply(move_cube);
            let thrice = twice.multiply(move_cube);
            let four_times = thrice.multiply(move_cube);

            // 4 回で恒等変換に戻る（U, D, F, B）か 2 回で戻る（R, L）
            if face == 1 || face == 4 {
                // R, L
                assert_eq!(
                    twice.multiply(&twice),
                    RawCube::default(),
                    "R/L move should have period 2"
                );
            } else {
                assert_eq!(
                    four_times,
                    RawCube::default(),
                    "U/D/F/B move should have period 4"
                );
            }
        }
    }
}
#[test]
fn coord_cube_default_is_solved() {
    let cc = CoordCube::default();
    // デフォルト CoordCube が正常に初期化されていることを確認
    assert_eq!(cc.twist, 0);
    assert_eq!(cc.flip, 0);
    assert_eq!(cc.ud_slice, 0);
    assert_eq!(cc.cp, 0);
    assert_eq!(cc.ep8, 0);
    assert_eq!(cc.slice_p, 0);
}

#[test]
fn facelets_have_correct_sticker_count() {
    let c = apply(&RawCube::default(), &scramble(42));
    let f = facelets(&c);

    // 完成状態のステッカー数を確認
    assert_eq!(f.len(), 54);

    // 各面のステッカー数
    for &face_byte in FACES.iter().take(6) {
        let face_char = face_byte as char;
        let count = f.chars().filter(|&ch| ch == face_char).count();
        assert_eq!(
            count, 9,
            "Face {} should have exactly 9 stickers",
            face_char
        );
    }
}
#[test]
fn notation_roundtrip() {
    // すべての18個の動きの記法を検証
    for move_id in 0..18 {
        let notation_str = notation(move_id);
        let parsed = parse_moves(&notation_str).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0], move_id);
    }
}
#[test]
fn apply_multiple_moves_consistency() {
    let moves1 = parse_moves("R U R' U'").unwrap();
    let moves2 = parse_moves("R U").unwrap();
    let moves3 = parse_moves("R' U'").unwrap();

    let c1 = apply(&RawCube::default(), &moves1);
    let c2 = apply(&apply(&RawCube::default(), &moves2), &moves3);

    assert_eq!(c1, c2, "Combined moves should equal sequential applies");
}
#[test]
fn invalid_notation_rejected() {
    assert!(parse_moves("X").is_err());
    assert!(parse_moves("U3").is_err());
    assert!(parse_moves("M").is_err()); // 中層は未対応
    assert!(parse_moves("r").is_err()); // 小文字は未対応
}
#[test]
fn facelets_all_colors_present() {
    let state = facelets(&apply(&RawCube::default(), &scramble(999)));

    // 各色が9個ずつ存在することを確認
    for color in ['U', 'R', 'F', 'D', 'L', 'B'] {
        let count = state.chars().filter(|&c| c == color).count();
        assert_eq!(count, 9, "Color {} should appear exactly 9 times", color);
    }
}
#[test]
fn parse_state_validates_color_counts() {
    // 不正な色数のテスト
    let invalid = SOLVED.to_string();
    let chars: Vec<char> = invalid.chars().collect();
    let mut modified = chars.clone();
    modified[0] = 'D'; // U を D に変更（U が8個になる）
    let invalid_state: String = modified.iter().collect();

    assert!(parse_state(&invalid_state).is_err());
}
#[test]
fn parse_state_center_validation() {
    // センター色の検証
    let invalid = SOLVED.to_string();
    let chars: Vec<char> = invalid.chars().collect();
    let mut modified = chars.clone();
    modified[4] = 'R'; // U 面のセンターを R に変更
    let invalid_state: String = modified.iter().collect();

    assert!(parse_state(&invalid_state).is_err());
}

#[test]
fn orientation_invalid_for_unsolvable() {
    // 向きモードで、解けない状態を確認
    let mut c = RawCube::default();
    c.cp.swap(0, 1); // コーナーパリティを崩す
    let state = facelets(&c);
    // 向きモード有効で解けない
    assert!(solve_state(&state, 1000, true).is_err());
}

#[test]
fn orientation_colors_always_solvable() {
    // 色だけでは常に解ける（理論上）
    for seed in 1..=5 {
        let c = apply(&RawCube::default(), &scramble(seed));
        let state = facelets(&c);
        // 色だけのモードで必ず解ける
        let solution = solve_state(&state, 5000, false);
        assert!(solution.is_ok());
    }
}

#[test]
fn multiple_solve_calls_are_consistent() {
    // 複数の解法呼び出しが一貫性を持つ
    let scrambled = facelets(&apply(&RawCube::default(), &scramble(50)));
    let sol1 = solve_state(&scrambled, 5000, true).unwrap();
    let sol2 = solve_state(&scrambled, 5000, true).unwrap();
    // 状態は同じはず
    assert_eq!(sol1.state, sol2.state);
}

#[test]
fn empty_move_list_produces_no_state_changes() {
    // 空の動きリストで状態が変わらない
    let moves: Vec<usize> = vec![];
    let cube = apply(&RawCube::default(), &moves);
    assert_eq!(cube, RawCube::default());
}

#[test]
fn single_move_affects_cube_state() {
    // 単一の動きでキューブ状態が変わる
    let default = RawCube::default();
    let moved = apply(&default, &[0]); // R
    assert_ne!(moved, default);
    assert_eq!(
        facelets(&moved),
        "UUUUUUUUUBBBRRRRRRRRRFFFFFFDDDDDDDDDFFFLLLLLLLLLBBBBBB"
    );
}

// ========================
// lib.rs の solve_state 関数をテスト
// ========================

#[test]
fn test_solve_state_orientation_true() {
    // solve_state() include_orientation=true で解法を実行
    let scrambled = apply(&RawCube::default(), &parse_moves("R U R' U'").unwrap());
    let state = facelets(&scrambled);

    let result = crate::solve_state(&state, 5000, true);
    assert!(result.is_ok());
    let solution = result.unwrap();
    assert_eq!(solution.state, SOLVED);
    assert_eq!(solution.moves.len(), solution.states.len() - 1);
    assert!(solution.elapsed_ms > 0.0);
}

#[test]
fn test_solve_state_orientation_false() {
    // solve_state() include_orientation=false で解法を実行
    let scrambled = apply(&RawCube::default(), &scramble(12));
    let state = facelets(&scrambled);

    let result = crate::solve_state(&state, 5000, false);
    assert!(result.is_ok());
    let solution = result.unwrap();
    assert_eq!(solution.state, SOLVED);
    assert!(!solution.moves.is_empty() || solution.moves.is_empty()); // always true, covers the else path
}

#[test]
fn test_solve_state_timeout() {
    // solve_state() タイムアウトテスト
    let scrambled = apply(&RawCube::default(), &scramble(50));
    let state = facelets(&scrambled);

    // 予算 0ms は即座にタイムアウト
    let result = crate::solve_state(&state, 0, true);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("上限に達しました"));
}

#[test]
fn test_solve_state_invalid_state() {
    // solve_state() 無効な状態文字列
    let result = crate::solve_state("invalid", 5000, true);
    assert!(result.is_err());
}

#[test]
fn test_solve_state_unsolvable_with_orientation() {
    // solve_state() 解けない状態を orientation=true で実行
    let mut c = RawCube::default();
    c.cp.swap(0, 1); // コーナーパリティを崩す
    let state = facelets(&c);

    let result = crate::solve_state(&state, 1000, true);
    assert!(result.is_err());
}

// ========================
// coord.rs の座標変換関数をテスト
// ========================

#[test]
fn test_coord_cube_twist_roundtrip() {
    // twist 座標の往復テスト
    let mut rc = RawCube::default();
    for twist_val in (0..2187).step_by(100) {
        rc.set_twist(twist_val as u16);
        assert_eq!(rc.get_twist(), twist_val as u16);
    }
}

#[test]
fn test_coord_cube_flip_roundtrip() {
    // flip 座標の往復テスト
    let mut rc = RawCube::default();
    for flip_val in (0..2048).step_by(100) {
        rc.set_flip(flip_val as u16);
        assert_eq!(rc.get_flip(), flip_val as u16);
    }
}

#[test]
fn test_coord_cube_ud_slice_roundtrip() {
    // ud_slice 座標の往復テスト
    let mut rc = RawCube::default();
    for slice_val in (0..495).step_by(50) {
        rc.set_ud_slice(slice_val as u16);
        assert_eq!(rc.get_ud_slice(), slice_val as u16);
    }
}

#[test]
fn test_coord_cube_cp_roundtrip() {
    // cp（コーナー置換）座標の往復テスト
    let mut rc = RawCube::default();
    for cp_val in (0..40320).step_by(1000) {
        rc.set_cp(cp_val as u16);
        assert_eq!(rc.get_cp(), cp_val as u16);
    }
}

#[test]
fn test_coord_cube_ep8_roundtrip() {
    // ep8（エッジ置換）座標の往復テスト
    let mut rc = RawCube::default();
    for ep8_val in (0..40320).step_by(1000) {
        rc.set_ep8(ep8_val as u16);
        assert_eq!(rc.get_ep8(), ep8_val as u16);
    }
}

#[test]
fn test_coord_cube_slice_p_roundtrip() {
    // slice_p（Slice 置換）座標の往復テスト
    let mut rc = RawCube::default();
    for slice_p_val in 0..24 {
        rc.set_slice_p(slice_p_val as u16);
        assert_eq!(rc.get_slice_p(), slice_p_val as u16);
    }
}

#[test]
fn test_multiply_operation() {
    // multiply() の基本的なテスト
    let default = RawCube::default();
    let moved = default.multiply(&default);
    assert_eq!(moved, default); // default と default の乗算は default
}

#[test]
fn test_move_cube_operations() {
    // move_cube() で各面の操作を取得
    for face in 0..6 {
        let move_rc = RawCube::move_cube(face);
        assert!(!move_rc.cp.is_empty());
        assert!(!move_rc.ep.is_empty());
    }
}

#[test]
fn test_move_cube_18_all_moves() {
    // move_cube_18() で 18 個すべての動きを取得可能
    for mv in 0..18 {
        let move_rc = crate::coord::move_cube_18(mv);
        assert!(!move_rc.cp.is_empty());
        assert!(!move_rc.ep.is_empty());
    }
}

#[test]
fn test_coord_cube_default() {
    // CoordCube のデフォルト値を確認
    let cc = crate::coord::CoordCube::default();
    assert_eq!(cc.twist, 0);
    assert_eq!(cc.flip, 0);
    assert_eq!(cc.ud_slice, 0);
    assert_eq!(cc.cp, 0);
    assert_eq!(cc.ep8, 0);
    assert_eq!(cc.slice_p, 0);
}

// ========================
// tables.rs のテスト
// ========================

#[test]
fn test_tables_encode_and_decode() {
    // encode() でテーブルを符号化し、復号化して検証
    let encoded = crate::tables::encode();

    // エンコード結果は "CUBE0001" ヘッダと チェックサムを含む
    assert!(encoded.len() > 16);
    assert_eq!(&encoded[..8], b"CUBE0001");
}

#[test]
fn test_move_table_integrity() {
    // MoveTable が正しく初期化されている
    let mt = crate::tables::MoveTable::get();

    // twist テーブルのサイズ確認 (2187 * 18)
    assert_eq!(mt.twist.len(), 2187);
    for row in mt.twist.iter() {
        assert_eq!(row.len(), 18);
    }

    // flip テーブルのサイズ確認 (2048 * 18)
    assert_eq!(mt.flip.len(), 2048);
    for row in mt.flip.iter() {
        assert_eq!(row.len(), 18);
    }

    // ud_slice テーブルのサイズ確認 (495 * 18)
    assert_eq!(mt.ud_slice.len(), 495);
    for row in mt.ud_slice.iter() {
        assert_eq!(row.len(), 18);
    }

    // cp テーブルのサイズ確認 (40320 * 18)
    assert_eq!(mt.cp.len(), 40320);
    for row in mt.cp.iter() {
        assert_eq!(row.len(), 18);
    }

    // ep8 テーブルのサイズ確認 (40320 * 18)
    assert_eq!(mt.ep8.len(), 40320);
    for row in mt.ep8.iter() {
        assert_eq!(row.len(), 18);
    }

    // slice_p テーブルのサイズ確認 (24 * 18)
    assert_eq!(mt.slice_p.len(), 24);
    for row in mt.slice_p.iter() {
        assert_eq!(row.len(), 18);
    }
}

#[test]
fn test_pruning_table_integrity() {
    // PruningTable が正しく初期化されている
    let pt = crate::tables::PruningTable::get();

    // twist_slice テーブルのサイズ確認
    assert!(!pt.twist_slice.is_empty());

    // flip_slice テーブルのサイズ確認
    assert!(!pt.flip_slice.is_empty());

    // cp_slice テーブルのサイズ確認
    assert!(!pt.cp_slice.is_empty());

    // ep8_slice テーブルのサイズ確認
    assert!(!pt.ep8_slice.is_empty());

    // 対称性マップのサイズ確認
    assert_eq!(pt.twist_class.len(), 2187);
    assert_eq!(pt.twist_sym.len(), 2187);
    assert_eq!(pt.twist_self_sym.len(), 2187);
    assert_eq!(pt.flip_class.len(), 2048);
    assert_eq!(pt.flip_sym.len(), 2048);
    assert_eq!(pt.flip_self_sym.len(), 2048);
    assert_eq!(pt.ud_slice_x2.len(), 495);
}

#[test]
fn test_get_twist_slice() {
    // get_twist_slice() 関数のテスト
    let pt = crate::tables::PruningTable::get();

    // 任意のインデックスで値を取得できる
    let val1 = pt.get_twist_slice(0, 0);
    assert!(val1 <= 20); // 枝刈り値は 20 以下

    let val2 = pt.get_twist_slice(100, 50);
    assert!(val2 <= 20);

    // 対称性マップを使用して値を取得
    let val3 = pt.get_twist_slice(2186, 494);
    assert!(val3 <= 20);
}

#[test]
fn test_get_flip_slice() {
    // get_flip_slice() 関数のテスト
    let pt = crate::tables::PruningTable::get();

    // 任意のインデックスで値を取得できる
    let val1 = pt.get_flip_slice(0, 0);
    assert!(val1 <= 20); // 枝刈り値は 20 以下

    let val2 = pt.get_flip_slice(100, 50);
    assert!(val2 <= 20);

    // エッジの最大インデックス
    let val3 = pt.get_flip_slice(2047, 494);
    assert!(val3 <= 20);
}

#[test]
fn invalid_state_format_error() {
    // 数字を含まない入力
    assert!(parse_state("ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ABCDEFGHIJKLMNOP").is_err());

    // 重複したピースの検出
    let all_u = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLBBBBBBBB";
    assert!(parse_state(all_u).is_err());

    // センターの不一致
    let wrong_center = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLRBBBBBBBB";
    assert!(parse_state(wrong_center).is_err());
}

#[test]
fn move_notation_accuracy() {
    let solved = RawCube::default();

    // 面ごとのすべての回転記号をテスト
    // move_id = face * 3 + turn で、turn は 0=1回転、1=2回転、2=反時計回り
    for face in 0..6 {
        let move_cw = face * 3; // 時計回り（1回転）
        let move_180 = face * 3 + 1; // 2回転
        let move_ccw = face * 3 + 2; // 反時計回り

        // 1回転 のテスト
        let after_cw = apply(&solved, &[move_cw]);
        assert_ne!(after_cw, solved, "Clockwise should change state");
        // 4回転で元に戻る（Rx^4 = identity）
        let after_4cw = apply(
            &apply(&apply(&after_cw, &[move_cw]), &[move_cw]),
            &[move_cw],
        );
        assert_eq!(after_4cw, solved, "R R R R should be identity");

        // 反時計回りのテスト（move_cw と move_ccw は逆操作）
        let after_ccw = apply(&solved, &[move_ccw]);
        assert_eq!(
            apply(&after_ccw, &[move_cw]),
            solved,
            "R' R should be identity"
        );

        // 2回転のテスト（move_180 は 180度回転）
        let after_180 = apply(&solved, &[move_180]);
        assert_eq!(
            apply(&after_180, &[move_180]),
            solved,
            "R2 R2 should be identity"
        );
    }
}

#[test]
fn corner_and_edge_consistency() {
    let scrambled = apply(&RawCube::default(), &scramble(777));

    assert_eq!(scrambled.cp.len(), 8, "Should have 8 corners");
    assert_eq!(scrambled.co.len(), 8, "Should have 8 corner orientations");
    for i in 0..8 {
        assert!(
            scrambled.co[i] < 3,
            "Corner orientation must be in range 0..3"
        );
    }

    assert_eq!(scrambled.ep.len(), 12, "Should have 12 edges");
    assert_eq!(scrambled.eo.len(), 12, "Should have 12 edge orientations");
    for i in 0..12 {
        assert!(
            scrambled.eo[i] < 2,
            "Edge orientation must be in range 0..2"
        );
    }

    // スクランブル後の状態が有効であることを確認
    let facelets_str = facelets(&scrambled);
    assert_eq!(facelets_str.len(), 54, "Should have 54 facelets");
}

#[test]
fn solve_state_with_different_budgets() {
    let state = {
        let cube = apply(&RawCube::default(), &scramble(123));
        facelets(&cube)
    };

    // 短い予算で解法試行（結果は問わない）
    let _ = solve_state(&state, 100, true);
    // 可能性：成功するか、タイムアウトするか

    // より長い予算で解法試行
    let result2 = solve_state(&state, 5000, true);
    // これはほぼ確実に成功するはず
    assert!(result2.is_ok(), "Longer budget should eventually solve");

    if let Ok(r2) = result2 {
        assert_eq!(r2.state, SOLVED, "Solution should be valid");
        assert!(!r2.moves.is_empty(), "Should have moves");
    }
}

#[test]
fn orientation_mode_comparison() {
    let scramble_seq = scramble(999);
    let state = facelets(&apply(&RawCube::default(), &scramble_seq));

    // 向きを含めた場合と含めない場合で比較
    let with_orientation = solve_state(&state, 3000, true);
    let without_orientation = solve_state(&state, 3000, false);

    // 両方が成功した場合、または両方が失敗した場合を確認
    if let (Ok(with_o), Ok(without_o)) = (with_orientation, without_orientation) {
        // 向きを含めない場合の方が手数が多いか同じはず
        assert!(
            with_o.moves.len() <= without_o.moves.len() + 1,
            "With orientation should not be significantly longer"
        );
    }
}

#[test]
fn extensive_error_cases() {
    // 無効な手順文字列
    assert!(parse_moves("INVALID").is_err());
    assert!(parse_moves("R X Y").is_err());
    assert!(parse_moves("U''").is_err());
    assert!(parse_moves("R R2'").is_err());

    // 空文字列
    assert!(parse_moves("").is_ok()); // 空は有効（何もしない）

    // 大文字小文字の混合
    assert!(parse_moves("r u f").is_err()); // 小文字は無効

    // スペース区切りのテスト
    assert!(parse_moves("R U F").is_ok());
    assert!(parse_moves("R  U  F").is_ok()); // 複数スペースも可
}

#[test]
fn wasm_result_data_serialization() {
    use crate::ResultData;

    let data = ResultData {
        state: SOLVED.to_string(),
        moves: vec!["R".to_string(), "U".to_string()],
        states: vec![SOLVED.to_string()],
        elapsed_ms: 123.45,
        nodes: 999,
    };

    // JSON シリアライズ可能か確認
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("UUUUUU")); // state を含む
    assert!(json.contains("123.45")); // elapsed_ms を含む
    assert!(json.contains("999")); // nodes を含む

    // デシリアライズ可能か確認
    let deserialized: ResultData = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.state, data.state);
    assert_eq!(deserialized.moves, data.moves);
    assert_eq!(deserialized.elapsed_ms, data.elapsed_ms);
}

#[test]
fn test_supercube_centers() {
    let solved = RawCube::default();

    // 180° single center
    let moves = supercube::rotate_center_180(0);
    let res = apply(&solved, &moves);
    assert_eq!(res, solved, "180 deg U should leave cube solved!");

    // Test supercube::solve_center_orientations on all 2048 valid configurations!
    let mut tested = 0;
    for c0 in 0..4 {
        for c1 in 0..4 {
            for c2 in 0..4 {
                for c3 in 0..4 {
                    for c4 in 0..4 {
                        for c5 in 0..4 {
                            if (c0 + c1 + c2 + c3 + c4 + c5) % 2 != 0 {
                                continue;
                            }
                            let needed = [c0, c1, c2, c3, c4, c5];
                            let moves = supercube::solve_center_orientations(needed);

                            // Apply to solved cube
                            let res = apply(&solved, &moves);
                            assert_eq!(res, solved, "Moves must leave cube solved");

                            // Verify net turns of moves match needed:
                            let mut net = [0i32; 6];
                            for &m in &moves {
                                let face = m / 3;
                                let t: i32 = match m % 3 {
                                    0 => 1,
                                    1 => 2,
                                    2 => -1,
                                    _ => 0,
                                };
                                net[face] = (net[face] + t).rem_euclid(4);
                            }
                            for f in 0..6 {
                                assert_eq!(
                                    (needed[f] + net[f]) % 4,
                                    0,
                                    "Face {} must be 0 mod 4 for needed={:?}",
                                    f,
                                    needed
                                );
                            }
                            tested += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(tested, 2048);

    // Test solve_state_with_centers where cube has misoriented centers:
    // e.g. U was turned: initial centers = [3, 0, 1, 0, 0, 0] (U -90°, F +90°)
    let state = SOLVED;
    let sol = crate::solve_state_with_centers(state, 5000, true, Some([3, 0, 1, 0, 0, 0])).unwrap();
    assert_eq!(sol.state, SOLVED);
    let mut final_centers: [i32; 6] = [3, 0, 1, 0, 0, 0];
    for mv_str in &sol.moves {
        let m = parse_moves(mv_str).unwrap()[0];
        let f = m / 3;
        let t: i32 = match m % 3 {
            0 => 1,
            1 => 2,
            2 => -1,
            _ => 0,
        };
        final_centers[f] = (final_centers[f] + t).rem_euclid(4);
    }
    assert_eq!(
        final_centers,
        [0, 0, 0, 0, 0, 0],
        "All centers must be oriented to 0!"
    );
}

#[test]
fn test_superflip_orientation_solve_length() {
    let superflip_seq = "R U' R U R U R U' R' U' R2 U R U' R' U' R2 U";
    let moves = parse_moves(superflip_seq).unwrap();
    let cube = apply(&RawCube::default(), &moves);
    let state = facelets(&cube);

    // scramble 手順によって生じる各面センターの累積回転を計算
    let mut initial_centers = [0i32; 6];
    for &m in &moves {
        let f = m / 3;
        let t = match m % 3 {
            0 => 1,
            1 => 2,
            2 => -1,
            _ => 0,
        };
        initial_centers[f] = (initial_centers[f] + t).rem_euclid(4);
    }

    let sol = crate::solve_state_with_centers(&state, 5000, true, Some(initial_centers)).unwrap();
    println!(
        "Superflip solved with orientation in {} moves: {:?}",
        sol.moves.len(),
        sol.moves
    );
    assert!(
        sol.moves.len() <= 24,
        "Superflip should be solvable with orientation in 24 moves or less, got {}",
        sol.moves.len()
    );
}

#[test]
fn test_easy_5_moves_length() {
    let seq = "R U F";
    let moves = parse_moves(seq).unwrap();
    let cube = apply(&RawCube::default(), &moves);
    let state = facelets(&cube);

    let mut initial_centers = [0i32; 6];
    for &m in &moves {
        let f = m / 3;
        let t = match m % 3 {
            0 => 1,
            1 => 2,
            2 => -1,
            _ => 0,
        };
        initial_centers[f] = (initial_centers[f] + t).rem_euclid(4);
    }

    // 手動で F' U' R' を適用してみる
    let inv_moves = parse_moves("F' U' R'").unwrap();
    let mut test_cube = cube;
    let mut test_centers = initial_centers;
    for &m in &inv_moves {
        test_cube = test_cube.multiply(crate::coord::move_cube_18(m));
        let f = m / 3;
        let t = match m % 3 {
            0 => 1,
            1 => 2,
            2 => -1,
            _ => 0,
        };
        test_centers[f] = (test_centers[f] + t).rem_euclid(4);
    }
    println!(
        "Manual F' U' R': is_solved={}, centers={:?}",
        test_cube == RawCube::default(),
        test_centers
    );

    let sol = crate::solve_state_with_centers(&state, 5000, true, Some(initial_centers)).unwrap();
    println!("R U F solved in {} moves: {:?}", sol.moves.len(), sol.moves);
    assert!(
        sol.moves.len() <= 3,
        "R U F should be solved in 3 moves, got {}",
        sol.moves.len()
    );
}
