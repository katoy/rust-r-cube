use crate::coord::{move_cube_18, RawCube};
use crate::cube::apply;
use std::sync::OnceLock;
use web_time::Instant;

fn is_valid_g3_corner_permutation(cp: u16) -> bool {
    static G3_CP_SET: OnceLock<[bool; 40320]> = OnceLock::new();
    let set = G3_CP_SET.get_or_init(|| {
        let mut table = [false; 40320];
        let mut queue = std::collections::VecDeque::new();
        let g3_half_turns = [1, 4, 7, 10, 13, 16]; // U2, R2, F2, D2, L2, B2

        let solved = RawCube::default();
        table[solved.get_cp() as usize] = true;
        queue.push_back(solved);

        while let Some(c) = queue.pop_front() {
            for &m in &g3_half_turns {
                let next = c.multiply(move_cube_18(m));
                let next_cp = next.get_cp() as usize;
                if !table[next_cp] {
                    table[next_cp] = true;
                    queue.push_back(next);
                }
            }
        }
        table
    });
    set[cp as usize]
}

#[derive(Debug, Clone)]
pub struct ThistlePhase {
    pub name: &'static str,
    pub moves: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct ThistleResult {
    pub moves: Vec<usize>,
    pub phases: Vec<ThistlePhase>,
}

pub struct ThistlethwaiteSearch {
    start: Instant,
    budget_ms: u128,
    main_budget_ms: u128,
    pub nodes: u64,
    pub timed_out: bool,
}

impl ThistlethwaiteSearch {
    pub fn new(budget_ms: u32) -> Self {
        let b = budget_ms as u128;
        let main_b = (b * 7 / 10).min(b.saturating_sub(50));
        Self {
            start: Instant::now(),
            budget_ms: b,
            main_budget_ms: main_b,
            nodes: 0,
            timed_out: false,
        }
    }

    pub fn exhausted(&mut self) -> bool {
        if self.timed_out {
            return true;
        }
        if self.budget_ms == 0
            || (self.nodes & 4095 == 0 && self.start.elapsed().as_millis() >= self.main_budget_ms)
        {
            self.timed_out = true;
            return true;
        }
        false
    }

    pub fn solve(&mut self, cube: &RawCube) -> Result<ThistleResult, String> {
        let mut current = *cube;
        let mut all_phases = Vec::new();

        // Phase 1: G0 -> G1 (全エッジの向き flip = 0)
        let p1 = match self.solve_phase1(&current) {
            Ok(p) => p,
            Err(_) => return self.fallback_solve(&current, all_phases),
        };
        current = apply(&current, &p1);
        all_phases.push(ThistlePhase {
            name: "Phase 1 (G0→G1: エッジ向き)",
            moves: p1,
        });

        // Phase 2: G1 -> G2 (コーナー向き twist = 0 & Eスライスエッジ)
        let p2 = match self.solve_phase2(&current) {
            Ok(p) => p,
            Err(_) => return self.fallback_solve(&current, all_phases),
        };
        current = apply(&current, &p2);
        all_phases.push(ThistlePhase {
            name: "Phase 2 (G1→G2: コーナー向き&Eスライス)",
            moves: p2,
        });

        // Phase 3: G2 -> G3 (ピースのオービット分離 & パリティ)
        let p3 = match self.solve_phase3(&current) {
            Ok(p) => p,
            Err(_) => return self.fallback_solve(&current, all_phases),
        };
        current = apply(&current, &p3);
        all_phases.push(ThistlePhase {
            name: "Phase 3 (G2→G3: オービット分離)",
            moves: p3,
        });

        // Phase 4: G3 -> G4 (完成)
        let p4 = match self.solve_phase4(&current) {
            Ok(p) => p,
            Err(_) => return self.fallback_solve(&current, all_phases),
        };
        #[cfg(any(debug_assertions, test))]
        let current = apply(&current, &p4);
        all_phases.push(ThistlePhase {
            name: "Phase 4 (G3→G4: 最終解決)",
            moves: p4,
        });

        #[cfg(any(debug_assertions, test))]
        assert_eq!(
            current,
            RawCube::default(),
            "Thistlethwaite解法の検証に失敗しました。"
        );

        let mut total = Vec::new();
        for p in &all_phases {
            total.extend(&p.moves);
        }

        Ok(ThistleResult {
            moves: total,
            phases: all_phases,
        })
    }

    fn fallback_solve(
        &mut self,
        current: &RawCube,
        mut all_phases: Vec<ThistlePhase>,
    ) -> Result<ThistleResult, String> {
        let elapsed_ms = self.start.elapsed().as_millis() as u32;
        let remaining_ms = (self.budget_ms as u32).saturating_sub(elapsed_ms);
        if remaining_ms == 0 {
            return Err("探索時間の上限に達しました。".to_owned());
        }
        let mut fallback = crate::search::Search::new(remaining_ms);
        let moves = fallback
            .solve(current)
            .ok_or_else(|| "探索時間の上限に達しました。".to_owned())?;
        self.nodes += fallback.nodes;
        if !moves.is_empty() {
            all_phases.push(ThistlePhase {
                name: "Phase 4 (最終解決フォールバック)",
                moves,
            });
        }
        let mut total = Vec::new();
        for p in &all_phases {
            total.extend(&p.moves);
        }
        Ok(ThistleResult {
            moves: total,
            phases: all_phases,
        })
    }

    // Phase 1: G0 -> G1 (すべての手OK)
    fn is_g1(&self, c: &RawCube) -> bool {
        c.eo.iter().all(|&o| o == 0)
    }

    fn solve_phase1(&mut self, cube: &RawCube) -> Result<Vec<usize>, String> {
        if self.is_g1(cube) {
            return Ok(Vec::new());
        }
        for depth in 1..=8 {
            let mut path = Vec::new();
            if self.search_g1(cube, depth, 99, &mut path) {
                return Ok(path);
            }
            if self.exhausted() {
                break;
            }
        }
        Err("Phase 1の探索に失敗しました。".into())
    }

    fn search_g1(
        &mut self,
        c: &RawCube,
        depth: u8,
        last_face: usize,
        path: &mut Vec<usize>,
    ) -> bool {
        self.nodes += 1;
        if self.exhausted() {
            return false;
        }
        if depth == 0 {
            return self.is_g1(c);
        }
        let bad_eo = c.eo.iter().filter(|&&o| o != 0).count() as u8;
        if bad_eo.div_ceil(4) > depth {
            return false;
        }

        for face in 0..6 {
            if redundant(face, last_face) {
                continue;
            }
            for turn in 0..3 {
                let m = face * 3 + turn;
                let next = c.multiply(move_cube_18(m));
                path.push(m);
                if self.search_g1(&next, depth - 1, face, path) {
                    return true;
                }
                path.pop();
            }
        }
        false
    }

    // Phase 2: G1 -> G2 (許可手: U, D, L, R, F2, B2)
    fn is_g2(&self, c: &RawCube) -> bool {
        self.is_g1(c)
            && c.co.iter().all(|&o| o == 0)
            && (8..12).all(|slot| {
                let p = c.ep[slot] as usize;
                (8..=11).contains(&p)
            })
    }

    fn solve_phase2(&mut self, cube: &RawCube) -> Result<Vec<usize>, String> {
        if self.is_g2(cube) {
            return Ok(Vec::new());
        }
        let g1_moves: Vec<usize> = (0..18)
            .filter(|&m| {
                let f = m / 3;
                let t = m % 3;
                // F(2), B(5) は半回転のみ (t == 1)
                (f != 2 && f != 5) || t == 1
            })
            .collect();

        for depth in 1..=12 {
            let mut path = Vec::new();
            if self.search_g2(cube, depth, 99, &g1_moves, &mut path) {
                return Ok(path);
            }
            if self.exhausted() {
                break;
            }
        }
        Err("Phase 2の探索に失敗しました。".into())
    }

    fn search_g2(
        &mut self,
        c: &RawCube,
        depth: u8,
        last_face: usize,
        allowed: &[usize],
        path: &mut Vec<usize>,
    ) -> bool {
        self.nodes += 1;
        if self.exhausted() {
            return false;
        }
        if depth == 0 {
            return self.is_g2(c);
        }
        let bad_co = c.co.iter().filter(|&&o| o != 0).count() as u8;
        let bad_eslice = (8..12).filter(|&i| (c.ep[i] as usize) < 8).count() as u8;
        if bad_co.div_ceil(4) > depth || bad_eslice.div_ceil(2) > depth {
            return false;
        }

        for &m in allowed {
            let face = m / 3;
            if redundant(face, last_face) {
                continue;
            }
            let next = c.multiply(move_cube_18(m));
            path.push(m);
            if self.search_g2(&next, depth - 1, face, allowed, path) {
                return true;
            }
            path.pop();
        }
        false
    }

    // Phase 3: G2 -> G3 (許可手: U, D, L2, R2, F2, B2)
    pub(crate) fn is_g3(&self, c: &RawCube) -> bool {
        if !self.is_g2(c) {
            return false;
        }
        // 1. エッジ軌道: Sスライス {0,2,4,6} と Mスライス {1,3,5,7} が分離されていること
        let s_slice = [0, 2, 4, 6]
            .iter()
            .all(|&i| matches!(c.ep[i] as usize, 0 | 2 | 4 | 6));
        let m_slice = [1, 3, 5, 7]
            .iter()
            .all(|&i| matches!(c.ep[i] as usize, 1 | 3 | 5 | 7));
        if !s_slice || !m_slice {
            return false;
        }
        // 2. コーナーテトラヘドロン軌道: {0,2,5,7} と {1,3,4,6}
        let tetrad1 = [0, 2, 5, 7]
            .iter()
            .all(|&i| matches!(c.cp[i] as usize, 0 | 2 | 5 | 7));
        let tetrad2 = [1, 3, 4, 6]
            .iter()
            .all(|&i| matches!(c.cp[i] as usize, 1 | 3 | 4 | 6));
        if !tetrad1 || !tetrad2 {
            return false;
        }
        // 3. パリティ: G3 のすべての操作は偶置換なので、コーナー置換パリティは偶数 (0)
        if crate::cube::parity(&c.cp.map(|x| x as u8)) != 0 {
            return false;
        }
        // 4. 半回転群 <U2, D2, L2, R2, F2, B2> で到達可能な 96 通りのコーナー置換に完全に属すること
        is_valid_g3_corner_permutation(c.get_cp())
    }

    fn solve_phase3(&mut self, cube: &RawCube) -> Result<Vec<usize>, String> {
        if self.is_g3(cube) {
            return Ok(Vec::new());
        }
        let g2_moves: Vec<usize> = (0..18)
            .filter(|&m| {
                let f = m / 3;
                let t = m % 3;
                // U(0), D(3) は全回転、他は半回転のみ
                (f == 0 || f == 3) || t == 1
            })
            .collect();

        for depth in 1..=14 {
            let mut path = Vec::new();
            if self.search_g3(cube, depth, 99, &g2_moves, &mut path) {
                return Ok(path);
            }
            if self.exhausted() {
                break;
            }
        }
        Err("Phase 3の探索に失敗しました。".into())
    }

    fn search_g3(
        &mut self,
        c: &RawCube,
        depth: u8,
        last_face: usize,
        allowed: &[usize],
        path: &mut Vec<usize>,
    ) -> bool {
        self.nodes += 1;
        if self.exhausted() {
            return false;
        }
        if depth == 0 {
            return self.is_g3(c);
        }

        for &m in allowed {
            let face = m / 3;
            if redundant(face, last_face) {
                continue;
            }
            let next = c.multiply(move_cube_18(m));
            path.push(m);
            if self.search_g3(&next, depth - 1, face, allowed, path) {
                return true;
            }
            path.pop();
        }
        false
    }

    // Phase 4: G3 -> G4 (許可手: U2, D2, L2, R2, F2, B2)
    fn solve_phase4(&mut self, cube: &RawCube) -> Result<Vec<usize>, String> {
        if *cube == RawCube::default() {
            return Ok(Vec::new());
        }
        let g3_moves: Vec<usize> = (0..6).map(|f| f * 3 + 1).collect(); // すべての面の半回転 (2)

        for depth in 1..=15 {
            let mut path = Vec::new();
            if self.search_g4(cube, depth, 99, &g3_moves, &mut path) {
                return Ok(path);
            }
            if self.exhausted() {
                break;
            }
        }
        Err("Phase 4の探索に失敗しました。".into())
    }

    fn search_g4(
        &mut self,
        c: &RawCube,
        depth: u8,
        last_face: usize,
        allowed: &[usize],
        path: &mut Vec<usize>,
    ) -> bool {
        self.nodes += 1;
        if self.exhausted() {
            return false;
        }
        if depth == 0 {
            return *c == RawCube::default();
        }
        let bad_cp = (0..8).filter(|&i| c.cp[i] as usize != i).count() as u8;
        let bad_ep = (0..12).filter(|&i| c.ep[i] as usize != i).count() as u8;
        if bad_cp.div_ceil(4) > depth || bad_ep.div_ceil(4) > depth {
            return false;
        }

        for &m in allowed {
            let face = m / 3;
            if redundant(face, last_face) {
                continue;
            }
            let next = c.multiply(move_cube_18(m));
            path.push(m);
            if self.search_g4(&next, depth - 1, face, allowed, path) {
                return true;
            }
            path.pop();
        }
        false
    }
}

fn redundant(face: usize, last: usize) -> bool {
    face == last || ((3..6).contains(&last) && face + 3 == last)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cube::{apply, parse_moves};

    #[test]
    fn test_thistlethwaite_solved() {
        let cube = RawCube::default();
        let mut th = ThistlethwaiteSearch::new(1000);
        let res = th.solve(&cube).unwrap();
        assert!(res.moves.is_empty());
    }

    #[test]
    fn test_thistlethwaite_short_scramble() {
        let cube = RawCube::default();
        let sc = parse_moves("R2 U2 F2 D2").unwrap();
        let scrambled = apply(&cube, &sc);
        let mut th = ThistlethwaiteSearch::new(3000);
        let res = th.solve(&scrambled).unwrap();
        let final_cube = apply(&scrambled, &res.moves);
        assert_eq!(final_cube, RawCube::default());
    }

    #[test]
    fn test_thistlethwaite_phases_1_and_2() {
        // F や R を含めることで Phase 1（EO）と Phase 2（CO+E）の探索を網羅
        let cube = RawCube::default();
        let sc = parse_moves("F R U").unwrap();
        let scrambled = apply(&cube, &sc);
        let mut th = ThistlethwaiteSearch::new(5000);
        let res = th.solve(&scrambled).unwrap();
        let final_cube = apply(&scrambled, &res.moves);
        assert_eq!(final_cube, RawCube::default());
    }

    #[test]
    fn test_thistlethwaite_direct_phases_on_solved() {
        let cube = RawCube::default();
        let mut th = ThistlethwaiteSearch::new(1000);
        assert!(th.solve_phase1(&cube).unwrap().is_empty());
        assert!(th.solve_phase2(&cube).unwrap().is_empty());
        assert!(th.solve_phase3(&cube).unwrap().is_empty());
        assert!(th.solve_phase4(&cube).unwrap().is_empty());

        // タイムアウト / exhausted
        let mut th2 = ThistlethwaiteSearch::new(0);
        th2.start = Instant::now() - std::time::Duration::from_millis(10);
        th2.nodes = 4096;
        assert!(th2.exhausted());
        assert!(th2.exhausted()); // 2回目 (timed_out == true)

        // タイムアウト時の各Phaseのエラー
        let scrambled = apply(&cube, &parse_moves("F").unwrap());
        let mut th_err = ThistlethwaiteSearch::new(0);
        th_err.timed_out = true;
        assert!(th_err.solve_phase1(&scrambled).is_err());
        assert!(th_err.solve_phase2(&scrambled).is_err());
        assert!(th_err.solve_phase3(&scrambled).is_err());
        assert!(th_err.solve_phase4(&scrambled).is_err());

        // is_g3 の非g2ケース
        assert!(!th.is_g3(&scrambled));
    }

    #[test]
    fn test_thistlethwaite_phase2_explicit() {
        let cube = RawCube::default();
        // U R U' R' は G1（エッジ反転なし）だが G2 ではない（コーナー向き崩れ）
        let sc = parse_moves("U R U' R'").unwrap();
        let g1_cube = apply(&cube, &sc);
        let mut th = ThistlethwaiteSearch::new(3000);
        assert!(th.is_g1(&g1_cube));
        assert!(!th.is_g2(&g1_cube));
        let p2 = th.solve_phase2(&g1_cube).unwrap();
        assert!(!p2.is_empty());
        let g2_cube = apply(&g1_cube, &p2);
        assert!(th.is_g2(&g2_cube));
    }

    #[test]
    fn test_thistlethwaite_is_g3_not_ud_edges() {
        let mut cube = RawCube::default();
        cube.ep[0] = cube.ep[8]; // Eスライスエッジを0スロットにも配置
        let th = ThistlethwaiteSearch::new(1000);
        assert!(!th.is_g3(&cube));
    }

    #[test]
    fn test_thistlethwaite_superflip_solves_under_45_moves() {
        let preset: serde_json::Value =
            serde_json::from_str(include_str!("../cubes/superflip.json")).unwrap();
        let sc = parse_moves(preset["scramble"].as_str().unwrap()).unwrap();
        let cube = apply(&RawCube::default(), &sc);
        let budget = 60_000;
        let mut th = ThistlethwaiteSearch::new(budget);

        let res = th.solve(&cube).unwrap();
        assert_eq!(res.phases.len(), 4, "All 4 phases must succeed");
        assert!(res.moves.len() >= 20);
        assert!(res.moves.len() <= 45);
        let final_cube = apply(&cube, &res.moves);
        assert_eq!(final_cube, RawCube::default(), "Cube must be fully solved");
    }

    #[test]
    fn test_thistlethwaite_fallback_solve() {
        let cube = RawCube::default();
        let sc = parse_moves("R U F").unwrap();
        let scrambled = apply(&cube, &sc);
        let mut th = ThistlethwaiteSearch::new(10_000);
        let res = th.fallback_solve(&scrambled, Vec::new()).unwrap();
        assert!(!res.moves.is_empty());
        assert_eq!(apply(&scrambled, &res.moves), RawCube::default());

        // 既存フェーズを引き継いだフォールバック
        let p1 = th.solve_phase1(&scrambled).unwrap();
        let c1 = apply(&scrambled, &p1);
        let mut th2 = ThistlethwaiteSearch::new(10_000);
        let res2 = th2
            .fallback_solve(
                &c1,
                vec![ThistlePhase {
                    name: "Phase 1",
                    moves: p1,
                }],
            )
            .unwrap();
        assert!(res2.phases.len() >= 2);

        // 予算ゼロでのエラー
        let mut th_zero = ThistlethwaiteSearch::new(0);
        assert!(th_zero.fallback_solve(&scrambled, Vec::new()).is_err());
    }

    #[test]
    fn test_thistlethwaite_phase_fallbacks() {
        // G1状態のキューブ (eo=0だがco!=0): F/Bの90度回転を含まないスクランブル
        let g1_cube = apply(&RawCube::default(), &parse_moves("R U R' U'").unwrap());
        let mut th_p2 = ThistlethwaiteSearch::new(10_000);
        th_p2.main_budget_ms = 0;
        th_p2.timed_out = true;
        let res_p2 = th_p2.solve(&g1_cube).unwrap();
        assert_eq!(apply(&g1_cube, &res_p2.moves), RawCube::default());

        // G2状態のキューブ (eo=0, co=0, Eスライスエッジ保持): U, D, R2, L2, F2, B2
        let g2_cube = apply(&RawCube::default(), &parse_moves("U D R2 L2").unwrap());
        let mut th_p3 = ThistlethwaiteSearch::new(10_000);
        th_p3.main_budget_ms = 0;
        th_p3.timed_out = true;
        let res_p3 = th_p3.solve(&g2_cube).unwrap();
        assert_eq!(apply(&g2_cube, &res_p3.moves), RawCube::default());

        // G3状態のキューブ (全手180度回転): U2, D2, F2, B2, L2, R2
        let g3_cube = apply(&RawCube::default(), &parse_moves("U2 D2 R2 L2").unwrap());
        let mut th_p4 = ThistlethwaiteSearch::new(10_000);
        th_p4.main_budget_ms = 0;
        th_p4.timed_out = true;
        let res_p4 = th_p4.solve(&g3_cube).unwrap();
        assert_eq!(apply(&g3_cube, &res_p4.moves), RawCube::default());
    }
}
