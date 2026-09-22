/// スーパーフリップ状態を、同時最適化 vs 逐次方式 で解いたときの手数を比較するベンチマーク
fn main() {
    // UI と同じプリセットを使用する
    let preset: serde_json::Value =
        serde_json::from_str(include_str!("../../cubes/superflip.json")).unwrap();
    let superflip_seq = preset["scramble"].as_str().unwrap();

    let moves: Vec<usize> = cube_studio::cube::parse_moves(superflip_seq).unwrap();
    let cube = cube_studio::cube::apply(&cube_studio::coord::RawCube::default(), &moves);
    let state = cube_studio::cube::facelets(&cube);

    // スクランブル手順によるセンター累積回転を計算
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
    println!("スクランブルシーケンス: {superflip_seq}");
    println!("initial_centers (センター累積回転): {initial_centers:?}");
    println!();

    // ── 1. 同時最適化（Search::with_target_centers, 30秒）──────────────────
    {
        let budget_ms: u32 = 30_000;
        let start = std::time::Instant::now();
        let mut search =
            cube_studio::search::Search::new(budget_ms).with_target_centers(initial_centers);
        let result = search.solve(&cube);
        let elapsed = start.elapsed();

        print!("[同時最適化 30s] ");
        match result {
            Some(sol) => {
                let notations: Vec<_> = sol
                    .iter()
                    .map(|&m| cube_studio::cube::notation(m))
                    .collect();
                println!(
                    "{}手 ({:.1}秒, {}ノード)",
                    sol.len(),
                    elapsed.as_secs_f64(),
                    search.nodes
                );
                println!("  手順: {}", notations.join(" "));

                // 検証: センターが全て 0 になるか
                let mut centers = initial_centers;
                for &m in &sol {
                    let f = m / 3;
                    let t: i32 = match m % 3 {
                        0 => 1,
                        1 => 2,
                        2 => -1,
                        _ => 0,
                    };
                    centers[f] = (centers[f] + t).rem_euclid(4);
                }
                println!(
                    "  最終センター: {centers:?} (全て0 = {})",
                    centers.iter().all(|&c| c == 0)
                );

                let final_cube = cube_studio::cube::apply(&cube, &sol);
                println!(
                    "  キューブ完成: {}",
                    final_cube == cube_studio::coord::RawCube::default()
                );
            }
            None => println!(
                "タイムアウト ({:.1}秒, {}ノード)",
                elapsed.as_secs_f64(),
                search.nodes
            ),
        }
    }

    println!();

    // ── 2. 逐次方式（色のみ解法 → センター後付け補正）──────────────────────
    {
        let budget_ms: u32 = 30_000;
        let start = std::time::Instant::now();
        // センター制約なしで色解法を求める
        let color_sol =
            cube_studio::solve_state_with_algorithm(&state, budget_ms, false, None, "kociemba");
        let elapsed = start.elapsed();

        print!("[逐次方式 30s] ");
        match color_sol {
            Ok(color_res) => {
                let parsed_moves =
                    cube_studio::cube::parse_moves(&color_res.moves.join(" ")).unwrap();
                let mut current_centers = initial_centers;
                for &m in &parsed_moves {
                    let f = m / 3;
                    let t = match m % 3 {
                        0 => 1,
                        1 => 2,
                        2 => -1,
                        _ => 0,
                    };
                    current_centers[f] = (current_centers[f] + t).rem_euclid(4);
                }
                let center_fixes =
                    cube_studio::supercube::solve_center_orientations(current_centers)
                        .unwrap_or_default();

                let mut total_moves = color_res.moves.clone();
                for &m in &center_fixes {
                    total_moves.push(cube_studio::cube::notation(m).to_string());
                }

                // センター検証
                let mut final_centers = current_centers;
                for &m in &center_fixes {
                    let f = m / 3;
                    let t = match m % 3 {
                        0 => 1,
                        1 => 2,
                        2 => -1,
                        _ => 0,
                    };
                    final_centers[f] = (final_centers[f] + t).rem_euclid(4);
                }
                let mut all_moves = parsed_moves;
                all_moves.extend(center_fixes);
                let final_cube = cube_studio::cube::apply(&cube, &all_moves);

                println!(
                    "合計{}手 (色{}手 + センター補正{}手) ({:.1}秒, {}ノード)",
                    total_moves.len(),
                    color_res.moves.len(),
                    total_moves.len() - color_res.moves.len(),
                    elapsed.as_secs_f64(),
                    color_res.nodes
                );
                println!("  手順: {}", total_moves.join(" "));
                println!(
                    "  最終センター: {final_centers:?} (全て0 = {})",
                    final_centers.iter().all(|&c| c == 0)
                );
                println!(
                    "  キューブ完成: {}",
                    final_cube == cube_studio::coord::RawCube::default()
                );
            }
            Err(e) => println!("エラー: {e}"),
        }
    }

    println!();

    // ── 3. 各アルゴリズムでの検証（センター無視 vs センター向き揃える）──────
    let algorithms = ["kociemba", "cfop", "thistlethwaite", "korf"];
    for alg in algorithms {
        println!("=== Algorithm: {alg} ===");
        // センター向き無視
        let start = std::time::Instant::now();
        let sol_no_orient =
            cube_studio::solve_state_with_algorithm(&state, 10_000, false, None, alg);
        let elapsed = start.elapsed();
        match sol_no_orient {
            Ok(res) => println!(
                "  [センター無視] {}手 ({:.2}ms, {}ノード)",
                res.moves.len(),
                elapsed.as_secs_f64() * 1000.0,
                res.nodes
            ),
            Err(e) => println!("  [センター無視] エラー: {e}"),
        }

        // センター向き揃える
        let start = std::time::Instant::now();
        let sol_orient = cube_studio::solve_state_with_algorithm(
            &state,
            10_000,
            true,
            Some(initial_centers),
            alg,
        );
        let elapsed = start.elapsed();
        match sol_orient {
            Ok(res) => println!(
                "  [センター揃える] {}手 ({:.2}ms, {}ノード)",
                res.moves.len(),
                elapsed.as_secs_f64() * 1000.0,
                res.nodes
            ),
            Err(e) => println!("  [センター揃える] エラー: {e}"),
        }
    }
}
