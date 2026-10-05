// docs/review-evidence/c95d097/probe.rs
// c95d097 の挙動検証用プローブ
use cube_studio::coord::RawCube;
use cube_studio::cube::{apply, facelets, parse_moves};

fn main() {
    println!("=== Probe: c95d097 Verification ===");

    // 1. R04 検証: Kociemba 同時最適化時の phase_infos
    let moves = parse_moves("U R2 F B R B2 R U2 L B2 R U' D' R2 F R' L B2 U2 F2").unwrap();
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

    // Kociemba でセンター向きを考慮して解く（同時最適化が成功する）
    let sol = cube_studio::solve_state_with_algorithm(
        &state,
        10000,
        true,
        Some(initial_centers),
        "kociemba",
    )
    .unwrap();

    println!("\n[R04] Kociemba センター同時最適化時の phase_infos:");
    println!("  解法手数: {} 手", sol.moves.len());
    println!("  phase_infos 個数: {}", sol.phases.len());
    for (i, p) in sol.phases.iter().enumerate() {
        println!("    Phase {}: {} ({}..{})", i, p.name, p.start, p.end);
    }
    if sol.phases.is_empty() {
        println!("  => 【バグ確認】phase_infos が空 (len=0) です！フロントエンドはセンター解決フェーズを一切認識できません。");
    }

    // 2. R05 検証: ネイティブでのエラー時の戻り値
    println!("\n[R05] ネイティブでのエラーハンドリング:");
    let invalid_res = cube_studio::apply_moves(&state, "INVALID_MOVE");
    match invalid_res {
        Ok(_) => println!("  予期せぬ成功"),
        Err(js_val) => {
            println!("  apply_moves(\"INVALID_MOVE\") -> Err(JsValue)");
            let panic_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                format!("{:?}", js_val)
            }));
            match panic_res {
                Ok(s) => println!("  format!(\"{{:?}}\") 成功: {}", s),
                Err(_) => println!("  => 【重大バグ確認】ネイティブで Err(JsValue) を format!(\"{{:?}}\") または println! すると panic! します！"),
            }
        }
    }

    // 3. R08 検証: validate 関数のセマンティクス
    println!("\n[R08] validate 関数のセマンティクス:");
    // スクランブル状態（合法だが未完成）
    let scramble_res = cube_studio::validate(&state);
    println!("  合法なスクランブル状態の validate() 戻り値: {:?}", scramble_res);
    match scramble_res {
        Ok(is_solved) => {
            println!("  戻り値 bool = {} (完成しているかどうかが返る)", is_solved);
            if !is_solved {
                println!("  => 【バグ確認】合法なキューブであるにもかかわらず false が返ります。");
            }
        }
        Err(e) => println!("  エラー: {:?}", e),
    }
    // 完成状態
    let solved_res = cube_studio::validate(cube_studio::cube::SOLVED);
    println!("  完成状態の validate() 戻り値: {:?}", solved_res);
}
