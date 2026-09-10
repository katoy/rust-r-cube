use crate::coord::{move_cube_18, RawCube};
use crate::tables::{MoveTable, PruningTable};
use web_time::Instant;

pub struct Search {
    mt: &'static MoveTable,
    pt: &'static PruningTable,
    start: Instant,
    budget_ms: f64,
    pub nodes: u64,
    pub timed_out: bool,
    path: Vec<usize>,
    initial: RawCube,
    max_total: usize,
}
impl Search {
    pub fn new(budget_ms: u32) -> Self {
        Self {
            mt: MoveTable::get(),
            pt: PruningTable::get(),
            start: Instant::now(),
            budget_ms: f64::from(budget_ms),
            nodes: 0,
            timed_out: false,
            path: Vec::with_capacity(32),
            initial: RawCube::default(),
            max_total: 22,
        }
    }
    pub fn solve(&mut self, cube: &RawCube) -> Option<Vec<usize>> {
        if *cube == RawCube::default() {
            return Some(Vec::new());
        }
        self.initial = *cube;
        for total in [22, 24, 30] {
            self.max_total = total;
            for depth in 0..=12 {
                self.path.clear();
                if self.phase1(
                    cube.get_twist(),
                    cube.get_flip(),
                    cube.get_ud_slice(),
                    depth,
                    99,
                ) {
                    return Some(self.path.clone());
                }
                if self.timed_out {
                    return None;
                }
            }
        }
        None
    }
    fn exhausted(&mut self) -> bool {
        self.nodes += 1;
        if self.nodes & 1023 == 0 && self.start.elapsed().as_secs_f64() * 1000.0 >= self.budget_ms {
            self.timed_out = true;
        }
        self.timed_out
    }
    fn phase1(&mut self, twist: u16, flip: u16, slice: u16, depth: u8, last: usize) -> bool {
        if self.exhausted() {
            return false;
        }
        let distance = self
            .pt
            .get_twist_slice(twist as usize, slice as usize)
            .max(self.pt.get_flip_slice(flip as usize, slice as usize));
        if distance > depth {
            return false;
        }
        if depth == 0 {
            let cube = self
                .path
                .iter()
                .fold(self.initial, |c, m| c.multiply(move_cube_18(*m)));
            let max =
                (self.max_total - self.path.len()).min(if self.max_total == 30 { 18 } else { 12 });
            for d in 0..=max {
                if self.phase2(
                    cube.get_cp(),
                    cube.get_ep8(),
                    cube.get_slice_p(),
                    d as u8,
                    last,
                ) {
                    return true;
                }
                if self.timed_out {
                    return false;
                }
            }
            return false;
        }
        for face in 0..6 {
            if redundant(face, last) {
                continue;
            }
            for turn in 0..3 {
                let m = face * 3 + turn;
                self.path.push(m);
                if self.phase1(
                    self.mt.twist[twist as usize][m],
                    self.mt.flip[flip as usize][m],
                    self.mt.ud_slice[slice as usize][m],
                    depth - 1,
                    face,
                ) {
                    return true;
                }
                self.path.pop();
                if self.timed_out {
                    return false;
                }
            }
        }
        false
    }
    fn phase2(&mut self, cp: u16, ep: u16, sp: u16, depth: u8, last: usize) -> bool {
        if self.exhausted() {
            return false;
        }
        let distance = self.pt.cp_slice[cp as usize * 24 + sp as usize]
            .max(self.pt.ep8_slice[ep as usize * 24 + sp as usize]);
        if distance > depth {
            return false;
        }
        if depth == 0 {
            return cp == 0 && ep == 0 && sp == 0;
        }
        for m in [0, 1, 2, 4, 7, 9, 10, 11, 13, 16] {
            if redundant(m / 3, last) {
                continue;
            }
            self.path.push(m);
            if self.phase2(
                self.mt.cp[cp as usize][m],
                self.mt.ep8[ep as usize][m],
                self.mt.slice_p[sp as usize][m],
                depth - 1,
                m / 3,
            ) {
                return true;
            }
            self.path.pop();
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
