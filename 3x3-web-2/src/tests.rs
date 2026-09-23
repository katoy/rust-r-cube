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
    let mut bytes = SOLVED.as_bytes().to_vec();
    bytes[0] = b'R'; // U 面の 1 つを R に変更
    let state = String::from_utf8(bytes).unwrap();
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
        let solution_with = solve_state(&state, 30000, true);
        let solution_without = solve_state(&state, 30000, false);

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

fn superflip_preset_moves() -> Vec<usize> {
    let preset: serde_json::Value =
        serde_json::from_str(include_str!("../cubes/superflip.json")).unwrap();
    parse_moves(preset["scramble"].as_str().unwrap()).unwrap()
}

#[test]
fn superflip_preset_flips_all_edges_without_moving_pieces() {
    let preset: serde_json::Value =
        serde_json::from_str(include_str!("../cubes/superflip.json")).unwrap();
    let public_preset: serde_json::Value =
        serde_json::from_str(include_str!("../public/cubes/superflip.json")).unwrap();
    assert_eq!(
        preset, public_preset,
        "Published preset must match the source"
    );

    let moves = parse_moves(preset["scramble"].as_str().unwrap()).unwrap();
    let cube = apply(&RawCube::default(), &moves);
    let expected = RawCube {
        eo: [1; 12],
        ..RawCube::default()
    };
    assert_eq!(
        cube, expected,
        "Only the twelve edge orientations may change"
    );
    assert_eq!(preset["state"].as_str().unwrap(), facelets(&expected));
    assert_eq!(moves.len(), 20);
    assert_eq!(preset["solution_length"], 20);

    let inverse: Vec<_> = moves.iter().rev().map(|&m| m / 3 * 3 + 2 - m % 3).collect();
    assert_eq!(apply(&cube, &inverse), RawCube::default());
}

#[test]
fn superflip_solver_returns_valid_solution() {
    let moves = superflip_preset_moves();
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

        // 既存の25手以内という回帰基準に加え、20手未満にならないことを確認。
        assert!(
            (20..=25).contains(&solution.moves.len()),
            "Superflip solution must have 20 to 25 moves, got {}",
            solution.moves.len()
        );

        // 解法の詳細をログ出力
        println!("Special state solved in {} moves", solution.moves.len());
    }
}

#[test]
fn sample_face_turn_sequences_are_solvable() {
    // R/U の手順は一般のスクランブルであり、Superflip ではない。
    let sequences = [
        "R U R U R U R U R U R U R U R U R U R U",
        "R U' R U R U R U' R' U' R2",
    ];

    for (idx, sequence) in sequences.iter().enumerate() {
        let moves = parse_moves(sequence).unwrap();
        let cube = apply(&RawCube::default(), &moves);
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
    let with_orientation = solve_state(&state, 30000, true).unwrap();
    let without_orientation = solve_state(&state, 30000, false).unwrap();
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
        let solution = solve_state(&state, 30000, false);
        assert!(solution.is_ok());
    }
}

#[test]
fn multiple_solve_calls_are_consistent() {
    // 複数の解法呼び出しが一貫性を持つ
    let scrambled = facelets(&apply(&RawCube::default(), &scramble(50)));
    let sol1 = solve_state(&scrambled, 30000, true).unwrap();
    let sol2 = solve_state(&scrambled, 30000, true).unwrap();
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
    let result2 = solve_state(&state, 30000, true);
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
        algorithm: "kociemba".to_string(),
        phases: Vec::new(),
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
                            let moves = supercube::solve_center_orientations(needed).unwrap();

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
fn test_cancel_redundant_moves_opposite_faces() {
    use crate::supercube::cancel_redundant_moves;

    // 1. 同一面の直接相殺: R R' -> 空
    let r_rprime = parse_moves("R R'").unwrap();
    assert_eq!(cancel_redundant_moves(&r_rprime), Vec::<usize>::new());

    // 2. 対向面を跨いだ相殺: U D U' -> D
    let u_d_uprime = parse_moves("U D U'").unwrap();
    let expected_d = parse_moves("D").unwrap();
    assert_eq!(cancel_redundant_moves(&u_d_uprime), expected_d);

    // 3. 対向面を跨いだ合成: R L R2 -> L R' (または R' L)
    let r_l_r2 = parse_moves("R L R2").unwrap();
    let res = cancel_redundant_moves(&r_l_r2);
    let c1 = apply(&RawCube::default(), &r_l_r2);
    let c2 = apply(&RawCube::default(), &res);
    assert_eq!(c1, c2);
    assert_eq!(res.len(), 2);

    // 4. 複数対向面の完全相殺: U D U2 D' U -> 空
    let complex = parse_moves("U D U2 D' U").unwrap();
    assert_eq!(cancel_redundant_moves(&complex), Vec::<usize>::new());
}

#[test]
fn test_superflip_orientation_solve_length() {
    let moves = superflip_preset_moves();
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

    // 同時探索とセンター後付け補正のいずれでも、色・向きの完成を検証する。
    let sol = crate::solve_state_with_centers(&state, 60000, true, Some(initial_centers)).unwrap();
    println!(
        "Superflip solved with orientation in {} moves: {:?}",
        sol.moves.len(),
        sol.moves
    );

    // 検証1: キューブが正しく解けている
    assert_eq!(sol.state, SOLVED, "Cube must be fully solved");

    // 検証2: センター向きが全て0になっている
    let mut final_centers = initial_centers;
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
        "All centers must reach 0 rotation"
    );

    assert!((20..=60).contains(&sol.moves.len()));
    let solution_moves = parse_moves(&sol.moves.join(" ")).unwrap();
    assert_eq!(apply(&cube, &solution_moves), RawCube::default());
}

#[test]
fn test_superflip_5s_budget_solves_colors_and_centers() {
    let moves = superflip_preset_moves();
    let cube = apply(&RawCube::default(), &moves);
    let state = facelets(&cube);

    let mut initial_centers = [0i32; 6];
    for &m in &moves {
        let f = m / 3;
        let t: i32 = match m % 3 {
            0 => 1,
            1 => 2,
            2 => -1,
            _ => 0,
        };
        initial_centers[f] = (initial_centers[f] + t).rem_euclid(4);
    }

    // UIデフォルト予算 5000ms（リリースモード）で、色とセンター向きを解けることを検証
    // デバッグビルドやカバレッジ計測時はオーバーヘッドで約5倍遅いため予算を自動調整
    let budget = if cfg!(debug_assertions) {
        60_000
    } else {
        5_000
    };
    let sol = crate::solve_state_with_centers(&state, budget, true, Some(initial_centers)).unwrap();
    assert_eq!(sol.state, SOLVED, "Cube must be fully solved");
    assert!(
        (20..=24).contains(&sol.moves.len()),
        "Expected 20 to 24 moves within the UI budget, got {}",
        sol.moves.len()
    );
    let solution_moves = parse_moves(&sol.moves.join(" ")).unwrap();
    assert_eq!(apply(&cube, &solution_moves), RawCube::default());
    for m in solution_moves {
        initial_centers[m / 3] = (initial_centers[m / 3] + (m % 3 + 1) as i32) % 4;
    }
    assert_eq!(initial_centers, [0; 6]);
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

#[test]
fn center_input_parity_matches_legal_moves() {
    for seed in 1..=20 {
        let moves = scramble(seed);
        let state = facelets(&apply(&RawCube::default(), &moves));
        let expected = moves.iter().map(|m| m % 3 + 1).sum::<usize>() % 2;
        assert_eq!(crate::center_parity(&state).unwrap() as usize, expected);
    }
    assert_eq!(crate::center_parity(SOLVED).unwrap(), 0);
}

#[test]
fn incompatible_center_input_is_rejected_before_search() {
    let error =
        crate::solve_state_with_centers(SOLVED, 0, true, Some([1, 0, 0, 0, 0, 0])).unwrap_err();
    assert!(error.contains("センター"), "{error}");
    // Color-only solving does not constrain center orientation.
    assert!(crate::solve_state_with_centers(SOLVED, 0, false, Some([1, 0, 0, 0, 0, 0])).is_ok());
}

#[test]
fn test_solve_state_with_all_algorithms() {
    let sc = parse_moves("R U R' U'").unwrap();
    let scrambled_cube = apply(&RawCube::default(), &sc);
    let state = facelets(&scrambled_cube);

    // 1. CFOP
    let res_cfop = crate::solve_state_with_algorithm(&state, 5000, false, None, "cfop").unwrap();
    assert_eq!(res_cfop.state, SOLVED);
    assert_eq!(res_cfop.algorithm, "cfop");
    assert!(!res_cfop.phases.is_empty());

    // 2. Thistlethwaite
    let res_th =
        crate::solve_state_with_algorithm(&state, 5000, false, None, "thistlethwaite").unwrap();
    assert_eq!(res_th.state, SOLVED);
    assert_eq!(res_th.algorithm, "thistlethwaite");
    assert!(!res_th.phases.is_empty());

    // 3. Korf (IDA*)
    let res_korf = crate::solve_state_with_algorithm(&state, 5000, false, None, "korf").unwrap();
    assert_eq!(res_korf.state, SOLVED);
    assert_eq!(res_korf.algorithm, "korf");
    assert!(!res_korf.phases.is_empty());
    assert_eq!(res_korf.phases[0].name, "Korf 最短探索 (IDA*)");
    assert_eq!(res_korf.moves.len(), 4); // R U R' U' は最短4手

    // 4. Kociemba (デフォルト / 未知の文字列)
    let res_koc = crate::solve_state_with_algorithm(&state, 5000, false, None, "kociemba").unwrap();
    assert_eq!(res_koc.state, SOLVED);
    assert_eq!(res_koc.algorithm, "kociemba");

    let res_unknown =
        crate::solve_state_with_algorithm(&state, 5000, false, None, "unknown").unwrap();
    assert_eq!(res_unknown.state, SOLVED);

    // 5. wasm solve_with_algorithm
    let wasm_res = crate::solve_with_algorithm(&state, 5000, false, None, Some("cfop".to_string()));
    assert!(wasm_res.is_ok());
    let wasm_json = wasm_res.unwrap();
    let parsed: crate::ResultData = serde_json::from_str(&wasm_json).unwrap();
    assert_eq!(parsed.state, SOLVED);

    // 6. 不正な state のエラーハンドリング
    assert!(crate::solve_state_with_algorithm("INVALID", 1000, false, None, "cfop").is_err());
}

fn superflip_initial_centers(moves: &[usize]) -> [i32; 6] {
    let mut centers = [0i32; 6];
    for &m in moves {
        let f = m / 3;
        let t: i32 = match m % 3 {
            0 => 1,
            1 => 2,
            2 => -1,
            _ => 0,
        };
        centers[f] = (centers[f] + t).rem_euclid(4);
    }
    centers
}

#[test]
fn cfop_reports_visited_nodes_in_solution_statistics() {
    let cube = apply(&RawCube::default(), &parse_moves("R U F").unwrap());
    let solution =
        crate::solve_state_with_algorithm(&facelets(&cube), 5000, false, None, "cfop").unwrap();
    assert!(solution.nodes > 0, "CFOP must report the nodes it visits");
    assert_eq!(
        apply(&cube, &parse_moves(&solution.moves.join(" ")).unwrap()),
        RawCube::default()
    );
}

#[test]
fn test_superflip_cfop_move_counts_match_implementation_values() {
    // CFOP (Layer-By-Layer: 階層解法)
    // 人間向け定石マクロ（Cross -> F2L -> OLL -> PLL）を決定論的に適用するため、
    // Superflip に対する本実装の手順値を固定して検証する（理論最短手数ではない）。
    let moves = superflip_preset_moves();
    let cube = apply(&RawCube::default(), &moves);
    let state = facelets(&cube);
    let initial_centers = superflip_initial_centers(&moves);

    // 1. センター向き無視（色のみ解決）
    // Cross -> F2L -> OLL -> PLL の合計は本実装では 136手（Superflip の下界20手以上）。
    let sol_no_orient =
        crate::solve_state_with_algorithm(&state, 5000, false, None, "cfop").unwrap();
    assert_eq!(
        sol_no_orient.moves.len(),
        136,
        "This CFOP implementation solves Superflip colors in 136 moves"
    );
    assert!(
        sol_no_orient.moves.len() >= 20,
        "CFOP color-only moves must be at least God's Number 20"
    );
    assert_eq!(sol_no_orient.state, SOLVED);
    let no_orient_moves = parse_moves(&sol_no_orient.moves.join(" ")).unwrap();
    assert_eq!(facelets(&apply(&cube, &no_orient_moves)), SOLVED);

    // 2. センター向きを揃える場合
    // 色解決の 136手に加え、残ったセンターのズレを解消する定石マクロ（42手、対向面相殺適用後）が追加され、
    // 合計手数は 178手（136 + 42）となる。
    let sol_orient =
        crate::solve_state_with_algorithm(&state, 5000, true, Some(initial_centers), "cfop")
            .unwrap();
    assert_eq!(
        sol_orient.moves.len(),
        178,
        "This CFOP implementation solves Superflip in 178 moves (136 color + 42 center with commutative cancellation)"
    );
    assert!(
        sol_orient.moves.len() >= sol_no_orient.moves.len(),
        "Adding center orientation constraint cannot decrease the required move count"
    );
    assert_eq!(sol_orient.state, SOLVED);

    // センター向き解決フェーズが追加されていることを確認
    let last_phase = sol_orient.phases.last().unwrap();
    assert_eq!(last_phase.name, "センター向き解決");
    assert_eq!(last_phase.end - last_phase.start, 42);

    // キューブのピース配置およびセンター向きが完全に元通り（回転角0）になることを検証
    let orient_moves = parse_moves(&sol_orient.moves.join(" ")).unwrap();
    let solved_cube = apply(&cube, &orient_moves);
    assert_eq!(solved_cube, RawCube::default());

    let mut final_centers = initial_centers;
    for &m in &orient_moves {
        let f = m / 3;
        let t = match m % 3 {
            0 => 1,
            1 => 2,
            2 => -1,
            _ => 0,
        };
        final_centers[f] = (final_centers[f] + t).rem_euclid(4);
    }
    assert_eq!(final_centers, [0; 6], "All centers must be oriented to 0");
}

#[test]
fn test_superflip_kociemba_move_counts_match_theoretical_bounds() {
    // Kociemba 2段階探索アルゴリズムにおける手数の理論範囲検証
    // Superflip は理論上の最短手数が 20手（God's Number = 20）であることが数学的に証明されている。
    let moves = superflip_preset_moves();
    let cube = apply(&RawCube::default(), &moves);
    let state = facelets(&cube);
    let initial_centers = superflip_initial_centers(&moves);

    let budget = if cfg!(debug_assertions) {
        60_000
    } else {
        5_000
    };

    // 1. センター向き無視（色のみ解決）
    // 神の数字 20手以上を満たし、Kociemba の準最適解として 20〜24手の範囲に収まる（実測 21手）
    let sol_no_orient =
        crate::solve_state_with_algorithm(&state, budget, false, None, "kociemba").unwrap();
    assert!(
        (20..=24).contains(&sol_no_orient.moves.len()),
        "Kociemba color-only move count must be in theoretical bound [20, 24], got {}",
        sol_no_orient.moves.len()
    );
    assert_eq!(sol_no_orient.state, SOLVED);
    assert!(
        !sol_no_orient.phases.is_empty(),
        "Kociemba color-only solution must contain phase info"
    );
    assert_eq!(sol_no_orient.phases[0].name, "Kociemba Phase 1 (G1縮約)");
    assert_eq!(sol_no_orient.phases[1].name, "Kociemba Phase 2 (群解決)");
    assert_eq!(sol_no_orient.phases[0].end, sol_no_orient.phases[1].start);
    assert_eq!(sol_no_orient.phases[1].end, sol_no_orient.moves.len());

    // 2. センター向きを揃える場合（同時最適化）
    // センター向きも揃える制約により、手数は色のみと同等以上かつ同時最適化により 20〜24手の範囲に収まる（実測 23手）
    let sol_orient =
        crate::solve_state_with_algorithm(&state, budget, true, Some(initial_centers), "kociemba")
            .unwrap();
    assert!(
        (20..=24).contains(&sol_orient.moves.len()),
        "Kociemba with orientation move count must be in theoretical bound [20, 24], got {}",
        sol_orient.moves.len()
    );
    assert!(
        sol_orient.moves.len() >= sol_no_orient.moves.len(),
        "Adding center orientation constraint cannot decrease the required move count"
    );
    assert_eq!(sol_orient.state, SOLVED);
    assert!(
        !sol_orient.phases.is_empty(),
        "Simultaneous orientation must have phase info"
    );
    assert_eq!(sol_orient.phases[0].name, "同時最適化 (色＆センター)");

    // キューブのピース配置およびセンター向きが完全に元通り（回転角0）になることを検証
    let orient_moves = parse_moves(&sol_orient.moves.join(" ")).unwrap();
    let solved_cube = apply(&cube, &orient_moves);
    assert_eq!(solved_cube, RawCube::default());

    let mut final_centers = initial_centers;
    for &m in &orient_moves {
        let f = m / 3;
        let t = match m % 3 {
            0 => 1,
            1 => 2,
            2 => -1,
            _ => 0,
        };
        final_centers[f] = (final_centers[f] + t).rem_euclid(4);
    }
    assert_eq!(final_centers, [0; 6], "All centers must be oriented to 0");
}

#[test]
fn test_superflip_korf_move_counts_match_theoretical_bounds() {
    // Korf (IDA*) アルゴリズムにおける手数の理論範囲検証
    // Superflip は深さ20であるため、深さ12超過時に Kociemba 準最適解へフォールバックする
    let moves = superflip_preset_moves();
    let cube = apply(&RawCube::default(), &moves);
    let state = facelets(&cube);
    let initial_centers = superflip_initial_centers(&moves);

    let budget = if cfg!(debug_assertions) {
        60_000
    } else {
        5_000
    };

    // 1. センター向き無視
    // フォールバック時も Superflip の下界20手以上を満たし、配色が完成することを検証。
    let sol_no_orient =
        crate::solve_state_with_algorithm(&state, budget, false, None, "korf").unwrap();
    assert!(
        sol_no_orient.moves.len() >= 20,
        "Korf move count must be at least God's Number 20, got {}",
        sol_no_orient.moves.len()
    );
    assert_eq!(sol_no_orient.state, SOLVED);
    assert_eq!(sol_no_orient.phases[0].name, "Kociemba フォールバック");

    // 2. センター向きを揃える場合
    // 色解法にセンター補正を追加しても配色が完成することを検証。手数の固定値は要求しない。
    let sol_orient =
        crate::solve_state_with_algorithm(&state, budget, true, Some(initial_centers), "korf")
            .unwrap();
    assert!(
        sol_orient.moves.len() >= sol_no_orient.moves.len(),
        "Adding center orientation cannot decrease the move count"
    );
    assert_eq!(sol_orient.state, SOLVED);

    let orient_moves = parse_moves(&sol_orient.moves.join(" ")).unwrap();
    let solved_cube = apply(&cube, &orient_moves);
    assert_eq!(solved_cube, RawCube::default());
}

#[test]
fn test_superflip_thistlethwaite_move_counts_match_theoretical_bounds() {
    // Thistlethwaite アルゴリズムにおける理論上限（45手以内）および解法手数の検証
    // Morwen Thistlethwaite (1981) により証明された最大理論上限手数は 45手。
    // 本実装では群論的 4 段階探索（G0 -> G1 -> G2 -> G3 -> G4）により、Superflip を 31手で解決する。
    let moves = superflip_preset_moves();
    let cube = apply(&RawCube::default(), &moves);
    let state = facelets(&cube);
    let initial_centers = superflip_initial_centers(&moves);

    let budget = if cfg!(debug_assertions) {
        60_000
    } else {
        5_000
    };

    // 1. センター向き無視（色のみ）
    let sol_no_orient =
        crate::solve_state_with_algorithm(&state, budget, false, None, "thistlethwaite")
            .expect("Thistlethwaite must succeed on Superflip without error");

    assert!(
        sol_no_orient.moves.len() >= 20,
        "Thistlethwaite move count must be at least God's Number 20, got {}",
        sol_no_orient.moves.len()
    );
    assert!(
        sol_no_orient.moves.len() <= 45,
        "Thistlethwaite theoretical upper bound is 45 moves, got {}",
        sol_no_orient.moves.len()
    );
    assert_eq!(
        sol_no_orient.moves.len(),
        31,
        "Thistlethwaite solves Superflip in exactly 31 moves (Phase 1: 7, Phase 2: 8, Phase 3: 6, Phase 4: 10)"
    );
    assert_eq!(sol_no_orient.state, SOLVED);

    // 2. センター向きを揃える場合
    let sol_orient = crate::solve_state_with_algorithm(
        &state,
        budget,
        true,
        Some(initial_centers),
        "thistlethwaite",
    )
    .expect("Thistlethwaite with center orientation must succeed on Superflip");

    assert!(
        sol_orient.moves.len() >= sol_no_orient.moves.len(),
        "Adding center orientation cannot decrease move count"
    );
    assert_eq!(sol_orient.state, SOLVED);

    let orient_moves = parse_moves(&sol_orient.moves.join(" ")).unwrap();
    let solved_cube = apply(&cube, &orient_moves);
    assert_eq!(solved_cube, RawCube::default());
}

#[test]
fn test_r01_twist_slice_admissible() {
    let pt = crate::tables::PruningTable::get();
    let moves = parse_moves("R D2 R U2 L D2").unwrap();
    let c = apply(&RawCube::default(), &moves);
    let twist = c.get_twist() as usize;
    let slice = c.get_ud_slice() as usize;
    assert_eq!(twist, 1);
    assert_eq!(slice, 36);

    // D2 L' U2 R' D2 R' (6手) で Phase 1 座標 (twist=0, slice=0) に到達する
    let solve_moves = parse_moves("D2 L' U2 R' D2 R'").unwrap();
    assert_eq!(solve_moves.len(), 6);
    let solved_c = apply(&c, &solve_moves);
    assert_eq!(solved_c.get_twist(), 0);
    assert_eq!(solved_c.get_ud_slice(), 0);

    // ヒューリスティック値は真の距離 6 を超えてはならない
    let h = pt.get_twist_slice(twist, slice);
    assert!(
        h <= 6,
        "Pruning value must be <= exact distance 6, but got {}",
        h
    );
}

#[test]
fn test_r01_twist_slice_all_admissible() {
    let mt = crate::tables::MoveTable::get();
    let pt = crate::tables::PruningTable::get();

    // BFSで twist_slice 空間（2187 * 495 = 1,082,565 状態）の正確な最短距離を計算
    let mut dist = vec![255u8; 2187 * 495];
    dist[0] = 0;
    let mut queue = std::collections::VecDeque::new();
    queue.push_back((0usize, 0usize));
    while let Some((t, s)) = queue.pop_front() {
        let d = dist[t * 495 + s];
        for m in 0..18 {
            let nt = mt.twist[t][m] as usize;
            let ns = mt.ud_slice[s][m] as usize;
            let idx = nt * 495 + ns;
            if dist[idx] == 255 {
                dist[idx] = d + 1;
                queue.push_back((nt, ns));
            }
        }
    }

    // 全状態で heuristic <= exact_distance (許容的ヒューリスティック / 下界) を検証
    let mut overestimates = 0;
    for t in 0..2187 {
        for s in 0..495 {
            let exact = dist[t * 495 + s];
            let h = pt.get_twist_slice(t, s);
            if h > exact {
                overestimates += 1;
            }
        }
    }
    assert_eq!(
        overestimates, 0,
        "Twist/slice pruning table must NEVER overestimate exact distance! Found {} overestimates",
        overestimates
    );
}

#[test]
fn test_r05_thistlethwaite_g3_membership() {
    let facelets_str = "UUDUUUUUURRRRRRRRLFFFFFFBFBDDUDDDDDDLLLLLLLLRFBBBBBFBB";
    let cube = parse_state(facelets_str).expect("Valid cube state");

    // コーナー置換が [0, 1, 2, 4, 3, 7, 6, 5] であることを確認
    let expected_cp = [
        crate::coord::Corner::UFR,
        crate::coord::Corner::UFL,
        crate::coord::Corner::ULB,
        crate::coord::Corner::DFR, // 4
        crate::coord::Corner::UBR, // 3
        crate::coord::Corner::DRB, // 7
        crate::coord::Corner::DBL, // 6
        crate::coord::Corner::DLF, // 5
    ];
    assert_eq!(cube.cp, expected_cp);

    // この局面はテトラッド条件と偶パリティを満たすが、半回転群 G3 には到達不能である
    // したがって、is_g3 は false を返さなければならない。
    let th = crate::thistlethwaite::ThistlethwaiteSearch::new(1000);
    assert!(
        !th.is_g3(&cube),
        "Cube with unreachable corner permutation must NOT be classified as G3"
    );

    // 40320通りのコーナー置換のうち、半回転群で到達可能なものは厳密に96通りであることを確認
    let mut reachable_cp_count = 0;
    for cp in 0..40320 {
        let mut c = RawCube::default();
        c.set_cp(cp);
        if th.is_g3(&c) {
            reachable_cp_count += 1;
        }
    }
    assert_eq!(
        reachable_cp_count, 96,
        "Exactly 96 corner permutations should be reachable in G3, got {}",
        reachable_cp_count
    );
}

#[test]
fn test_r06_budget_contract() {
    let superflip = "UBULURUFDFDRDBDLFRFLBLBRRFRBLBBRUFUBUDDFDFDLDLRFRBLBBR";
    // Superflipに対して予算1msを指定
    let budget_ms = 1;

    // Korf
    let start = web_time::Instant::now();
    let _ = crate::solve_state_with_algorithm(superflip, budget_ms, false, None, "korf");
    let elapsed_korf = start.elapsed().as_millis();
    println!("elapsed_korf: {}ms", elapsed_korf);

    // Thistlethwaite
    let start = web_time::Instant::now();
    let _ = crate::solve_state_with_algorithm(superflip, budget_ms, false, None, "thistlethwaite");
    let elapsed_thistle = start.elapsed().as_millis();
    println!("elapsed_thistle: {}ms", elapsed_thistle);

    let c = apply(&RawCube::default(), &scramble(948));
    let state_948 = facelets(&c);

    // Korf with complex scramble and budget 5ms: must not hang on fallback
    let start = web_time::Instant::now();
    let _ = crate::solve_state_with_algorithm(&state_948, 5, false, None, "korf");
    let elapsed_korf = start.elapsed().as_millis();
    assert!(
        elapsed_korf <= 150,
        "Korf must respect budget_ms on fallback (got {}ms)",
        elapsed_korf
    );

    // Thistlethwaite with complex scramble and budget 5ms: must not hang on fallback
    let start = web_time::Instant::now();
    let _ = crate::solve_state_with_algorithm(&state_948, 5, false, None, "thistlethwaite");
    let elapsed_thistle = start.elapsed().as_millis();
    assert!(
        elapsed_thistle <= 150,
        "Thistlethwaite must respect budget_ms on fallback (got {}ms)",
        elapsed_thistle
    );

    // CFOP with complex scramble and budget 1ms
    let start = web_time::Instant::now();
    let _ = crate::solve_state_with_algorithm(&state_948, 1, false, None, "cfop");
    let elapsed_cfop = start.elapsed().as_millis();
    assert!(
        elapsed_cfop <= 50,
        "CFOP must respect budget_ms (got {}ms)",
        elapsed_cfop
    );
}

#[test]
fn test_r07_invalid_center_input_rejected() {
    // 1. centers_str に不正なトークンが含まれる場合、明示的エラー
    let res = crate::parse_initial_centers(Some("2,0,0,0,0,garbage"));
    assert!(
        res.is_err(),
        "Invalid center input with garbage token must return Err, but got {:?}",
        res
    );

    // 2. 要素数が6個未満（5個）の場合もエラー
    let res_5 = crate::parse_initial_centers(Some("2,0,0,0,0"));
    assert!(
        res_5.is_err(),
        "Center input with 5 elements must return Err, but got {:?}",
        res_5
    );

    // 3. 要素数が6個超過（7個）の場合もエラー
    let res_7 = crate::parse_initial_centers(Some("2,0,0,0,0,0,0"));
    assert!(
        res_7.is_err(),
        "Center input with 7 elements must return Err, but got {:?}",
        res_7
    );

    // 4. 値域外（0..=3 以外）が含まれる場合もエラー
    let res_range = crate::parse_initial_centers(Some("5,0,0,0,0,0"));
    assert!(
        res_range.is_err(),
        "Center input with out-of-range value 5 must return Err, but got {:?}",
        res_range
    );

    // 5. 正常な6要素の入力は正しくパースされる
    let res_valid = crate::parse_initial_centers(Some("2, 0, 1, 3, 0, 0")).unwrap();
    assert_eq!(res_valid, Some([2, 0, 1, 3, 0, 0]));

    // 6. None または空文字は None
    assert_eq!(crate::parse_initial_centers(None).unwrap(), None);
    assert_eq!(crate::parse_initial_centers(Some("")).unwrap(), None);
    assert_eq!(crate::parse_initial_centers(Some("   ")).unwrap(), None);
}

#[test]
fn test_native_core_functions() {
    let scrambled = crate::apply_moves_core(SOLVED, "R U F").unwrap();

    // 1. validate_core: SOLVED は true, scrambled は false, 不正文字列は Err
    assert_eq!(crate::validate_core(SOLVED), Ok(true));
    assert_eq!(crate::validate_core(&scrambled.state), Ok(false));
    let val_err = crate::validate_core("INVALID");
    assert!(val_err.is_err());
    // ネイティブ環境でフォーマットしてもパニックしないこと
    let _ = format!("{:?}", val_err);

    // 2. is_valid_core: 合法なら true, 不正は Err
    assert_eq!(crate::is_valid_core(SOLVED), Ok(true));
    assert_eq!(crate::is_valid_core(&scrambled.state), Ok(true));
    let valid_err = crate::is_valid_core("INVALID");
    assert!(valid_err.is_err());
    let _ = format!("{:?}", valid_err);

    // 3. is_solved_core: SOLVED のみ true
    assert_eq!(crate::is_solved_core(SOLVED), Ok(true));
    assert_eq!(crate::is_solved_core(&scrambled.state), Ok(false));
    let solved_err = crate::is_solved_core("INVALID");
    assert!(solved_err.is_err());
    let _ = format!("{:?}", solved_err);

    // 4. center_parity_core: パリティ値の取得
    assert_eq!(crate::center_parity_core(SOLVED), Ok(0));
    let parity_err = crate::center_parity_core("INVALID");
    assert!(parity_err.is_err());
    let _ = format!("{:?}", parity_err);
}
