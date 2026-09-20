use crate::coord::{move_cube_18, RawCube};
use crate::cube::apply;
use web_time::Instant;

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
    budget_ms: f64,
    pub nodes: u64,
    pub timed_out: bool,
}

impl ThistlethwaiteSearch {
    pub fn new(budget_ms: u32) -> Self {
        Self {
            start: Instant::now(),
            budget_ms: f64::from(budget_ms),
            nodes: 0,
            timed_out: false,
        }
    }

    fn exhausted(&mut self) -> bool {
        if self.timed_out {
            return true;
        }
        if self.nodes & 4095 == 0 && self.start.elapsed().as_secs_f64() * 1000.0 > self.budget_ms {
            self.timed_out = true;
            return true;
        }
        false
    }

    pub fn solve(&mut self, cube: &RawCube) -> Result<ThistleResult, String> {
        let mut current = *cube;
        let mut all_phases = Vec::new();

        // Phase 1: G0 -> G1 (全エッジの向き flip = 0)
        let p1 = self.solve_phase1(&current)?;
        current = apply(&current, &p1);
        all_phases.push(ThistlePhase {
            name: "Phase 1 (G0→G1: エッジ向き)",
            moves: p1,
        });

        // Phase 2: G1 -> G2 (コーナー向き twist = 0 & Eスライスエッジ)
        let p2 = self.solve_phase2(&current)?;
        current = apply(&current, &p2);
        all_phases.push(ThistlePhase {
            name: "Phase 2 (G1→G2: コーナー向き&Eスライス)",
            moves: p2,
        });

        // Phase 3: G2 -> G3 (ピースのオービット分離 & パリティ)
        let p3 = self.solve_phase3(&current)?;
        current = apply(&current, &p3);
        all_phases.push(ThistlePhase {
            name: "Phase 3 (G2→G3: オービット分離)",
            moves: p3,
        });

        // Phase 4: G3 -> G4 (完成)
        let p4 = self.solve_phase4(&current)?;
        current = apply(&current, &p4);
        all_phases.push(ThistlePhase {
            name: "Phase 4 (G3→G4: 最終解決)",
            moves: p4,
        });

        if current != RawCube::default() {
            return Err("Thistlethwaite解法の検証に失敗しました。".into());
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

    fn search_g1(&mut self, c: &RawCube, depth: u8, last_face: usize, path: &mut Vec<usize>) -> bool {
        self.nodes += 1;
        if self.exhausted() {
            return false;
        }
        if depth == 0 {
            return self.is_g1(c);
        }
        let bad_eo = c.eo.iter().filter(|&&o| o != 0).count() as u8;
        if (bad_eo + 3) / 4 > depth {
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
                p >= 8 && p <= 11
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
        if (bad_co + 3) / 4 > depth || (bad_eslice + 3) / 4 > depth {
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
    fn is_g3(&self, c: &RawCube) -> bool {
        if !self.is_g2(c) {
            return false;
        }
        // U/DエッジがU/D層に留まっていること
        let ud_edges = (0..8).all(|i| (c.ep[i] as usize) < 8);
        if !ud_edges {
            return false;
        }
        // コーナーのオービット判定 (0..4 は U層、4..8 は D層またはテトラヘドロン)
        // 簡易判定: コーナーの置換パリティとエッジの置換パリティが各オービット内で整致
        (0..8).all(|i| (c.cp[i] as usize / 4) == (i / 4))
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
        if (bad_cp + 3) / 4 > depth || (bad_ep + 3) / 4 > depth {
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
}


