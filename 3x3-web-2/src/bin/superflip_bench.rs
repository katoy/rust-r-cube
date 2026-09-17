/// スーパーフリップ状態を、同時最適化 vs 逐次方式 で解いたときの手数を比較するベンチマーク
fn main() {
    // テストと同じシーケンス
    let superflip_seq = "R U' R U R U R U' R' U' R2 U R U' R' U' R2 U";

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

    // ── 2. 逐次方式（キューブ解法 → センター後付け補正）────────────────────
    {
        let budget_ms: u32 = 30_000;
        let start = std::time::Instant::now();
        let sol =
            cube_studio::solve_state_with_centers(&state, budget_ms, true, Some(initial_centers));
        let elapsed = start.elapsed();

        print!("[逐次方式 30s] ");
        match sol {
            Ok(res) => {
                println!(
                    "{}手 ({:.1}秒, {}ノード)",
                    res.moves.len(),
                    elapsed.as_secs_f64(),
                    res.nodes
                );
                println!("  手順: {}", res.moves.join(" "));
            }
            Err(e) => println!("エラー: {e}"),
        }
    }

    println!();

    // ── 3. 色だけモード（センター無視, 30秒）──────────────────────────────
    {
        let budget_ms: u32 = 30_000;
        let start = std::time::Instant::now();
        let sol = cube_studio::solve_state(&state, budget_ms, false);
        let elapsed = start.elapsed();

        print!("[色だけモード 30s] ");
        match sol {
            Ok(res) => {
                println!(
                    "{}手 ({:.1}秒, {}ノード)",
                    res.moves.len(),
                    elapsed.as_secs_f64(),
                    res.nodes
                );
                println!("  手順: {}", res.moves.join(" "));
            }
            Err(e) => println!("エラー: {e}"),
        }
    }
}
