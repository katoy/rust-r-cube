use crate::coord::{move_cube_18, RawCube};
use crate::tables::{MoveTable, PruningTable};
use web_time::Instant;

pub struct Search {
    mt: &'static MoveTable,
    pt: &'static PruningTable,
    start: Instant,
    budget_ms: u128,
    pub nodes: u64,
    pub timed_out: bool,
    pub path: Vec<usize>,
    initial: RawCube,
    max_total: usize,
    target_centers: Option<[i32; 6]>,
    current_centers: [i32; 6],
    best_solution: Option<Vec<usize>>,
    pub best_phase1_len: usize,
    current_p1_len: usize,
}
impl Search {
    pub fn new(budget_ms: u32) -> Self {
        Self {
            mt: MoveTable::get(),
            pt: PruningTable::get(),
            start: Instant::now(),
            budget_ms: budget_ms as u128,
            nodes: 0,
            timed_out: false,
            path: Vec::with_capacity(32),
            initial: RawCube::default(),
            max_total: 22,
            target_centers: None,
            current_centers: [0; 6],
            best_solution: None,
            best_phase1_len: 0,
            current_p1_len: 0,
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
        self.best_phase1_len = 0;

        // 短い手数（深さ 1..=5）の直接探索（理論的最短手数を瞬時に見つける）
        for d in 1..=5 {
            self.path.clear();
            if let Some(target) = self.target_centers {
                self.current_centers = target;
            }
            if self.direct_solve(cube, d, 99) {
                self.best_phase1_len = self.path.len();
                return Some(self.path.clone());
            }
            if self.timed_out {
                return None;
            }
        }

        for total in [22, 24, 30] {
            self.max_total = total;

            for depth in 0..=12 {
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
                ) || self.timed_out
                {
                    break;
                }
            }
            if self.best_solution.is_some() || self.timed_out {
                break;
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
        if self.nodes & 1023 == 0 && self.start.elapsed().as_millis() >= self.budget_ms {
            self.timed_out = true;
        }
        self.timed_out
    }
    fn min_phase2_center_moves(&self) -> u8 {
        self.current_centers.iter().filter(|&&c| c != 0).count() as u8
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
                // Phase 2 では R(1), F(2), L(4), B(5) は 180° (+2 mod 4) 回転のみ許可される。
                // したがって、残余センター回転が奇数の面がある場合、Phase 2 内で 0 mod 4 に解消することはできない。
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
            self.current_p1_len = self.path.len();
            let mut found = false;
            let max_d = max.min(self.max_total.saturating_sub(self.path.len()));
            for d in min_d..=max_d {
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
            self.best_solution = Some(self.path.clone());
            self.best_phase1_len = self.current_p1_len;
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

    #[test]
    fn test_min_phase2_center_moves() {
        let mut search = Search::new(1000);
        assert_eq!(search.min_phase2_center_moves(), 0);

        search.current_centers = [1, 2, 0, 3, 2, 0];
        // 非ゼロのセンター回転面 (0, 1, 3, 4) の個数は 4
        assert_eq!(search.min_phase2_center_moves(), 4);

        search.current_centers = [0, 0, 0, 0, 0, 0];
        assert_eq!(search.min_phase2_center_moves(), 0);

        search.current_centers = [2, 2, 2, 2, 2, 2];
        assert_eq!(search.min_phase2_center_moves(), 6);
    }

    #[test]
    fn test_search_solved_with_zero_target_centers() {
        let mut search = Search::new(1000).with_target_centers([0; 6]);
        let res = search.solve(&RawCube::default());
        assert_eq!(res, Some(Vec::new()));
    }

    #[test]
    fn test_search_direct_solve_edge_cases() {
        let mut search = Search::new(1000).with_target_centers([1, 0, 0, 0, 0, 0]);
        search.current_centers = [1, 0, 0, 0, 0, 0];
        // キューブは default だが centers が 0 ではないので false
        assert!(!search.direct_solve(&RawCube::default(), 0, 99));

        // キューブが default でない場合
        let mut non_default = RawCube::default();
        non_default.co[0] = 1;
        assert!(!search.direct_solve(&non_default, 0, 99));

        // non_zero_centers > depth
        search.current_centers = [1, 2, 1, 2, 1, 0];
        assert!(!search.direct_solve(&RawCube::default(), 2, 99));
    }

    #[test]
    fn test_search_phase2_branch_coverage() {
        let mut search = Search::new(1000).with_target_centers([0; 6]);

        // depth == 0 でパーツ未解決 (cp != 0)
        assert!(!search.phase2(1, 0, 0, 0, 99));
        assert!(!search.phase2(0, 1, 0, 0, 99));
        assert!(!search.phase2(0, 0, 1, 0, 99));

        // depth == 0 でパーツは解決 (0, 0, 0) だが centers が 0 ではない
        search.current_centers = [1, 0, 0, 0, 0, 0];
        assert!(!search.phase2(0, 0, 0, 0, 99));

        // センター奇数回転チェック (face 1, 2, 4, 5 のいずれかが奇数)
        search.current_centers = [0, 1, 0, 0, 0, 0]; // R が奇数
        assert!(!search.phase2(0, 0, 0, 1, 99));

        search.current_centers = [0, 0, 1, 0, 0, 0]; // F が奇数
        assert!(!search.phase2(0, 0, 0, 1, 99));

        search.current_centers = [0, 0, 0, 0, 1, 0]; // L が奇数
        assert!(!search.phase2(0, 0, 0, 1, 99));

        search.current_centers = [0, 0, 0, 0, 0, 1]; // B が奇数
        assert!(!search.phase2(0, 0, 0, 1, 99));

        // min_phase2_center_moves > depth
        search.current_centers = [2, 2, 2, 2, 2, 2]; // 6手必要
        assert!(!search.phase2(0, 0, 0, 2, 99));

        // depth == 0 で完全解決
        search.current_centers = [0; 6];
        assert!(search.phase2(0, 0, 0, 0, 99));
        assert!(search.best_solution.is_some());

        // phase2 先頭でのタイムアウト分岐
        search.timed_out = true;
        assert!(!search.phase2(0, 0, 0, 1, 99));

        // phase2 再帰ループ内でのタイムアウト分岐 (timed_out チェック)
        let mut search2 = Search::new(0);
        search2.timed_out = true;
        assert!(!search2.phase2(1, 0, 0, 1, 99));
    }

    #[test]
    fn test_search_phase1_branch_coverage() {
        let mut search = Search::new(1000).with_target_centers([0; 6]);
        search.max_total = 5;
        search.path = vec![0; 10]; // path.len() > max_total
        assert!(!search.phase1(0, 0, 0, 0, 99));

        // phase1 depth == 0 で target_centers の奇数回転チェック
        search.path = vec![0; 2];
        search.max_total = 10;
        search.current_centers = [0, 1, 0, 0, 0, 0]; // R が奇数
        assert!(!search.phase1(0, 0, 0, 0, 99));
    }

    #[test]
    fn test_search_timeout_returns_none() {
        let sc = crate::cube::parse_moves("R U F B L D R U F B L D").unwrap();
        let scrambled = crate::cube::apply(&RawCube::default(), &sc);
        let mut search = Search::new(0);
        assert!(search.solve(&scrambled).is_none());
    }
}
