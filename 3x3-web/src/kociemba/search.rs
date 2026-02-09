use super::coord::{move_cube_18, RawCube};
use super::tables::{MoveTable, PruningTable};
use crate::cube::Move;

pub struct Search {
    move_table: &'static MoveTable,
    pruning_table: &'static PruningTable,
    initial_cube: RawCube,
    phase1_moves: Vec<usize>,
    phase2_moves: Vec<usize>,
    pub min_total_length: usize,
    pub solution: Option<Vec<Move>>,
    phase1_solutions_found: usize,
    pub node_count: usize,
}

const MAX_NODES: usize = 20_000_000;

impl Default for Search {
    fn default() -> Self {
        Self::new()
    }
}

impl Search {
    pub fn new() -> Self {
        Self {
            move_table: MoveTable::get(),
            pruning_table: PruningTable::get(),
            initial_cube: RawCube::default(),
            phase1_moves: Vec::with_capacity(32),
            phase2_moves: Vec::with_capacity(32),
            min_total_length: 99,
            solution: None,
            phase1_solutions_found: 0,
            node_count: 0,
        }
    }

    pub fn solve(&mut self, rc: &RawCube, max_depth: usize) -> Option<Vec<Move>> {
        self.solution = None;
        self.phase1_moves.clear();
        self.phase2_moves.clear();
        self.min_total_length = max_depth + 1;
        self.initial_cube = *rc;
        self.phase1_solutions_found = 0;
        self.node_count = 0;

        let twist = rc.get_twist();
        let flip = rc.get_flip();
        let slice = rc.get_ud_slice();

        for depth in 0..=max_depth {
            if self.search_phase1(twist, flip, slice, depth as u8, 99) {
                break;
            }
            if self.node_count > MAX_NODES {
                break;
            }
        }
        if self.solution.is_none() && self.node_count > MAX_NODES {
            println!("Search hit node limit: {}", self.node_count);
        }
        self.solution.clone()
    }

    pub fn search_phase1(
        &mut self,
        twist: u16,
        flip: u16,
        slice: u16,
        depth: u8,
        last_face: usize,
    ) -> bool {
        if depth == 0 {
            if twist == 0 && flip == 0 && slice == 0 {
                return self.init_phase2();
            }
            return false;
        }

        // 枝刈り
        self.node_count += 1;
        if self.node_count > MAX_NODES {
            return false;
        }

        let dist = self.get_phase1_dist(twist, flip, slice);
        if dist > depth {
            return false;
        }

        for m in 0..6 {
            if m == last_face || is_redundant(m, last_face) {
                continue;
            }
            for r in 0..3 {
                let mv_idx = m * 3 + r;
                let next_twist = self.move_table.twist[twist as usize][mv_idx];
                let next_flip = self.move_table.flip[flip as usize][mv_idx];
                let next_slice = self.move_table.ud_slice[slice as usize][mv_idx];

                self.phase1_moves.push(mv_idx);
                if self.search_phase1(next_twist, next_flip, next_slice, depth - 1, m) {
                    return true;
                }
                self.phase1_moves.pop();
            }
        }
        false
    }

    fn get_phase1_dist(&self, twist: u16, flip: u16, slice: u16) -> u8 {
        let d1 = self.pruning_table.twist_slice[twist as usize * 495 + slice as usize];
        let d2 = self.pruning_table.flip_slice[flip as usize * 495 + slice as usize];
        d1.max(d2)
    }

    fn init_phase2(&mut self) -> bool {
        self.phase2_moves.clear();
        // Phase 1 の解を適用して正確な RawCube を取得
        let mut rc = self.initial_cube;
        for &m_idx in &self.phase1_moves {
            rc = rc.multiply(move_cube_18(m_idx));
        }

        let cp = rc.get_cp();
        let ep8 = rc.get_ep8();
        let slice_p = rc.get_slice_p();

        let p1_len = self.phase1_moves.len();
        let mut found_any = false;
        // 現在の最善解より短いもののみ探す。ただし Phase 2 が深すぎると探索が終わらないため、
        // 12手程度で打ち切るのが Kociemba の一般的実装。
        let max_p2_d = (self
            .min_total_length
            .saturating_sub(p1_len)
            .saturating_sub(1))
        .min(18);
        for d in 0..=max_p2_d {
            if self.search_phase2(cp, ep8, slice_p, d as u8, 99) {
                self.phase1_solutions_found += 1;
                found_any = true;
                break;
            }
        }
        found_any
    }

    fn search_phase2(
        &mut self,
        cp: u16,
        ep8: u16,
        slice_p: u16,
        depth: u8,
        last_face: usize,
    ) -> bool {
        if depth == 0 {
            if cp == 0 && ep8 == 0 && slice_p == 0 {
                // 解が見つかった
                self.extract_solution();
                return true;
            }
            return false;
        }

        // 枝刈り
        self.node_count += 1;
        if self.node_count > MAX_NODES {
            return false;
        }

        let dist = self.get_phase2_dist(cp, ep8, slice_p);
        if dist > depth {
            return false;
        }

        // U(0-2), R2(4), F2(7), D(9-11), L2(13), B2(16)
        let allowed_p2_moves = [0, 1, 2, 4, 7, 9, 10, 11, 13, 16];
        for &mv_idx in &allowed_p2_moves {
            let m = mv_idx / 3;
            if m == last_face || is_redundant(m, last_face) {
                continue;
            }

            let next_cp = self.move_table.cp[cp as usize][mv_idx];
            let next_ep8 = self.move_table.ep8[ep8 as usize][mv_idx];
            let next_slice_p = self.move_table.slice_p[slice_p as usize][mv_idx];

            self.phase2_moves.push(mv_idx);
            if self.search_phase2(next_cp, next_ep8, next_slice_p, depth - 1, m) {
                return true;
            }
            self.phase2_moves.pop();
        }
        false
    }

    fn get_phase2_dist(&self, cp: u16, ep8: u16, slice_p: u16) -> u8 {
        let d1 = self.pruning_table.cp_slice[cp as usize * 24 + slice_p as usize];
        let d2 = self.pruning_table.ep8_slice[ep8 as usize * 24 + slice_p as usize];
        d1.max(d2)
    }

    fn extract_solution(&mut self) {
        let mut sol = Vec::new();
        for &m_idx in &self.phase1_moves {
            sol.push(idx_to_move(m_idx));
        }
        for &m_idx in &self.phase2_moves {
            sol.push(idx_to_move(m_idx));
        }
        self.min_total_length = sol.len();
        self.solution = Some(sol);
    }
}

pub fn is_redundant(m: usize, last_m: usize) -> bool {
    // 同じ面は除外 (last_face と比較済み)
    // 対向面の重複 (U-D, R-L, F-B) を防ぐ。Uの後にDはOKだが、Dの後にUはインデックス順で制限
    if last_m == 99 {
        return false;
    }
    match (last_m, m) {
        (0, 3) => false,
        (3, 0) => true,
        (1, 4) => false,
        (4, 1) => true,
        (2, 5) => false,
        (5, 2) => true,
        _ => false,
    }
}

pub fn idx_to_move(idx: usize) -> Move {
    let m = idx / 3;
    let r = idx % 3;
    match (m, r) {
        (0, 0) => Move::U,
        (0, 1) => Move::U2,
        (0, 2) => Move::Up,
        (1, 0) => Move::R,
        (1, 1) => Move::R2,
        (1, 2) => Move::Rp,
        (2, 0) => Move::F,
        (2, 1) => Move::F2,
        (2, 2) => Move::Fp,
        (3, 0) => Move::D,
        (3, 1) => Move::D2,
        (3, 2) => Move::Dp,
        (4, 0) => Move::L,
        (4, 1) => Move::L2,
        (4, 2) => Move::Lp,
        (5, 0) => Move::B,
        (5, 1) => Move::B2,
        (5, 2) => Move::Bp,
        _ => unreachable!(),
    }
}
