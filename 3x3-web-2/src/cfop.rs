use crate::coord::{move_cube_18, RawCube};
use crate::cube::{apply, parse_moves};
use std::sync::OnceLock;
use web_time::Instant;

#[derive(Debug, Clone)]
pub struct CfopPhase {
    pub name: &'static str,
    pub moves: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct CfopResult {
    pub moves: Vec<usize>,
    pub phases: Vec<CfopPhase>,
    pub nodes: u64,
}

fn moves(text: &str) -> Vec<usize> {
    parse_moves(text).expect("定石パースエラー")
}

struct SolverState {
    start: Instant,
    budget_ms: u128,
    nodes: u64,
}

impl SolverState {
    fn new(budget_ms: u32) -> Self {
        Self {
            start: Instant::now(),
            budget_ms: budget_ms as u128,
            nodes: 0,
        }
    }

    fn check_timeout(&mut self) -> Result<(), String> {
        self.nodes += 1;
        if self.budget_ms == 0
            || (self.nodes & 511 == 0 && self.start.elapsed().as_millis() >= self.budget_ms)
        {
            Err("探索時間の上限に達しました。".to_owned())
        } else {
            Ok(())
        }
    }
}

/// CFOP (Layer-By-Layer) ソルバー
pub fn solve(cube: &RawCube, budget_ms: u32) -> Result<CfopResult, String> {
    let mut state = SolverState::new(budget_ms);
    if budget_ms == 0 {
        return Err("探索時間の上限に達しました。".to_owned());
    }
    let mut current = *cube;
    let mut all_phases = Vec::new();

    // 1. Cross (D面エッジ: 4:DR, 5:DF, 6:DL, 7:DB)
    let cross_moves = solve_cross(&current, &mut state)?;
    current = apply(&current, &cross_moves);
    all_phases.push(CfopPhase {
        name: "Cross (クロス)",
        moves: cross_moves,
    });
    state.check_timeout()?;

    // 2. First Layer Corners (D面コーナー: 4:DFR, 5:DLF, 6:DBL, 7:DRB)
    let corner_moves = solve_first_layer(&current, &mut state)?;
    current = apply(&current, &corner_moves);
    all_phases.push(CfopPhase {
        name: "First Layer (第1層コーナー)",
        moves: corner_moves,
    });
    state.check_timeout()?;

    // 3. Second Layer Edges (中層エッジ: 8:FR, 9:FL, 10:BL, 11:BR)
    let second_layer_moves = solve_second_layer(&current, &mut state)?;
    current = apply(&current, &second_layer_moves);
    all_phases.push(CfopPhase {
        name: "Second Layer (中層エッジ)",
        moves: second_layer_moves,
    });
    state.check_timeout()?;

    // 4. OLL (Orientation of Last Layer: U面エッジ・コーナーの向き)
    let oll_moves = solve_oll(&current, &mut state)?;
    current = apply(&current, &oll_moves);
    all_phases.push(CfopPhase {
        name: "OLL (ラストレイヤー向き)",
        moves: oll_moves,
    });
    state.check_timeout()?;

    // 5. PLL (Permutation of Last Layer: U面エッジ・コーナーの位置)
    let pll_moves = solve_pll(&current, &mut state)?;
    all_phases.push(CfopPhase {
        name: "PLL (ラストレイヤー配置)",
        moves: pll_moves.clone(),
    });

    #[cfg(debug_assertions)]
    {
        current = apply(&current, &pll_moves);
        debug_assert_eq!(
            current,
            RawCube::default(),
            "CFOP解法の検証に失敗しました。"
        );
    }

    let mut total_moves = Vec::new();
    for p in &all_phases {
        total_moves.extend(&p.moves);
    }

    Ok(CfopResult {
        moves: total_moves,
        phases: all_phases,
        nodes: state.nodes,
    })
}

// -------------------------------------------------------------
// Step 1: D面 Cross (4:DR, 5:DF, 6:DL, 7:DB)
// -------------------------------------------------------------
fn is_cross_solved(c: &RawCube) -> bool {
    (4..8).all(|i| c.ep[i] as usize == i && c.eo[i] == 0)
}

fn solve_cross(cube: &RawCube, state: &mut SolverState) -> Result<Vec<usize>, String> {
    if is_cross_solved(cube) {
        return Ok(Vec::new());
    }
    let mut current = *cube;
    let mut total_moves = Vec::new();

    // 4つのD面エッジ (4..8) を1つずつ揃える
    for target in 4..8 {
        state.check_timeout()?;
        if current.ep[target] as usize == target && current.eo[target] == 0 {
            continue;
        }
        let mut found = false;
        for depth in 1..=5 {
            let mut path = Vec::new();
            if search_cross_edge(&current, depth, 99, target, &mut path, state)? {
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

#[inline]
fn redundant(face: usize, last: usize) -> bool {
    face == last || ((3..6).contains(&last) && face + 3 == last)
}

fn search_cross_edge(
    c: &RawCube,
    depth: usize,
    last_face: usize,
    target: usize,
    path: &mut Vec<usize>,
    state: &mut SolverState,
) -> Result<bool, String> {
    state.check_timeout()?;
    if depth == 0 {
        return Ok((4..=target).all(|i| c.ep[i] as usize == i && c.eo[i] == 0));
    }

    for face in 0..6 {
        if redundant(face, last_face) {
            continue;
        }
        for turn in 0..3 {
            let m = face * 3 + turn;
            let next = c.multiply(move_cube_18(m));
            path.push(m);
            if search_cross_edge(&next, depth - 1, face, target, path, state)? {
                return Ok(true);
            }
            path.pop();
        }
    }
    Ok(false)
}

// -------------------------------------------------------------
// Step 2: D面 Corners (4:DFR, 5:DLF, 6:DBL, 7:DRB)
// -------------------------------------------------------------
fn is_cross_intact(c: &RawCube) -> bool {
    (4..8).all(|i| c.ep[i] as usize == i && c.eo[i] == 0)
}

fn solve_first_layer(cube: &RawCube, state: &mut SolverState) -> Result<Vec<usize>, String> {
    let mut current = *cube;
    let mut moves = Vec::new();

    for corner_slot in 4..8 {
        state.check_timeout()?;
        if current.cp[corner_slot] as usize == corner_slot && current.co[corner_slot] == 0 {
            continue;
        }
        let solved_slots = corner_slot;
        let mut found = false;
        for depth in 1..=6 {
            let mut path = Vec::new();
            if search_corner(
                &current,
                depth,
                99,
                solved_slots,
                corner_slot,
                &mut path,
                state,
            )? {
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
    state: &mut SolverState,
) -> Result<bool, String> {
    state.check_timeout()?;
    if depth == 0 {
        return Ok(is_cross_intact(c)
            && (4..solved_slots).all(|i| c.cp[i] as usize == i && c.co[i] == 0)
            && c.cp[target_slot] as usize == target_slot
            && c.co[target_slot] == 0);
    }

    let broken_cross = (4..8)
        .filter(|&i| c.ep[i] as usize != i || c.eo[i] != 0)
        .count();
    if broken_cross > depth {
        return Ok(false);
    }

    for face in 0..6 {
        // D面 (face == 3) は除外する:
        // 1. 第1層コーナーの挿入にD面回転は不要
        // 2. D面を回すと broken_cross = 4 となり、D' 1手で復帰可能にもかかわらず
        //    broken_cross > depth による非アドミッシブルな誤枝刈りが発生する
        // 3. D面を除外することで各手番で動くクロスエッジは高々1個となり、
        //    broken_cross <= depth が数学的にアドミッシブルな許容下界として成立する
        if face == 3 || redundant(face, last_face) {
            continue;
        }
        for turn in 0..3 {
            let m = face * 3 + turn;
            let next_cube = c.multiply(move_cube_18(m));
            path.push(m);
            if search_corner(
                &next_cube,
                depth - 1,
                face,
                solved_slots,
                target_slot,
                path,
                state,
            )? {
                return Ok(true);
            }
            path.pop();
        }
    }
    Ok(false)
}

// -------------------------------------------------------------
// Step 3: Second Layer Edges (8:FR, 9:FL, 10:BL, 11:BR)
// -------------------------------------------------------------
fn is_first_layer_intact(c: &RawCube) -> bool {
    is_cross_intact(c) && (4..8).all(|i| c.cp[i] as usize == i && c.co[i] == 0)
}

// 中層インサートマクロ（標準CFOP: D面が第1層、U面がラストレイヤー）
fn get_slot_macros(slot: usize) -> &'static [Vec<usize>] {
    static SLOT_8: OnceLock<Vec<Vec<usize>>> = OnceLock::new();
    static SLOT_9: OnceLock<Vec<Vec<usize>>> = OnceLock::new();
    static SLOT_10: OnceLock<Vec<Vec<usize>>> = OnceLock::new();
    static SLOT_11: OnceLock<Vec<Vec<usize>>> = OnceLock::new();

    match slot {
        8 => SLOT_8.get_or_init(|| {
            vec![
                moves("U R U' R' U' F' U F"),
                moves("U' F' U F U R U' R'"),
            ]
        }),
        9 => SLOT_9.get_or_init(|| {
            vec![
                moves("U' L' U L U F U' F'"),
                moves("U F U' F' U' L' U L"),
            ]
        }),
        10 => SLOT_10.get_or_init(|| {
            vec![
                moves("U L U' L' U' B' U B"),
                moves("U' B' U B U L U' L'"),
            ]
        }),
        11 => SLOT_11.get_or_init(|| {
            vec![
                moves("U' R' U R U B U' B'"),
                moves("U B U' B' U' R' U R"),
            ]
        }),
        _ => &[],
    }
}

fn solve_second_layer(cube: &RawCube, state: &mut SolverState) -> Result<Vec<usize>, String> {
    let mut current = *cube;
    let mut result_moves = Vec::new();

    for (step_idx, &target_slot) in [8, 9, 10, 11].iter().enumerate() {
        state.check_timeout()?;
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

            for mac in slot_macs {
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

fn oll_edge_ops() -> &'static [Vec<usize>] {
    static OPS: OnceLock<Vec<Vec<usize>>> = OnceLock::new();
    OPS.get_or_init(|| {
        vec![
            moves("F R U R' U' F'"),
            moves("F U R U' R' F'"),
            moves("U"),
            moves("U2"),
            moves("U'"),
        ]
    })
}

fn oll_corner_ops() -> &'static [Vec<usize>] {
    static OPS: OnceLock<Vec<Vec<usize>>> = OnceLock::new();
    OPS.get_or_init(|| {
        vec![
            moves("R U R' U R U2 R'"),
            moves("R U2 R' U' R U' R'"),
            moves("U"),
            moves("U2"),
            moves("U'"),
        ]
    })
}

fn solve_oll(cube: &RawCube, state: &mut SolverState) -> Result<Vec<usize>, String> {
    let mut current = *cube;
    let mut total_moves = Vec::new();

    // 4.1 エッジの向きを揃える (黄十字)
    if !is_oll_edges_solved(&current) {
        let edge_ops = oll_edge_ops();
        let mut queue = std::collections::VecDeque::new();
        let mut seen = std::collections::HashSet::new();
        seen.insert(current);
        queue.push_back((current, Vec::new(), 0usize));
        let mut found_path = None;

        while let Some((c, path, count)) = queue.pop_front() {
            state.check_timeout()?;
            if is_oll_edges_solved(&c) {
                found_path = Some(path);
                break;
            }
            if count < 3 {
                for op in edge_ops {
                    let next = apply(&c, op);
                    debug_assert!(is_f2l_intact(&next));
                    if seen.insert(next) {
                        let mut next_path = path.clone();
                        next_path.extend(op);
                        queue.push_back((next, next_path, count + 1));
                    }
                }
            }
        }

        let p = found_path.ok_or_else(|| "OLLエッジの解決に失敗しました。".to_string())?;
        current = apply(&current, &p);
        total_moves.extend(p);
    }

    // 4.2 コーナーの向きを揃える (黄色全面)
    if !is_oll_solved(&current) {
        let corner_ops = oll_corner_ops();
        let mut queue = std::collections::VecDeque::new();
        let mut seen = std::collections::HashSet::new();
        seen.insert(current);
        queue.push_back((current, Vec::<usize>::new(), 0usize));
        let mut found_path = None;

        while let Some((c, path, count)) = queue.pop_front() {
            state.check_timeout()?;
            if is_oll_solved(&c) {
                found_path = Some(path);
                break;
            }
            if count < 4 {
                for op in corner_ops {
                    let next = apply(&c, op);
                    debug_assert!(is_f2l_intact(&next) && is_oll_edges_solved(&next));
                    if seen.insert(next) {
                        let mut next_path = path.clone();
                        next_path.extend(op);
                        queue.push_back((next, next_path, count + 1));
                    }
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
fn pll_corner_ops() -> &'static [Vec<usize>] {
    static OPS: OnceLock<Vec<Vec<usize>>> = OnceLock::new();
    OPS.get_or_init(|| {
        vec![
            moves("R U R' U' R' F R2 U' R' U' R U R' F'"), // T-perm
            moves("F R U' R' U' R U R' F' R U R' U' R' F R F'"), // Y-perm
            moves("U"),
            moves("U2"),
            moves("U'"),
        ]
    })
}

fn pll_edge_ops() -> &'static [Vec<usize>] {
    static OPS: OnceLock<Vec<Vec<usize>>> = OnceLock::new();
    OPS.get_or_init(|| {
        vec![
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
        ]
    })
}

fn solve_pll(cube: &RawCube, state: &mut SolverState) -> Result<Vec<usize>, String> {
    let mut current = *cube;
    let mut total_moves = Vec::new();

    // 5.1 コーナーの位置を揃える
    let corner_ops = pll_corner_ops();

    let mut queue = std::collections::VecDeque::new();
    let mut seen = std::collections::HashSet::new();
    seen.insert(current);
    queue.push_back((current, Vec::new(), 0usize));
    let mut found_path = None;

    while let Some((c, path, count)) = queue.pop_front() {
        state.check_timeout()?;
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
        for op in corner_ops {
            let next = apply(&c, op);
            debug_assert!(is_oll_solved(&next));
            if seen.insert(next) {
                let mut next_path = path.clone();
                next_path.extend(op);
                queue.push_back((next, next_path, count + 1));
            }
        }
    }

    let p = found_path.ok_or_else(|| "PLLコーナーの解決に失敗しました。".to_string())?;
    current = apply(&current, &p);
    total_moves.extend(p);

    // 5.2 エッジの位置を揃える (完成へ)
    // コーナーを固定したままエッジだけを交換するマクロ（4面展開およびZ-perm）
    let edge_ops = pll_edge_ops();

    let mut queue = std::collections::VecDeque::new();
    let mut seen = std::collections::HashSet::new();
    seen.insert(current);
    queue.push_back((current, Vec::new(), 0usize));
    let mut found_path = None;

    while let Some((c, path, count)) = queue.pop_front() {
        state.check_timeout()?;
        if c == RawCube::default() {
            found_path = Some(path);
            break;
        }
        if count < 2 {
            for op in edge_ops {
                let next = apply(&c, op);
                #[cfg(debug_assertions)]
                debug_assert!(
                    is_f2l_intact(&next)
                        && (0..4).all(|i| next.cp[i] as usize == i && next.co[i] == 0)
                        && is_oll_edges_solved(&next)
                );
                if seen.insert(next) {
                    let mut next_path = path.clone();
                    next_path.extend(op.clone());
                    queue.push_back((next, next_path, count + 1));
                }
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
        let res = solve_cross(&cube, &mut SolverState::new(10_000)).unwrap();
        assert!(res.is_empty());
    }

    #[test]
    fn test_cfop_corner_search_d_face_exclusion() {
        // D面 (face == 3) がコーナー解法の手順に含まれないことと、
        // アドミッシブルな探索によりクロスを崩さず第1層を解決できることを検証
        let cube = RawCube::default();
        let test_scramble = moves("R U R' U'");
        let scrambled = apply(&cube, &test_scramble);
        assert!(is_cross_intact(&scrambled));
        assert!(!is_first_layer_intact(&scrambled));

        let mut state = SolverState::new(10_000);
        let first_layer_moves = solve_first_layer(&scrambled, &mut state).expect("第1層解決成功");

        let solved_fl = apply(&scrambled, &first_layer_moves);
        assert!(is_cross_intact(&solved_fl));
        assert!(is_first_layer_intact(&solved_fl));

        // 第1層コーナー解法の手順にD面 (face == 3: D, D2, D') が一切含まれていないこと
        for &m in &first_layer_moves {
            let face = m / 3;
            assert_ne!(face, 3, "第1層コーナー解法にD面回転が含まれてはならない");
        }
    }

    #[test]
    fn test_cfop_scramble_10_moves() {
        for seed in 1..=5 {
            let sc = scramble(seed);
            let sc_10 = &sc[..10];
            let scrambled = apply(&RawCube::default(), sc_10);
            let res = solve(&scrambled, 10_000).expect("10手スクランブルのCFOP解法失敗");
            let final_cube = apply(&scrambled, &res.moves);
            assert_eq!(final_cube, RawCube::default());
        }
    }

    #[test]
    fn test_cfop_scramble_full_25_moves() {
        for seed in 1..=3 {
            let sc = scramble(seed);
            let scrambled = apply(&RawCube::default(), &sc);
            let res = solve(&scrambled, 10_000).expect("25手フルスクランブルのCFOP解法失敗");
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
        let full = solve(&cube, 10_000).unwrap();
        assert!(full.moves.is_empty());

        assert!(solve_first_layer(&cube, &mut SolverState::new(10_000))
            .unwrap()
            .is_empty());
        assert!(solve_second_layer(&cube, &mut SolverState::new(10_000))
            .unwrap()
            .is_empty());
        assert!(solve_oll(&cube, &mut SolverState::new(10_000))
            .unwrap()
            .is_empty());
        assert!(solve_pll(&cube, &mut SolverState::new(10_000))
            .unwrap()
            .is_empty());

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
        assert!(solve_cross(&broken_cross, &mut SolverState::new(10_000)).is_err());

        // 第1層エラー (目的のコーナーが存在しない)
        let mut broken_c1 = RawCube::default();
        broken_c1.cp[4] = Corner::UFR;
        assert!(solve_first_layer(&broken_c1, &mut SolverState::new(10_000)).is_err());

        // 第2層エラー (目的のエッジが存在しない)
        let mut broken_e2_none = RawCube::default();
        broken_e2_none.ep[8] = Edge::UR;
        assert!(solve_second_layer(&broken_e2_none, &mut SolverState::new(10_000)).is_err());

        // 第2層エラー (第1層が崩れているため解決不可)
        let mut broken_e2_unsolvable = RawCube::default();
        broken_e2_unsolvable.ep.swap(0, 8);
        broken_e2_unsolvable.eo[4] = 1;
        assert!(solve_second_layer(&broken_e2_unsolvable, &mut SolverState::new(10_000)).is_err());

        // OLLエラー (単一Uエッジ反転でパリティ不正)
        let mut broken_oll = RawCube::default();
        broken_oll.eo[0] = 1;
        assert!(solve_oll(&broken_oll, &mut SolverState::new(10_000)).is_err());

        // PLLコーナーエラー (U層コーナーが欠損しており解決不可)
        let mut broken_pll = RawCube::default();
        broken_pll.cp[0] = Corner::DLF;
        assert!(solve_pll(&broken_pll, &mut SolverState::new(10_000)).is_err());

        // OLLコーナーエラー (単一コーナー反転でパリティ不正)
        let mut broken_oll_c = RawCube::default();
        broken_oll_c.co[0] = 1;
        assert!(solve_oll(&broken_oll_c, &mut SolverState::new(10_000)).is_err());

        // PLLエッジエラー (単一エッジ交換でパリティ不正)
        let mut broken_pll_e = RawCube::default();
        broken_pll_e.ep.swap(0, 1);
        assert!(solve_pll(&broken_pll_e, &mut SolverState::new(10_000)).is_err());

        // solve() における各フェーズのエラー伝播 (?) の網羅
        assert!(solve(&broken_cross, 10_000).is_err());
        assert!(solve(&broken_c1, 10_000).is_err());
        assert!(solve(&broken_e2_none, 10_000).is_err());
        assert!(solve(&broken_oll, 10_000).is_err());
        assert!(solve(&broken_pll, 10_000).is_err());
        assert!(solve(&RawCube::default(), 0).is_err());

        // search_corner 再帰中の check_timeout による Err 伝播 (?)
        let sc = crate::cube::parse_moves("U").unwrap();
        let c1_scrambled = apply(&RawCube::default(), &sc);
        let mut state_timeout = SolverState::new(10);
        state_timeout.nodes = 510;
        state_timeout.start = web_time::Instant::now() - std::time::Duration::from_millis(10_000);
        let mut path = Vec::new();
        let res = search_corner(&c1_scrambled, 2, 99, 4, 4, &mut path, &mut state_timeout);
        assert!(res.is_err());

        // solve_first_layer での search_corner エラー伝播 (?)
        let mut state_c1_timeout = SolverState::new(10);
        state_c1_timeout.nodes = 510;
        state_c1_timeout.start =
            web_time::Instant::now() - std::time::Duration::from_millis(10_000);
        assert!(solve_first_layer(&c1_scrambled, &mut state_c1_timeout).is_err());
    }

    #[test]
    fn test_cfop_redundant() {
        assert!(redundant(0, 0)); // face == last
        assert!(redundant(0, 3)); // 0 + 3 == 3 (UとD)
        assert!(redundant(1, 4)); // 1 + 3 == 4 (RとL)
        assert!(redundant(2, 5)); // 2 + 3 == 5 (FとB)
        assert!(!redundant(0, 1)); // UとRは非冗長
        assert!(!redundant(3, 0)); // DとU (順序固定)
    }
}
