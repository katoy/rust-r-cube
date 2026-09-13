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
    pub path: Vec<usize>,
    initial: RawCube,
    max_total: usize,
    target_centers: Option<[i32; 6]>,
    current_centers: [i32; 6],
    best_solution: Option<Vec<usize>>,
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
            target_centers: None,
            current_centers: [0; 6],
            best_solution: None,
        }
    }
    pub fn with_target_centers(mut self, centers: [i32; 6]) -> Self {
        let mut c = [0i32; 6];
        for i in 0..6 {
            c[i] = centers[i].rem_euclid(4);
        }
        self.target_centers = Some(c);
        self.current_centers = c;
        self
    }
    pub fn solve(&mut self, cube: &RawCube) -> Option<Vec<usize>> {
        if *cube == RawCube::default() {
            if let Some(target) = self.target_centers {
                if target.iter().all(|&c| c == 0) {
                    return Some(Vec::new());
                }
            } else {
                return Some(Vec::new());
            }
        }
        self.initial = *cube;
        self.best_solution = None;

        // 短い手数（深さ 1..=5）の直接探索（理論的最短手数を瞬時に見つける）
        for d in 1..=5 {
            self.path.clear();
            if let Some(target) = self.target_centers {
                self.current_centers = target;
            }
            if self.direct_solve(cube, d, 99) {
                return Some(self.path.clone());
            }
            if self.timed_out {
                return None;
            }
        }

        for total in [22, 24, 30] {
            if let Some(ref best) = self.best_solution {
                if best.len() <= total {
                    return self.best_solution.clone();
                }
            }
            self.max_total = self
                .best_solution
                .as_ref()
                .map_or(total, |b| (b.len().saturating_sub(1)).min(total));

            for depth in 0..=12 {
                if depth > self.max_total {
                    break;
                }
                self.path.clear();
                if let Some(target) = self.target_centers {
                    self.current_centers = target;
                }
                if self.phase1(
                    cube.get_twist(),
                    cube.get_flip(),
                    cube.get_ud_slice(),
                    depth as u8,
                    99,
                ) {
                    // 解が見つかった場合、max_total はすでに短縮されている。
                    // 非常に短い解（<= 5手）なら直ちに最善解として終了してよい
                    if let Some(ref b) = self.best_solution {
                        if b.len() <= depth || b.len() <= 5 {
                            return self.best_solution.clone();
                        }
                    }
                }
                if self.timed_out {
                    return self.best_solution.clone();
                }
            }
            if self.best_solution.is_some() {
                return self.best_solution.clone();
            }
        }
        self.best_solution.clone()
    }
    pub fn direct_solve(&mut self, cube: &RawCube, depth: u8, last: usize) -> bool {
        if self.exhausted() {
            return false;
        }
        if depth == 0 {
            if *cube != RawCube::default() {
                return false;
            }
            if self.target_centers.is_some() && !self.current_centers.iter().all(|&c| c == 0) {
                return false;
            }
            return true;
        }
        if self.target_centers.is_some() {
            let non_zero_centers = self.current_centers.iter().filter(|&&c| c != 0).count() as u8;
            if non_zero_centers > depth {
                return false;
            }
        }
        for face in 0..6 {
            if redundant(face, last) {
                continue;
            }
            for turn in 0..3 {
                let m = face * 3 + turn;
                let t = match turn {
                    0 => 1,
                    1 => 2,
                    _ => 3,
                };
                self.path.push(m);
                self.current_centers[face] = (self.current_centers[face] + t) & 3;
                let next_cube = cube.multiply(move_cube_18(m));
                if self.direct_solve(&next_cube, depth - 1, face) {
                    return true;
                }
                self.current_centers[face] = (self.current_centers[face] + 4 - t) & 3;
                self.path.pop();
                if self.timed_out {
                    return false;
                }
            }
        }
        false
    }
    fn exhausted(&mut self) -> bool {
        self.nodes += 1;
        if self.nodes & 1023 == 0 && self.start.elapsed().as_secs_f64() * 1000.0 >= self.budget_ms {
            self.timed_out = true;
        }
        self.timed_out
    }
    fn min_phase2_center_moves(&self) -> u8 {
        let mut count = 0u8;
        if self.current_centers[0] != 0 {
            count += 1;
        }
        if self.current_centers[1] == 2 {
            count += 1;
        }
        if self.current_centers[2] == 2 {
            count += 1;
        }
        if self.current_centers[3] != 0 {
            count += 1;
        }
        if self.current_centers[4] == 2 {
            count += 1;
        }
        if self.current_centers[5] == 2 {
            count += 1;
        }
        count
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
            if self.path.len() > self.max_total {
                return false;
            }
            if self.target_centers.is_some() {
                // In Phase 2, R(1), F(2), L(4), B(5) can only turn by 180° (+2 mod 4).
                // If any of their remaining center rotations is odd, it can never be solved to 0 mod 4.
                if (self.current_centers[1] & 1) != 0
                    || (self.current_centers[2] & 1) != 0
                    || (self.current_centers[4] & 1) != 0
                    || (self.current_centers[5] & 1) != 0
                {
                    return false;
                }
            }
            let cube = self
                .path
                .iter()
                .fold(self.initial, |c, m| c.multiply(move_cube_18(*m)));
            let max =
                (self.max_total - self.path.len()).min(if self.max_total == 30 { 18 } else { 12 });
            let min_d = if self.target_centers.is_some() {
                self.min_phase2_center_moves() as usize
            } else {
                0
            };
            let mut found = false;
            for d in min_d..=max {
                if self.path.len() + d > self.max_total {
                    break;
                }
                if self.phase2(
                    cube.get_cp(),
                    cube.get_ep8(),
                    cube.get_slice_p(),
                    d as u8,
                    last,
                ) {
                    found = true;
                    // 見つかった解で max_total が縮小されたので、これ以上大きい d は探索不要
                    break;
                }
                if self.timed_out {
                    return false;
                }
            }
            return found;
        }
        for face in 0..6 {
            if redundant(face, last) {
                continue;
            }
            for turn in 0..3 {
                let m = face * 3 + turn;
                let t = match turn {
                    0 => 1,
                    1 => 2,
                    _ => 3,
                };
                self.path.push(m);
                self.current_centers[face] = (self.current_centers[face] + t) & 3;
                if self.phase1(
                    self.mt.twist[twist as usize][m],
                    self.mt.flip[flip as usize][m],
                    self.mt.ud_slice[slice as usize][m],
                    depth - 1,
                    face,
                ) {
                    return true;
                }
                self.current_centers[face] = (self.current_centers[face] + 4 - t) & 3;
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
        if self.target_centers.is_some() {
            if (self.current_centers[1] & 1) != 0
                || (self.current_centers[2] & 1) != 0
                || (self.current_centers[4] & 1) != 0
                || (self.current_centers[5] & 1) != 0
            {
                return false;
            }
            if self.min_phase2_center_moves() > depth {
                return false;
            }
        }
        if depth == 0 {
            if cp != 0 || ep != 0 || sp != 0 {
                return false;
            }
            if self.target_centers.is_some() && !self.current_centers.iter().all(|&c| c == 0) {
                return false;
            }
            self.best_solution = Some(self.path.clone());
            self.max_total = self.path.len().saturating_sub(1);
            return true;
        }
        for m in [0, 1, 2, 4, 7, 9, 10, 11, 13, 16] {
            let face = m / 3;
            if redundant(face, last) {
                continue;
            }
            let t = match m % 3 {
                0 => 1,
                1 => 2,
                _ => 3,
            };
            self.path.push(m);
            self.current_centers[face] = (self.current_centers[face] + t) & 3;
            if self.phase2(
                self.mt.cp[cp as usize][m],
                self.mt.ep8[ep as usize][m],
                self.mt.slice_p[sp as usize][m],
                depth - 1,
                face,
            ) {
                return true;
            }
            self.current_centers[face] = (self.current_centers[face] + 4 - t) & 3;
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
