use crate::coord::{move_cube_18, rank_perm8, unrank_perm8, RawCube};
use crate::cube::apply;
use crate::tables::{MoveTable, PruningTable, PHASE2_MOVES};
use std::collections::VecDeque;
use std::sync::OnceLock;
use web_time::Instant;

/// Phase 2 (G1 -> G2) で許可される 14 通りの基本手: U, D, R, L, F2, B2
const G1_MOVES: [usize; 14] = [0, 1, 2, 3, 4, 5, 7, 9, 10, 11, 12, 13, 14, 16];
/// Phase 4 (G3 -> G4) で許可される 6 通りの半回転（PHASE2_MOVES 内のインデックス: U2, R2, F2, D2, L2, B2）
const G3_P2_INDICES: [usize; 6] = [1, 3, 4, 6, 8, 9];

fn g1_eo_dist() -> &'static [u8; 2048] {
    static G1_EO_DIST: OnceLock<[u8; 2048]> = OnceLock::new();
    G1_EO_DIST.get_or_init(|| {
        let mt = MoveTable::get();
        let mut dist = [255u8; 2048];
        dist[0] = 0;
        let mut q = VecDeque::with_capacity(2048);
        q.push_back(0u16);
        while let Some(f) = q.pop_front() {
            let d = dist[f as usize];
            for &nf in &mt.flip[f as usize] {
                if dist[nf as usize] == 255 {
                    dist[nf as usize] = d + 1;
                    q.push_back(nf);
                }
            }
        }
        dist
    })
}

struct G3Table {
    cp_coset: Box<[u16; 40320]>,
    coset_move: Box<[[u16; 10]; 420]>,
    edge_id: [u8; 256],
    edge_move: [[u8; 10]; 70],
    dist: Box<[u8; 29400]>,
}

fn g3_table() -> &'static G3Table {
    static G3_TABLE: OnceLock<G3Table> = OnceLock::new();
    G3_TABLE.get_or_init(|| {
        let mt = MoveTable::get();
        let h_gens: [[u8; 8]; 6] = std::array::from_fn(|i| {
            let mc = move_cube_18(PHASE2_MOVES[G3_P2_INDICES[i]]);
            mc.cp.map(|c| c as u8)
        });

        // 1. H = <U2, R2, F2, D2, L2, B2> による 40320 個のコーナー置換の 420 右コセット H*c 分割
        let mut cp_coset = Box::new([u16::MAX; 40320]);
        let mut coset_repr = [0u16; 420];
        let mut num_cosets = 0usize;
        let mut q_cp = VecDeque::with_capacity(96);

        for start in 0..40320u16 {
            if cp_coset[start as usize] != u16::MAX {
                continue;
            }
            let cid = num_cosets as u16;
            coset_repr[num_cosets] = start;
            num_cosets += 1;
            cp_coset[start as usize] = cid;
            q_cp.push_back(start);

            while let Some(c) = q_cp.pop_front() {
                let p = unrank_perm8(c);
                for gen in &h_gens {
                    let q = [
                        gen[p[0] as usize],
                        gen[p[1] as usize],
                        gen[p[2] as usize],
                        gen[p[3] as usize],
                        gen[p[4] as usize],
                        gen[p[5] as usize],
                        gen[p[6] as usize],
                        gen[p[7] as usize],
                    ];
                    let nc = rank_perm8(&q);
                    if cp_coset[nc as usize] == u16::MAX {
                        cp_coset[nc as usize] = cid;
                        q_cp.push_back(nc);
                    }
                }
            }
        }
        assert_eq!(num_cosets, 420, "G3 corner cosets must be 420");

        let mut coset_move = Box::new([[0u16; 10]; 420]);
        for (cid, &repr) in coset_repr.iter().enumerate() {
            for k in 0..10 {
                coset_move[cid][k] = cp_coset[mt.cp[repr as usize][k] as usize];
            }
        }

        // 2. 8-bit S/M スライスマスク（popcount == 4、計 70 状態）への遷移表
        let mut edge_id = [255u8; 256];
        let mut id_to_mask = [0u8; 70];
        edge_id[0x55] = 0;
        id_to_mask[0] = 0x55;
        let mut next_eid = 1usize;
        for mask in 0u16..256 {
            if mask != 0x55 && mask.count_ones() == 4 {
                edge_id[mask as usize] = next_eid as u8;
                id_to_mask[next_eid] = mask as u8;
                next_eid += 1;
            }
        }
        assert_eq!(next_eid, 70, "G3 S/M slice states must be 70");

        let ep_moves: [[u8; 8]; 10] = std::array::from_fn(|k| {
            let mc = move_cube_18(PHASE2_MOVES[k]);
            std::array::from_fn(|i| mc.ep[i] as u8)
        });
        let mut edge_move = [[0u8; 10]; 70];
        for (eid, &mask) in id_to_mask.iter().enumerate() {
            for (k, mv) in ep_moves.iter().enumerate() {
                let mut nmask = 0u8;
                for (i, &src) in mv.iter().enumerate() {
                    if ((mask >> src) & 1) != 0 {
                        nmask |= 1u8 << i;
                    }
                }
                edge_move[eid][k] = edge_id[nmask as usize];
            }
        }

        // 3. 420 * 70 = 29,400 状態の厳密最短距離 BFS（G2 -> G3）
        let mut dist = Box::new([255u8; 29400]);
        dist[0] = 0;
        let mut q = VecDeque::with_capacity(29400);
        q.push_back((0u16, 0u8));
        let mut visited = 1usize;
        while let Some((cid, eid)) = q.pop_front() {
            let d = dist[cid as usize * 70 + eid as usize];
            for k in 0..10 {
                let ncid = coset_move[cid as usize][k];
                let neid = edge_move[eid as usize][k];
                let nidx = ncid as usize * 70 + neid as usize;
                if dist[nidx] == 255 {
                    dist[nidx] = d + 1;
                    visited += 1;
                    q.push_back((ncid, neid));
                }
            }
        }
        assert_eq!(visited, 29400, "G2->G3 coset BFS incomplete");

        G3Table {
            cp_coset,
            coset_move,
            edge_id,
            edge_move,
            dist,
        }
    })
}

fn is_valid_g3_corner_permutation(cp: u16) -> bool {
    g3_table().cp_coset[cp as usize] == 0
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
            Err(_) => return self.fallback_solve(&current, all_phases, 1),
        };
        current = apply(&current, &p1);
        all_phases.push(ThistlePhase {
            name: "Phase 1 (G0→G1: エッジ向き)",
            moves: p1,
        });

        // Phase 2: G1 -> G2 (コーナー向き twist = 0 & Eスライスエッジ)
        let p2 = match self.solve_phase2(&current) {
            Ok(p) => p,
            Err(_) => return self.fallback_solve(&current, all_phases, 2),
        };
        current = apply(&current, &p2);
        all_phases.push(ThistlePhase {
            name: "Phase 2 (G1→G2: コーナー向き&Eスライス)",
            moves: p2,
        });

        // Phase 3: G2 -> G3 (ピースのオービット分離 & パリティ)
        let p3 = match self.solve_phase3(&current) {
            Ok(p) => p,
            Err(_) => return self.fallback_solve(&current, all_phases, 3),
        };
        current = apply(&current, &p3);
        all_phases.push(ThistlePhase {
            name: "Phase 3 (G2→G3: オービット分離)",
            moves: p3,
        });

        // Phase 4: G3 -> G4 (完成)
        let p4 = match self.solve_phase4(&current) {
            Ok(p) => p,
            Err(_) => return self.fallback_solve(&current, all_phases, 4),
        };
        let current = apply(&current, &p4);
        if current != RawCube::default() {
            return Err("Thistlethwaite解法の検証に失敗しました。".to_string());
        }
        all_phases.push(ThistlePhase {
            name: "Phase 4 (G3→G4: 最終解決)",
            moves: p4,
        });

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
        phase_num: usize,
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
                name: match phase_num {
                    1 => "Phase 1以降 Kociemba フォールバック",
                    2 => "Phase 2以降 Kociemba フォールバック",
                    3 => "Phase 3以降 Kociemba フォールバック",
                    _ => "Phase 4 (最終解決フォールバック)",
                },
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

    // Phase 1: G0 -> G1 (すべての手OK): flip 表 + BFS 距離による O(d) 貪欲降下
    fn is_g1(&self, c: &RawCube) -> bool {
        c.eo.iter().all(|&o| o == 0)
    }

    fn solve_phase1(&mut self, cube: &RawCube) -> Result<Vec<usize>, String> {
        if self.is_g1(cube) {
            return Ok(Vec::new());
        }
        if self.exhausted() {
            return Err("Phase 1の探索に失敗しました。".into());
        }
        let dist = g1_eo_dist();
        let mt = MoveTable::get();
        let mut flip = cube.get_flip() as usize;
        let mut cur_d = dist[flip];
        let mut last_face = 99usize;
        let mut path = Vec::with_capacity(cur_d as usize);

        while flip != 0 {
            self.nodes += 1;
            if self.exhausted() {
                return Err("Phase 1の探索に失敗しました。".into());
            }
            let mut stepped = false;
            for (m, &nf_u16) in mt.flip[flip].iter().enumerate() {
                let face = m / 3;
                if face == last_face {
                    continue;
                }
                let nf = nf_u16 as usize;
                if dist[nf] < cur_d {
                    path.push(m);
                    flip = nf;
                    cur_d = dist[nf];
                    last_face = face;
                    stepped = true;
                    break;
                }
            }
            if !stepped {
                return Err("Phase 1の探索に失敗しました。".into());
            }
        }
        Ok(path)
    }

    // Phase 2: G1 -> G2 (許可手: U, D, L, R, F2, B2): 座標のみ IDA*
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
        if !self.is_g1(cube) || self.exhausted() {
            return Err("Phase 2の探索に失敗しました。".into());
        }
        let twist = cube.get_twist() as usize;
        let slice = cube.get_ud_slice() as usize;
        let mt = MoveTable::get();
        let pt = PruningTable::get();
        let start_d = pt.get_twist_slice(twist, slice).max(1);

        for depth in start_d..=12 {
            let mut path = Vec::with_capacity(depth as usize);
            if self.search_g2_coords(mt, pt, twist, slice, depth, 99, &mut path) {
                return Ok(path);
            }
            if self.exhausted() {
                break;
            }
        }
        Err("Phase 2の探索に失敗しました。".into())
    }

    #[allow(clippy::too_many_arguments)]
    fn search_g2_coords(
        &mut self,
        mt: &MoveTable,
        pt: &PruningTable,
        twist: usize,
        slice: usize,
        depth: u8,
        last_face: usize,
        path: &mut Vec<usize>,
    ) -> bool {
        self.nodes += 1;
        if self.exhausted() {
            return false;
        }
        if depth == 0 {
            return twist == 0 && slice == 0;
        }
        let next_depth = depth - 1;
        for &m in &G1_MOVES {
            let face = m / 3;
            if redundant(face, last_face) {
                continue;
            }
            let nt = mt.twist[twist][m] as usize;
            let ns = mt.ud_slice[slice][m] as usize;
            if next_depth == 0 {
                if nt != 0 || ns != 0 {
                    continue;
                }
            } else if pt.get_twist_slice(nt, ns) > next_depth {
                continue;
            }
            path.push(m);
            if self.search_g2_coords(mt, pt, nt, ns, next_depth, face, path) {
                return true;
            }
            path.pop();
            if self.timed_out {
                return false;
            }
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
        if !self.is_g2(cube) || self.exhausted() {
            return Err("Phase 3の探索に失敗しました。".into());
        }
        let g3 = g3_table();
        let mut cid = g3.cp_coset[cube.get_cp() as usize];
        let mut mask = 0u8;
        for (i, &ep) in cube.ep.iter().enumerate().take(8) {
            if ((ep as u8) & 1) == 0 {
                mask |= 1u8 << i;
            }
        }
        let mut eid = g3.edge_id[mask as usize];
        if eid >= 70 {
            return Err("Phase 3の探索に失敗しました。".into());
        }
        let mut cur_d = g3.dist[cid as usize * 70 + eid as usize];
        let mut last_face = 99usize;
        let mut path = Vec::with_capacity(cur_d as usize);

        while cid != 0 || eid != 0 {
            self.nodes += 1;
            if self.exhausted() {
                return Err("Phase 3の探索に失敗しました。".into());
            }
            let mut stepped = false;
            for (k, &m) in PHASE2_MOVES.iter().enumerate() {
                let face = m / 3;
                if face == last_face {
                    continue;
                }
                let ncid = g3.coset_move[cid as usize][k];
                let neid = g3.edge_move[eid as usize][k];
                let nd = g3.dist[ncid as usize * 70 + neid as usize];
                if nd < cur_d {
                    path.push(m);
                    cid = ncid;
                    eid = neid;
                    cur_d = nd;
                    last_face = face;
                    stepped = true;
                    break;
                }
            }
            if !stepped {
                return Err("Phase 3の探索に失敗しました。".into());
            }
        }
        Ok(path)
    }

    // Phase 4: G3 -> G4 (許可手: U2, D2, L2, R2, F2, B2): 座標のみ IDA*
    fn solve_phase4(&mut self, cube: &RawCube) -> Result<Vec<usize>, String> {
        if *cube == RawCube::default() {
            return Ok(Vec::new());
        }
        if !self.is_g3(cube) || self.exhausted() {
            return Err("Phase 4の探索に失敗しました。".into());
        }
        let cp = cube.get_cp() as usize;
        let ep = cube.get_ep8() as usize;
        let sp = cube.get_slice_p() as usize;
        let mt = MoveTable::get();
        let pt = PruningTable::get();
        let start_d = pt.get_cp_slice(cp, sp).max(pt.get_ep8_slice(ep, sp)).max(1);

        for depth in start_d..=15 {
            let mut path = Vec::with_capacity(depth as usize);
            if self.search_g4_coords(mt, pt, cp, ep, sp, depth, 99, &mut path) {
                return Ok(path);
            }
            if self.exhausted() {
                break;
            }
        }
        Err("Phase 4の探索に失敗しました。".into())
    }

    #[allow(clippy::too_many_arguments)]
    fn search_g4_coords(
        &mut self,
        mt: &MoveTable,
        pt: &PruningTable,
        cp: usize,
        ep: usize,
        sp: usize,
        depth: u8,
        last_face: usize,
        path: &mut Vec<usize>,
    ) -> bool {
        self.nodes += 1;
        if self.exhausted() {
            return false;
        }
        if depth == 0 {
            return cp == 0 && ep == 0 && sp == 0;
        }
        let next_depth = depth - 1;
        for &p2_idx in &G3_P2_INDICES {
            let m = PHASE2_MOVES[p2_idx];
            let face = m / 3;
            if redundant(face, last_face) {
                continue;
            }
            let ncp = mt.cp[cp][p2_idx] as usize;
            let nsp = mt.slice_p[sp][p2_idx] as usize;
            let nep;
            if next_depth == 0 {
                if ncp != 0 || nsp != 0 {
                    continue;
                }
                nep = mt.ep8[ep][p2_idx] as usize;
                if nep != 0 {
                    continue;
                }
            } else {
                if pt.get_cp_slice(ncp, nsp) > next_depth {
                    continue;
                }
                nep = mt.ep8[ep][p2_idx] as usize;
                if pt.get_ep8_slice(nep, nsp) > next_depth {
                    continue;
                }
            }
            path.push(m);
            if self.search_g4_coords(mt, pt, ncp, nep, nsp, next_depth, face, path) {
                return true;
            }
            path.pop();
            if self.timed_out {
                return false;
            }
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
    use crate::cube::{apply, parse_moves, scramble};

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
            serde_json::from_str(include_str!("../../cubes/superflip.json")).unwrap();
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
    fn test_thistlethwaite_20_random_seeds_no_fallback() {
        for seed in 1..=20 {
            let cube = apply(&RawCube::default(), &scramble(seed));
            let mut th = ThistlethwaiteSearch::new(3000);
            let res = th
                .solve(&cube)
                .unwrap_or_else(|e| panic!("seed {seed} failed: {e}"));
            assert_eq!(res.phases.len(), 4, "seed {seed} phases != 4");
            for p in &res.phases {
                assert!(
                    !p.name.contains("フォールバック"),
                    "seed {seed} used fallback: {}",
                    p.name
                );
            }
            assert_eq!(apply(&cube, &res.moves), RawCube::default());
        }
    }

    #[test]
    fn test_thistlethwaite_fallback_solve() {
        let cube = RawCube::default();
        let sc = parse_moves("R U F").unwrap();
        let scrambled = apply(&cube, &sc);
        let mut th = ThistlethwaiteSearch::new(10_000);
        let res = th.fallback_solve(&scrambled, Vec::new(), 4).unwrap();
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
                2,
            )
            .unwrap();
        assert!(res2.phases.len() >= 2);

        // 予算ゼロでのエラー
        let mut th_zero = ThistlethwaiteSearch::new(0);
        assert!(th_zero.fallback_solve(&scrambled, Vec::new(), 4).is_err());
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
