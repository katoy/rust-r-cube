use super::coord::RawCube;
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
        }
    }

    pub fn solve(&mut self, rc: &RawCube) -> Option<Vec<Move>> {
        self.solution = None;
        self.min_total_length = 32;
        self.initial_cube = rc.clone();

        let twist = rc.get_twist();
        let flip = rc.get_flip();
        let slice = rc.get_ud_slice();

        for depth in 0..=31 {
            println!("  Phase 1 search: depth {}", depth);
            if self.search_phase1(twist, flip, slice, depth, 99) {
                break;
            }
        }
        self.solution.clone()
    }

    fn search_phase1(
        &mut self,
        twist: u16,
        flip: u16,
        slice: u16,
        depth: u8,
        last_face: usize,
    ) -> bool {
        if depth == 0 {
            if twist == 0 && flip == 0 && slice == 0 {
                // Phase 1 完了 -> Phase 2 準備
                return self.init_phase2();
            }
            return false;
        }

        // 枝刈り
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
        // Phase 1 の解を適用して正確な RawCube を取得
        let mut rc = self.initial_cube.clone();
        for &m_idx in &self.phase1_moves {
            let m = m_idx / 3;
            let r = m_idx % 3;
            for _ in 0..=r {
                rc = rc.multiply(RawCube::move_cube(m));
            }
        }

        let cp = rc.get_cp();
        let ep8 = rc.get_ep8();
        let slice_p = rc.get_slice_p();

        // Phase 2 の IDA*
        println!(
            "    Phase 1 found! length: {}. Starting Phase 2 search...",
            self.phase1_moves.len()
        );
        let p1_len = self.phase1_moves.len();
        for depth in 0..=(self.min_total_length - p1_len - 1) {
            println!("      Phase 2 search: depth {}", depth);
            if self.search_phase2(cp, ep8, slice_p, depth as u8, 99) {
                return true;
            }
        }
        false
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
        let dist = self.get_phase2_dist(cp, ep8, slice_p);
        if dist > depth {
            return false;
        }

        let allowed_faces = [0, 1, 2, 3, 4, 5];
        for &m in &allowed_faces {
            if m == last_face || is_redundant(m, last_face) {
                continue;
            }
            for r in 0..3 {
                // Phase 2 制限: R, L, F, B は 180度 (r=1) のみ
                if (m == 1 || m == 2 || m == 4 || m == 5) && r != 1 {
                    continue;
                }

                let mv_idx = m * 3 + r;
                let next_cp = self.move_table.cp[cp as usize][mv_idx];
                let next_ep8 = self.move_table.ep8[ep8 as usize][mv_idx];
                let next_slice_p = self.move_table.slice_p[slice_p as usize][mv_idx];

                self.phase2_moves.push(mv_idx);
                if self.search_phase2(next_cp, next_ep8, next_slice_p, depth - 1, m) {
                    return true;
                }
                self.phase2_moves.pop();
            }
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

fn is_redundant(m: usize, last_m: usize) -> bool {
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

fn idx_to_move(idx: usize) -> Move {
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
