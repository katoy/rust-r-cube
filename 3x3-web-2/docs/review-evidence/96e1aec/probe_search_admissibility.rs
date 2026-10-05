// 実証プローブ 1: Kociemba Phase 2 におけるセンター向きヒューリスティックのアドミッシビリティ証明
// 
// Phase 2 で許容される手: U, U2, U', R2, F2, D, D2, D', L2, B2 (10手)
// 各手 m が与えるセンター回転:
// - U, U', U2: 面 0 のみ回転 (+1, -1, +2 mod 4)
// - D, D', D2: 面 3 のみ回転 (+1, -1, +2 mod 4)
// - R2: 面 1 のみ回転 (+2 mod 4)
// - F2: 面 2 のみ回転 (+2 mod 4)
// - L2: 面 4 のみ回転 (+2 mod 4)
// - B2: 面 5 のみ回転 (+2 mod 4)

fn min_phase2_center_moves(centers: [i32; 6]) -> u8 {
    let mut count = 0u8;
    if centers[0] != 0 { count += 1; }
    if centers[1] != 0 { count += 1; }
    if centers[2] != 0 { count += 1; }
    if centers[3] != 0 { count += 1; }
    if centers[4] != 0 { count += 1; }
    if centers[5] != 0 { count += 1; }
    count
}

fn main() {
    println!("=== Probe 1: Kociemba Phase 2 Center Heuristic Admissibility Proof ===");

    // Phase 2 で利用可能なすべての回転 (face, turns)
    let p2_moves: [(usize, i32); 10] = [
        (0, 1), (0, 2), (0, 3), // U, U2, U'
        (1, 2),                 // R2
        (2, 2),                 // F2
        (3, 1), (3, 2), (3, 3), // D, D2, D'
        (4, 2),                 // L2
        (5, 2),                 // B2
    ];

    let mut total_states = 0;
    let mut max_reduction = 0i32;

    // 6面のセンター状態 (各面 0..4) を全探索
    // ※ R, F, L, B は Phase 2 では 180° しか回せないため、奇数回転は到達不能だが、全 4^6 = 4096 状態で検証
    for c0 in 0..4 {
        for c1 in 0..4 {
            for c2 in 0..4 {
                for c3 in 0..4 {
                    for c4 in 0..4 {
                        for c5 in 0..4 {
                            total_states += 1;
                            let state = [c0, c1, c2, c3, c4, c5];
                            let h_before = min_phase2_center_moves(state) as i32;

                            for &(face, turn) in &p2_moves {
                                let mut next = state;
                                next[face] = (next[face] + turn) % 4;
                                let h_after = min_phase2_center_moves(next) as i32;
                                let reduction = h_before - h_after;
                                if reduction > max_reduction {
                                    max_reduction = reduction;
                                }
                                assert!(
                                    reduction <= 1,
                                    "Admissibility violation! 1 move reduced heuristic by {} > 1",
                                    reduction
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    println!("検証完了:");
    println!("- 探索状態数: {} 状態", total_states);
    println!("- 1手あたりのヒューリスティック減少量の最大値: {}", max_reduction);
    println!("=> 【証明成立】任意の Phase 2 遷移においてヒューリスティックは高々 1 しか減少しない。");
    println!("   したがって h(s) <= h*(s) (三角不等式とアドミッシビリティ) が数学的に厳密に成立する！");
}
