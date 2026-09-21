use crate::coord::{move_cube_18, RawCube};
use crate::cube::{apply, parse_moves};

#[derive(Debug, Clone)]
pub struct CfopPhase {
    pub name: &'static str,
    pub moves: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct CfopResult {
    pub moves: Vec<usize>,
    pub phases: Vec<CfopPhase>,
}

fn moves(text: &str) -> Vec<usize> {
    parse_moves(text).expect("定石パースエラー")
}

/// CFOP (Layer-By-Layer) ソルバー
pub fn solve(cube: &RawCube) -> Result<CfopResult, String> {
    let mut current = *cube;
    let mut all_phases = Vec::new();

    // 1. Cross (D面エッジ: 4:DR, 5:DF, 6:DL, 7:DB)
    let cross_moves = solve_cross(&current)?;
    current = apply(&current, &cross_moves);
    all_phases.push(CfopPhase {
        name: "Cross (クロス)",
        moves: cross_moves,
    });

    // 2. First Layer Corners (D面コーナー: 4:DFR, 5:DLF, 6:DBL, 7:DRB)
    let corner_moves = solve_first_layer(&current)?;
    current = apply(&current, &corner_moves);
    all_phases.push(CfopPhase {
        name: "First Layer (第1層コーナー)",
        moves: corner_moves,
    });

    // 3. Second Layer Edges (中層エッジ: 8:FR, 9:FL, 10:BL, 11:BR)
    let second_layer_moves = solve_second_layer(&current)?;
    current = apply(&current, &second_layer_moves);
    all_phases.push(CfopPhase {
        name: "Second Layer (中層エッジ)",
        moves: second_layer_moves,
    });

    // 4. OLL (Orientation of Last Layer: U面エッジ・コーナーの向き)
    let oll_moves = solve_oll(&current)?;
    current = apply(&current, &oll_moves);
    all_phases.push(CfopPhase {
        name: "OLL (ラストレイヤー向き)",
        moves: oll_moves,
    });

    // 5. PLL (Permutation of Last Layer: U面エッジ・コーナーの位置)
    let pll_moves = solve_pll(&current)?;
    current = apply(&current, &pll_moves);
    all_phases.push(CfopPhase {
        name: "PLL (ラストレイヤー配置)",
        moves: pll_moves,
    });

    debug_assert_eq!(
        current,
        RawCube::default(),
        "CFOP解法の検証に失敗しました。"
    );

    let mut total_moves = Vec::new();
    for p in &all_phases {
        total_moves.extend(&p.moves);
    }

    Ok(CfopResult {
        moves: total_moves,
        phases: all_phases,
    })
}

// -------------------------------------------------------------
// Step 1: D面 Cross (4:DR, 5:DF, 6:DL, 7:DB)
// -------------------------------------------------------------
fn is_cross_solved(c: &RawCube) -> bool {
    (4..8).all(|i| c.ep[i] as usize == i && c.eo[i] == 0)
}

fn solve_cross(cube: &RawCube) -> Result<Vec<usize>, String> {
    if is_cross_solved(cube) {
        return Ok(Vec::new());
    }
    let mut current = *cube;
    let mut total_moves = Vec::new();

    // 4つのD面エッジ (4..8) を1つずつ揃える
    for target in 4..8 {
        if current.ep[target] as usize == target && current.eo[target] == 0 {
            continue;
        }
        let mut found = false;
        for depth in 1..=5 {
            let mut path = Vec::new();
            if search_cross_edge(&current, depth, 99, target, &mut path) {
                for &m in &path {
                    current = current.multiply(move_cube_18(m));
                }
                total_moves.extend(path);
                found = true;
                break;
            }
        }
        if !found {
            return Err(format!("クロスエッジ {} の探索に失敗しました。", target));
        }
    }
    Ok(total_moves)
}

fn search_cross_edge(
    c: &RawCube,
    depth: usize,
    last_face: usize,
    target: usize,
    path: &mut Vec<usize>,
) -> bool {
    if depth == 0 {
        return (4..=target).all(|i| c.ep[i] as usize == i && c.eo[i] == 0);
    }

    for face in 0..6 {
        if face == last_face {
            continue;
        }
        for turn in 0..3 {
            let m = face * 3 + turn;
            let next = c.multiply(move_cube_18(m));
            path.push(m);
            if search_cross_edge(&next, depth - 1, face, target, path) {
                return true;
            }
            path.pop();
        }
    }
    false
}

// -------------------------------------------------------------
// Step 2: D面 Corners (4:DFR, 5:DLF, 6:DBL, 7:DRB)
// -------------------------------------------------------------
fn is_cross_intact(c: &RawCube) -> bool {
    (4..8).all(|i| c.ep[i] as usize == i && c.eo[i] == 0)
}

fn solve_first_layer(cube: &RawCube) -> Result<Vec<usize>, String> {
    let mut current = *cube;
    let mut moves = Vec::new();

    for corner_slot in 4..8 {
        if current.cp[corner_slot] as usize == corner_slot && current.co[corner_slot] == 0 {
            continue;
        }
        let solved_slots = corner_slot;
        let mut found = false;
        for depth in 1..=6 {
            let mut path = Vec::new();
            if search_corner(&current, depth, 99, solved_slots, corner_slot, &mut path) {
                for &m in &path {
                    current = current.multiply(move_cube_18(m));
                }
                moves.extend(path);
                found = true;
                break;
            }
        }
        if !found {
            return Err(format!(
                "第1層コーナー {} の探索に失敗しました。",
                corner_slot
            ));
        }
    }
    Ok(moves)
}

fn search_corner(
    c: &RawCube,
    depth: usize,
    last_face: usize,
    solved_slots: usize,
    target_slot: usize,
    path: &mut Vec<usize>,
) -> bool {
    if depth == 0 {
        return is_cross_intact(c)
            && (4..solved_slots).all(|i| c.cp[i] as usize == i && c.co[i] == 0)
            && c.cp[target_slot] as usize == target_slot
            && c.co[target_slot] == 0;
    }

    for face in 0..6 {
        if face == last_face {
            continue;
        }
        for turn in 0..3 {
            let m = face * 3 + turn;
            let next_cube = c.multiply(move_cube_18(m));
            path.push(m);
            if search_corner(&next_cube, depth - 1, face, solved_slots, target_slot, path) {
                return true;
            }
            path.pop();
        }
    }
    false
}

// -------------------------------------------------------------
// Step 3: Second Layer Edges (8:FR, 9:FL, 10:BL, 11:BR)
// -------------------------------------------------------------
fn is_first_layer_intact(c: &RawCube) -> bool {
    is_cross_intact(c) && (4..8).all(|i| c.cp[i] as usize == i && c.co[i] == 0)
}

// 中層インサートマクロ（標準CFOP: D面が第1層、U面がラストレイヤー）
fn get_slot_macros(slot: usize) -> Vec<Vec<usize>> {
    match slot {
        8 => vec![
            // FRスロット
            moves("U R U' R' U' F' U F"),
            moves("U' F' U F U R U' R'"),
        ],
        9 => vec![
            // FLスロット
            moves("U' L' U L U F U' F'"),
            moves("U F U' F' U' L' U L"),
        ],
        10 => vec![
            // BLスロット
            moves("U L U' L' U' B' U B"),
            moves("U' B' U B U L U' L'"),
        ],
        11 => vec![
            // BRスロット
            moves("U' R' U R U B U' B'"),
            moves("U B U' B' U' R' U R"),
        ],
        _ => vec![],
    }
}

fn solve_second_layer(cube: &RawCube) -> Result<Vec<usize>, String> {
    let mut current = *cube;
    let mut result_moves = Vec::new();

    for (step_idx, &target_slot) in [8, 9, 10, 11].iter().enumerate() {
        if current.ep[target_slot] as usize == target_slot && current.eo[target_slot] == 0 {
            continue;
        }

        let solved_slots = &[8, 9, 10, 11][..step_idx];

        let current_pos = match (0..12).position(|i| current.ep[i] as usize == target_slot) {
            Some(p) => p,
            None => {
                return Err(format!(
                    "第2層エッジ {} の探索に失敗しました。",
                    target_slot
                ))
            }
        };
        if (8..=11).contains(&current_pos) {
            // 中層にあるので抜き出す（1つ目のマクロでU層に追い出す）
            let eject_mac = &get_slot_macros(current_pos)[0];
            result_moves.extend(eject_mac.clone());
            current = apply(&current, eject_mac);
        }

        // これで目的のピースは U層 (0..4) にある
        let mut solved = false;
        let slot_macs = get_slot_macros(target_slot);

        for u_turns in 0..4 {
            let mut prefix = Vec::new();
            if u_turns > 0 {
                let m = match u_turns {
                    1 => 0, // U
                    2 => 1, // U2
                    _ => 2, // U'
                };

                prefix.push(m);
            }
            let c_u = apply(&current, &prefix);

            for mac in &slot_macs {
                let after = apply(&c_u, mac);
                if is_first_layer_intact(&after)
                    && solved_slots
                        .iter()
                        .all(|&s| after.ep[s] as usize == s && after.eo[s] == 0)
                    && after.ep[target_slot] as usize == target_slot
                    && after.eo[target_slot] == 0
                {
                    result_moves.extend(prefix);
                    result_moves.extend(mac.clone());
                    current = after;
                    solved = true;
                    break;
                }
            }
            if solved {
                break;
            }
        }

        if !solved {
            return Err(format!(
                "第2層エッジ {} の解決に失敗しました。",
                target_slot
            ));
        }
    }

    Ok(result_moves)
}

// -------------------------------------------------------------
// Step 4: OLL (U面の向き)
// -------------------------------------------------------------
fn is_f2l_intact(c: &RawCube) -> bool {
    is_first_layer_intact(c) && (8..12).all(|slot| c.ep[slot] as usize == slot && c.eo[slot] == 0)
}

fn is_oll_edges_solved(c: &RawCube) -> bool {
    (0..4).all(|i| c.eo[i] == 0)
}

fn is_oll_solved(c: &RawCube) -> bool {
    is_f2l_intact(c) && is_oll_edges_solved(c) && (0..4).all(|i| c.co[i] == 0)
}

fn solve_oll(cube: &RawCube) -> Result<Vec<usize>, String> {
    let mut current = *cube;
    let mut total_moves = Vec::new();

    // 4.1 エッジの向きを揃える (黄十字)
    if !is_oll_edges_solved(&current) {
        let edge_ops = vec![
            moves("F R U R' U' F'"),
            moves("F U R U' R' F'"),
            moves("U"),
            moves("U2"),
            moves("U'"),
        ];
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((current, Vec::new(), 0usize));
        let mut found_path = None;

        while let Some((c, path, count)) = queue.pop_front() {
            if is_oll_edges_solved(&c) {
                found_path = Some(path);
                break;
            }
            if count < 3 {
                for op in &edge_ops {
                    let next = apply(&c, op);
                    debug_assert!(is_f2l_intact(&next));
                    let mut next_path = path.clone();
                    next_path.extend(op.clone());
                    queue.push_back((next, next_path, count + 1));
                }
            }
        }

        let p = found_path.ok_or_else(|| "OLLエッジの解決に失敗しました。".to_string())?;
        current = apply(&current, &p);
        total_moves.extend(p);
    }

    // 4.2 コーナーの向きを揃える (黄色全面)
    if !is_oll_solved(&current) {
        let corner_ops = vec![
            moves("R U R' U R U2 R'"),
            moves("R U2 R' U' R U' R'"),
            moves("U"),
            moves("U2"),
            moves("U'"),
        ];
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((current, Vec::new(), 0usize));
        let mut found_path = None;

        while let Some((c, path, count)) = queue.pop_front() {
            if is_oll_solved(&c) {
                found_path = Some(path);
                break;
            }
            if count < 4 {
                for op in &corner_ops {
                    let next = apply(&c, op);
                    debug_assert!(is_f2l_intact(&next) && is_oll_edges_solved(&next));
                    let mut next_path = path.clone();
                    next_path.extend(op.clone());
                    queue.push_back((next, next_path, count + 1));
                }
            }
        }

        let p = found_path.ok_or_else(|| "OLLコーナーの解決に失敗しました。".to_string())?;
        total_moves.extend(p);
    }

    Ok(total_moves)
}

// -------------------------------------------------------------
// Step 5: PLL (U面の配置)
// -------------------------------------------------------------
fn solve_pll(cube: &RawCube) -> Result<Vec<usize>, String> {
    let mut current = *cube;
    let mut total_moves = Vec::new();

    // 5.1 コーナーの位置を揃える
    let corner_ops = vec![
        moves("R U R' U' R' F R2 U' R' U' R U R' F'"), // T-perm
        moves("F R U' R' U' R U R' F' R U R' U' R' F R F'"), // Y-perm
        moves("U"),
        moves("U2"),
        moves("U'"),
    ];

    let mut queue = std::collections::VecDeque::new();
    queue.push_back((current, Vec::new(), 0usize));
    let mut found_path = None;

    while let Some((c, path, count)) = queue.pop_front() {
        // コーナーの相対配置が正しいかチェック
        let mut ok = false;
        for u_turns in 0..4 {
            let pref = match u_turns {
                0 => vec![],
                1 => vec![0],
                2 => vec![1],
                _ => vec![2],
            };
            let tc = apply(&c, &pref);
            if (0..4).all(|i| tc.cp[i] as usize == i) {
                let mut full = path.clone();
                full.extend(pref);
                found_path = Some(full);
                ok = true;
                break;
            }
        }
        if ok {
            break;
        }
        if count >= 2 {
            continue;
        }
        for op in &corner_ops {
            let next = apply(&c, op);
            debug_assert!(is_oll_solved(&next));
            let mut next_path = path.clone();
            next_path.extend(op.clone());
            queue.push_back((next, next_path, count + 1));
        }
    }

    let p = found_path.ok_or_else(|| "PLLコーナーの解決に失敗しました。".to_string())?;
    current = apply(&current, &p);
    total_moves.extend(p);

    // 5.2 エッジの位置を揃える (完成へ)
    // コーナーを固定したままエッジだけを交換するマクロ（4面展開）
    let edge_ops = vec![
        // Ua-perm (4面)
        moves("R U' R U R U R U' R' U' R2"),
        moves("F U' F U F U F U' F' U' F2"),
        moves("L U' L U L U L U' L' U' L2"),
        moves("B U' B U B U B U' B' U' B2"),
        // Ub-perm (4面)
        moves("R2 U R U R' U' R' U' R' U R'"),
        moves("F2 U F U F' U' F' U' F' U F'"),
        moves("L2 U L U L' U' L' U' L' U L'"),
        moves("B2 U B U B' U' B' U' B' U B'"),
        // H-perm (対面エッジ交換)
        moves("R2 U2 R U2 R2 U2 R2 U2 R U2 R2"),
    ];

    let mut queue = std::collections::VecDeque::new();
    queue.push_back((current, Vec::new(), 0usize));
    let mut found_path = None;

    while let Some((c, path, count)) = queue.pop_front() {
        if c == RawCube::default() {
            found_path = Some(path);
            break;
        }
        if count < 2 {
            for op in &edge_ops {
                let next = apply(&c, op);
                debug_assert!(
                    is_f2l_intact(&next)
                        && (0..4).all(|i| next.cp[i] as usize == i && next.co[i] == 0)
                        && is_oll_edges_solved(&next)
                );
                let mut next_path = path.clone();
                next_path.extend(op.clone());
                queue.push_back((next, next_path, count + 1));
            }
        }
    }

    let p = found_path.ok_or_else(|| "PLLエッジの解決に失敗しました。".to_string())?;
    total_moves.extend(p);

    Ok(total_moves)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cube::scramble;

    #[test]
    fn test_cross_on_solved() {
        let cube = RawCube::default();
        let res = solve_cross(&cube).unwrap();
        assert!(res.is_empty());
    }

    #[test]
    fn test_cfop_scramble_10_moves() {
        for seed in 1..=5 {
            let sc = scramble(seed);
            let sc_10 = &sc[..10];
            let scrambled = apply(&RawCube::default(), sc_10);
            let res = solve(&scrambled).expect("10手スクランブルのCFOP解法失敗");
            let final_cube = apply(&scrambled, &res.moves);
            assert_eq!(final_cube, RawCube::default());
        }
    }

    #[test]
    fn test_cfop_scramble_full_25_moves() {
        for seed in 1..=3 {
            let sc = scramble(seed);
            let scrambled = apply(&RawCube::default(), &sc);
            let res = solve(&scrambled).expect("25手フルスクランブルのCFOP解法失敗");
            let final_cube = apply(&scrambled, &res.moves);
            assert_eq!(final_cube, RawCube::default());
            println!(
                "Seed {}: total {} moves (phases: {})",
                seed,
                res.moves.len(),
                res.phases
                    .iter()
                    .map(|p| format!("{}: {} moves", p.name, p.moves.len()))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }

    #[test]
    fn test_cfop_phases_on_solved() {
        let cube = RawCube::default();
        // 完成状態での各フェーズ呼び出し
        let full = solve(&cube).unwrap();
        assert!(full.moves.is_empty());

        assert!(solve_first_layer(&cube).unwrap().is_empty());
        assert!(solve_second_layer(&cube).unwrap().is_empty());
        assert!(solve_oll(&cube).unwrap().is_empty());
        assert!(solve_pll(&cube).unwrap().is_empty());

        // 各判定関数の網羅
        assert!(is_cross_solved(&cube));
        assert!(is_cross_intact(&cube));
        assert!(is_first_layer_intact(&cube));
        assert!(is_f2l_intact(&cube));
        assert!(is_oll_edges_solved(&cube));
        assert!(is_oll_solved(&cube));
    }

    #[test]
    fn test_cfop_error_paths_and_helpers() {
        use crate::coord::{Corner, Edge};

        assert!(get_slot_macros(99).is_empty());

        // クロスエラー (目的のエッジが存在しない)
        let mut broken_cross = RawCube::default();
        broken_cross.ep[4] = Edge::UR;
        assert!(solve_cross(&broken_cross).is_err());

        // 第1層エラー (目的のコーナーが存在しない)
        let mut broken_c1 = RawCube::default();
        broken_c1.cp[4] = Corner::UFR;
        assert!(solve_first_layer(&broken_c1).is_err());

        // 第2層エラー (目的のエッジが存在しない)
        let mut broken_e2_none = RawCube::default();
        broken_e2_none.ep[8] = Edge::UR;
        assert!(solve_second_layer(&broken_e2_none).is_err());

        // 第2層エラー (第1層が崩れているため解決不可)
        let mut broken_e2_unsolvable = RawCube::default();
        broken_e2_unsolvable.ep.swap(0, 8);
        broken_e2_unsolvable.eo[4] = 1;
        assert!(solve_second_layer(&broken_e2_unsolvable).is_err());

        // OLLエラー (単一Uエッジ反転でパリティ不正)
        let mut broken_oll = RawCube::default();
        broken_oll.eo[0] = 1;
        assert!(solve_oll(&broken_oll).is_err());

        // PLLコーナーエラー (U層コーナーが欠損しており解決不可)
        let mut broken_pll = RawCube::default();
        broken_pll.cp[0] = Corner::DLF;
        assert!(solve_pll(&broken_pll).is_err());

        // OLLコーナーエラー (単一コーナー反転でパリティ不正)
        let mut broken_oll_c = RawCube::default();
        broken_oll_c.co[0] = 1;
        assert!(solve_oll(&broken_oll_c).is_err());

        // PLLエッジエラー (単一エッジ交換でパリティ不正)
        let mut broken_pll_e = RawCube::default();
        broken_pll_e.ep.swap(0, 1);
        assert!(solve_pll(&broken_pll_e).is_err());

        // solve() における各フェーズのエラー伝播 (?) の網羅
        assert!(solve(&broken_cross).is_err());
        assert!(solve(&broken_c1).is_err());
        assert!(solve(&broken_e2_none).is_err());
        assert!(solve(&broken_oll).is_err());
        assert!(solve(&broken_pll).is_err());
    }
}
