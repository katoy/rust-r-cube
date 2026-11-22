use crate::coord::{move_cube_18, rank_perm4, rank_perm8, RawCube};
use crate::tables::{MoveTable, PruningTable, PHASE2_MOVES};
use web_time::Instant;

/// 神の数字（HTM）。どの状態も 20 手以内で解けることが証明されている
pub const GODS_NUMBER: usize = 20;

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
    has_custom_max: bool,
    /// 20 手以内の解が得られるまで時間予算を超えて探索を続けるか
    gods_guarantee: bool,
    allow_deep_phase2: bool,
    target_centers: Option<[i32; 6]>,
    current_centers: [i32; 6],
    best_solution: Option<Vec<usize>>,
    pub best_phase1_len: usize,
    current_p1_len: usize,
    /// 探索中の G1 基準軸（0: UD, 1: RL, 2: FB）
    axis: usize,
    /// 逆状態を探索中か（見つかった解は逆順・逆回転にして元の状態の解へ戻す）
    inverse: bool,
    /// 最良解が逆状態の探索から得られたか（Phase 1 / Phase 2 の区切りは無意味になる）
    pub best_from_inverse: bool,
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
            has_custom_max: false,
            gods_guarantee: false,
            allow_deep_phase2: false,
            target_centers: None,
            current_centers: [0; 6],
            best_solution: None,
            best_phase1_len: 0,
            current_p1_len: 0,
            axis: 0,
            inverse: false,
            best_from_inverse: false,
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
    pub fn set_max_total(&mut self, max: usize) {
        self.max_total = max;
        self.has_custom_max = true;
    }
    /// 神の数字（20 手）以内の解が見つかるまで、時間予算を超えても探索を継続する。
    /// どの状態にも 20 手以内の解が存在し、Phase 1 を深さ 20 まで探索すればその解自体が Phase 1 の解として
    /// 列挙されるため、探索は必ず終了する。センター向き指定（`with_target_centers`）とは併用しない。
    pub fn with_gods_number_guarantee(mut self) -> Self {
        self.gods_guarantee = true;
        self
    }
    #[inline]
    fn has_gods_solution(&self) -> bool {
        self.best_solution
            .as_ref()
            .is_some_and(|b| b.len() <= GODS_NUMBER)
    }
    #[inline]
    fn has_acceptable_solution(&self) -> bool {
        if self.target_centers.is_some() {
            self.best_solution.is_some()
        } else {
            self.has_gods_solution()
        }
    }
    #[inline]
    fn h1(&self, twist: u16, flip: u16, slice: u16) -> u8 {
        self.pt
            .get_twist_slice(twist as usize, slice as usize)
            .max(self.pt.get_flip_slice(flip as usize, slice as usize))
    }
    pub fn solve(&mut self, cube: &RawCube) -> Option<Vec<usize>> {
        self.nodes = 0;
        self.timed_out = false;
        self.allow_deep_phase2 = false;
        self.start = Instant::now();
        self.path.clear();

        if *cube == RawCube::default() {
            if let Some(target) = self.target_centers {
                if target.iter().all(|&c| c == 0) {
                    return Some(Vec::new());
                }
            } else {
                return Some(Vec::new());
            }
        }
        if self.max_total == 0 {
            return None;
        }

        self.initial = *cube;
        self.best_solution = None;
        self.best_phase1_len = 0;
        self.best_from_inverse = false;

        let initial_max = self.max_total;

        // 3 軸共役の下界を使って 1..=5 手の直接探索の要不要と開始深さを決定する
        let conj_fwd = [*cube, conjugate_axis(cube, 1), conjugate_axis(cube, 2)];
        let t0 = conj_fwd[0].get_twist();
        let f0 = conj_fwd[0].get_flip();
        let s0 = conj_fwd[0].get_ud_slice();
        let t1 = conj_fwd[1].get_twist();
        let f1 = conj_fwd[1].get_flip();
        let s1 = conj_fwd[1].get_ud_slice();
        let t2 = conj_fwd[2].get_twist();
        let f2 = conj_fwd[2].get_flip();
        let s2 = conj_fwd[2].get_ud_slice();
        let init_h = self
            .h1(t0, f0, s0)
            .max(self.h1(t1, f1, s1))
            .max(self.h1(t2, f2, s2));
        let min_centers = self
            .target_centers
            .map_or(0, |tc| tc.iter().filter(|&&c| c != 0).count());
        let start_d = (init_h as usize).max(min_centers).max(1);
        let max_direct = 5.min(self.max_total);

        if start_d <= max_direct {
            for d in start_d..=max_direct {
                self.path.clear();
                if let Some(target) = self.target_centers {
                    self.current_centers = target;
                }
                if self.direct_solve_coords(cube, t0, f0, s0, d as u8, 99) {
                    self.best_phase1_len = self.path.len();
                    return Some(self.path.clone());
                }
                if self.timed_out {
                    return None;
                }
            }
        }

        struct DirEntry {
            axis: usize,
            inverse: bool,
            cube: RawCube,
            centers: [i32; 6],
            twist: u16,
            flip: u16,
            slice: u16,
        }

        let inv = inverse_cube(cube);
        let tables = axis_tables();
        let has_centers = self.target_centers.is_some();
        let mut directions: Vec<DirEntry> = Vec::with_capacity(6);
        let mut push_dir = |candidate: DirEntry| {
            if let Some(existing) = directions.iter_mut().find(|d| d.cube == candidate.cube) {
                if has_centers
                    && odd_side_centers(candidate.centers) < odd_side_centers(existing.centers)
                {
                    *existing = candidate;
                }
            } else {
                directions.push(candidate);
            }
        };
        let coords_fwd = [(t0, f0, s0), (t1, f1, s1), (t2, f2, s2)];
        for a in 0..3 {
            let dir_centers_fwd = if let Some(target) = self.target_centers {
                std::array::from_fn(|f| target[tables.unconj_move[a][f * 3] / 3])
            } else {
                [0; 6]
            };
            let (twist, flip, slice) = coords_fwd[a];
            push_dir(DirEntry {
                axis: a,
                inverse: false,
                cube: conj_fwd[a],
                centers: dir_centers_fwd,
                twist,
                flip,
                slice,
            });

            let inv_cube = conjugate_axis(&inv, a);
            let dir_centers_inv = if has_centers {
                std::array::from_fn(|f| (4 - dir_centers_fwd[f]) & 3)
            } else {
                [0; 6]
            };
            push_dir(DirEntry {
                axis: a,
                inverse: true,
                cube: inv_cube,
                centers: dir_centers_inv,
                twist: inv_cube.get_twist(),
                flip: inv_cube.get_flip(),
                slice: inv_cube.get_ud_slice(),
            });
        }

        // UD / RL / FB の 3 軸 × 順・逆状態の重複除去済み最大 6 方向を Phase 1 の深さごとに交互探索する
        'pass1: for depth in 0..=12 {
            for dir in &directions {
                if depth > self.max_total {
                    break 'pass1;
                }
                self.axis = dir.axis;
                self.inverse = dir.inverse;
                self.initial = dir.cube;
                self.path.clear();
                if self.target_centers.is_some() {
                    self.current_centers = dir.centers;
                }
                self.phase1(dir.twist, dir.flip, dir.slice, depth as u8, 99);
                if self.timed_out || self.has_acceptable_solution() {
                    break 'pass1;
                }
            }
        }

        // センター向き指定なしで神の数字（20 手）以内の解が得られていない場合、深さ 13 以降も 6 方向を継続する
        if self.target_centers.is_none() && !self.timed_out && !self.has_gods_solution() {
            'deepen: for depth in 13.. {
                for dir in &directions {
                    if depth > self.max_total {
                        break 'deepen;
                    }
                    self.axis = dir.axis;
                    self.inverse = dir.inverse;
                    self.initial = dir.cube;
                    self.path.clear();
                    self.phase1(dir.twist, dir.flip, dir.slice, depth as u8, 99);
                    if self.timed_out || self.has_gods_solution() {
                        break 'deepen;
                    }
                }
            }
        }

        // 1回目の探索（Phase 2 上限 12）で解が見つからなかった場合、段階的に上限を緩和する。
        // has_custom_max の場合、1回目のループですでに initial_max までの探索は完了しているため、
        // Phase 2 上限 12 のままの total == 24 はスキップし、initial_max > 12 であれば
        // 即座に Phase 2 上限 18（total == 30）の探索へ進む。
        if self.best_solution.is_none() && !self.timed_out {
            let totals: &[usize] = if self.has_custom_max {
                if initial_max > 12 {
                    &[30]
                } else {
                    &[]
                }
            } else if initial_max >= 24 {
                &[30]
            } else if initial_max >= 22 {
                &[24, 30]
            } else {
                &[]
            };

            for &total in totals {
                if total == 30 {
                    self.allow_deep_phase2 = true;
                }
                self.max_total = if self.has_custom_max {
                    total.min(initial_max)
                } else {
                    total
                };
                'pass2: for depth in 0..=12 {
                    for dir in &directions {
                        if depth > self.max_total {
                            break 'pass2;
                        }
                        self.axis = dir.axis;
                        self.inverse = dir.inverse;
                        self.initial = dir.cube;
                        self.path.clear();
                        if self.target_centers.is_some() {
                            self.current_centers = dir.centers;
                        }
                        self.phase1(dir.twist, dir.flip, dir.slice, depth as u8, 99);
                        if self.best_solution.is_some() || self.timed_out {
                            break 'pass2;
                        }
                    }
                }
                if self.best_solution.is_some() || self.timed_out {
                    break;
                }
            }
        }

        self.axis = 0;
        self.inverse = false;
        self.initial = *cube;
        self.best_solution.clone()
    }
    pub fn direct_solve(&mut self, cube: &RawCube, depth: u8, last: usize) -> bool {
        let twist = cube.get_twist();
        let flip = cube.get_flip();
        let slice = cube.get_ud_slice();
        self.direct_solve_coords(cube, twist, flip, slice, depth, last)
    }
    fn direct_solve_coords(
        &mut self,
        cube: &RawCube,
        twist: u16,
        flip: u16,
        slice: u16,
        depth: u8,
        last: usize,
    ) -> bool {
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
        if self.h1(twist, flip, slice) > depth {
            return false;
        }
        let has_centers = self.target_centers.is_some();
        if has_centers {
            let non_zero_centers = self.current_centers.iter().filter(|&&c| c != 0).count() as u8;
            if non_zero_centers > depth {
                return false;
            }
        }
        let next_depth = depth - 1;
        for face in 0..6 {
            if redundant(face, last) {
                continue;
            }
            for turn in 0..3 {
                let m = face * 3 + turn;
                let ns = self.mt.ud_slice[slice as usize][m];
                let nt = self.mt.twist[twist as usize][m];
                let nf;
                if next_depth == 0 {
                    // depth 0 で完成状態（G1 部分群にも含まれる）以外は RawCube::multiply を省略
                    if nt != 0 || ns != 0 {
                        continue;
                    }
                    nf = self.mt.flip[flip as usize][m];
                    if nf != 0 {
                        continue;
                    }
                } else {
                    if self.pt.get_twist_slice(nt as usize, ns as usize) > next_depth {
                        continue;
                    }
                    nf = self.mt.flip[flip as usize][m];
                    if self.pt.get_flip_slice(nf as usize, ns as usize) > next_depth {
                        continue;
                    }
                }
                let t = match turn {
                    0 => 1,
                    1 => 2,
                    _ => 3,
                };
                self.path.push(m);
                if has_centers {
                    self.current_centers[face] = (self.current_centers[face] + t) & 3;
                }
                let next_cube = cube.multiply(move_cube_18(m));
                if self.direct_solve_coords(&next_cube, nt, nf, ns, next_depth, face) {
                    return true;
                }
                if has_centers {
                    self.current_centers[face] = (self.current_centers[face] + 4 - t) & 3;
                }
                self.path.pop();
                if self.timed_out {
                    return false;
                }
            }
        }
        false
    }
    fn exhausted(&mut self) -> bool {
        if self.timed_out {
            return true;
        }
        // 20 手保証モードでは、20 手以内の解が得られるまで時間予算を無視して探索を続ける
        let extending = self.gods_guarantee && !self.has_gods_solution();
        if self.budget_ms == 0 && !extending {
            self.timed_out = true;
            return true;
        }
        self.nodes += 1;
        if !extending
            && self.nodes & 1023 == 0
            && self.start.elapsed().as_millis() >= self.budget_ms
        {
            self.timed_out = true;
        }
        self.timed_out
    }
    fn min_phase1_center_moves(&self) -> u8 {
        if self.target_centers.is_some() {
            // R(1), F(2), L(4), B(5) は Phase 2 では 180° 回転しか行えないため、
            // Phase 1 終了時点でこれらのセンター回転は偶数 (0 mod 2) でなければならない。
            // 奇数回転の面の数が残り深さを超える場合は到達不能として枝刈りする。
            odd_side_centers(self.current_centers)
        } else {
            0
        }
    }
    fn min_phase2_center_moves(&self) -> u8 {
        self.current_centers.iter().filter(|&&c| c != 0).count() as u8
    }
    fn phase1(&mut self, twist: u16, flip: u16, slice: u16, depth: u8, last: usize) -> bool {
        if self.h1(twist, flip, slice) > depth {
            return false;
        }
        self.phase1_search(twist, flip, slice, depth, last)
    }
    fn phase1_search(&mut self, twist: u16, flip: u16, slice: u16, depth: u8, last: usize) -> bool {
        if self.exhausted() {
            return false;
        }
        if self.min_phase1_center_moves() > depth {
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
            let max_d = (self.max_total - self.path.len()).min(
                if self.allow_deep_phase2 || self.max_total == 30 {
                    18
                } else {
                    12
                },
            );
            let min_d = if self.target_centers.is_some() {
                self.min_phase2_center_moves() as usize
            } else {
                0
            };
            if min_d > max_d {
                return false;
            }
            let (cp_arr, ep_arr) = replay_g1_perms(&self.initial, &self.path);
            let cp = rank_perm8(&cp_arr);
            let sp = rank_perm4(&[ep_arr[8] - 8, ep_arr[9] - 8, ep_arr[10] - 8, ep_arr[11] - 8]);
            let h_cp = self.pt.get_cp_slice(cp as usize, sp as usize) as usize;
            if h_cp > max_d {
                return false;
            }
            let ep = rank_perm8(&ep_arr[0..8].try_into().unwrap());
            let h_ep = self.pt.get_ep8_slice(ep as usize, sp as usize) as usize;
            let start_d = min_d.max(h_cp).max(h_ep);
            if start_d > max_d {
                return false;
            }
            self.current_p1_len = self.path.len();
            let mut found = false;
            for d in start_d..=max_d {
                if self.phase2(cp, ep, sp, d as u8, last) {
                    found = true;
                    // 見つかった解で max_total が縮小されたので、これ以上大きい d は探索不要
                    break;
                }
                if self.timed_out {
                    break;
                }
            }
            return found;
        }
        // depth == 1 で既に G1 に入っている場合、あと 1 手 G1 の手を足しても
        // 前の深さ（depth == 0）の Phase 2 探索で同じ手順が網羅済み（max_total の残り <= phase2_cap のとき）
        if depth == 1 && twist == 0 && flip == 0 && slice == 0 {
            let phase2_cap = if self.allow_deep_phase2 || self.max_total == 30 {
                18
            } else {
                12
            };
            if self.path.len() + phase2_cap >= self.max_total {
                return false;
            }
        }
        let next_depth = depth - 1;
        let has_centers = self.target_centers.is_some();
        for face in 0..6 {
            if redundant(face, last) {
                continue;
            }
            for turn in 0..3 {
                let m = face * 3 + turn;
                let ns = self.mt.ud_slice[slice as usize][m];
                let nt = self.mt.twist[twist as usize][m];
                let nf;
                if next_depth == 0 {
                    if nt != 0 || ns != 0 {
                        continue;
                    }
                    nf = self.mt.flip[flip as usize][m];
                    if nf != 0 {
                        continue;
                    }
                } else {
                    if self.pt.get_twist_slice(nt as usize, ns as usize) > next_depth {
                        continue;
                    }
                    nf = self.mt.flip[flip as usize][m];
                    if self.pt.get_flip_slice(nf as usize, ns as usize) > next_depth {
                        continue;
                    }
                }
                let t = match turn {
                    0 => 1,
                    1 => 2,
                    _ => 3,
                };
                self.path.push(m);
                if has_centers {
                    self.current_centers[face] = (self.current_centers[face] + t) & 3;
                }
                let _ = self.phase1_search(nt, nf, ns, next_depth, face);
                if has_centers {
                    self.current_centers[face] = (self.current_centers[face] + 4 - t) & 3;
                }
                self.path.pop();
                if self.timed_out || self.has_acceptable_solution() {
                    return self.best_solution.is_some();
                }
            }
        }
        self.best_solution.is_some()
    }
    fn phase2(&mut self, cp: u16, ep: u16, sp: u16, depth: u8, last: usize) -> bool {
        let distance = self
            .pt
            .get_cp_slice(cp as usize, sp as usize)
            .max(self.pt.get_ep8_slice(ep as usize, sp as usize));
        if distance > depth {
            return false;
        }
        self.phase2_search(cp, ep, sp, depth, last)
    }
    fn phase2_search(&mut self, cp: u16, ep: u16, sp: u16, depth: u8, last: usize) -> bool {
        if self.exhausted() {
            return false;
        }
        let has_centers = self.target_centers.is_some();
        if has_centers {
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
            let table = &axis_tables().unconj_move[self.axis];
            let mut sol: Vec<usize> = self.path.iter().map(|&m| table[m]).collect();
            if self.inverse {
                // 逆状態の解 M に対し、元の状態の解は M⁻¹（逆順・各手を逆回転）
                sol.reverse();
                for m in sol.iter_mut() {
                    *m = *m / 3 * 3 + 2 - *m % 3;
                }
            }
            self.best_solution = Some(sol);
            self.best_from_inverse = self.inverse;
            self.best_phase1_len = if self.inverse { 0 } else { self.current_p1_len };
            self.max_total = self.path.len().saturating_sub(1);
            return true;
        }
        let next_depth = depth - 1;
        for (p2_idx, &m) in PHASE2_MOVES.iter().enumerate() {
            let face = m / 3;
            if redundant(face, last) {
                continue;
            }
            let nsp = self.mt.slice_p[sp as usize][p2_idx];
            let ncp = self.mt.cp[cp as usize][p2_idx];
            let nep;
            if next_depth == 0 {
                if ncp != 0 || nsp != 0 {
                    continue;
                }
                nep = self.mt.ep8[ep as usize][p2_idx];
                if nep != 0 {
                    continue;
                }
            } else {
                if self.pt.get_cp_slice(ncp as usize, nsp as usize) > next_depth {
                    continue;
                }
                nep = self.mt.ep8[ep as usize][p2_idx];
                if self.pt.get_ep8_slice(nep as usize, nsp as usize) > next_depth {
                    continue;
                }
            }
            let t = match m % 3 {
                0 => 1,
                1 => 2,
                _ => 3,
            };
            self.path.push(m);
            if has_centers {
                self.current_centers[face] = (self.current_centers[face] + t) & 3;
            }
            let solved = self.phase2_search(ncp, nep, nsp, next_depth, face);
            if has_centers {
                self.current_centers[face] = (self.current_centers[face] + 4 - t) & 3;
            }
            self.path.pop();
            if solved {
                return true;
            }
            if self.timed_out {
                return false;
            }
        }
        false
    }
}

struct PermMove18 {
    cp: [u8; 8],
    ep: [u8; 12],
}

fn perm_moves_18() -> &'static [PermMove18; 18] {
    static TABLE: std::sync::OnceLock<[PermMove18; 18]> = std::sync::OnceLock::new();
    TABLE.get_or_init(|| {
        std::array::from_fn(|m| {
            let mc = move_cube_18(m);
            PermMove18 {
                cp: mc.cp.map(|c| c as u8),
                ep: mc.ep.map(|e| e as u8),
            }
        })
    })
}

/// G1 突入時用の高速リプレイ。向き（co/eo）と剰余演算を行わず 20 バイトの置換のみを計算する。
#[inline]
fn replay_g1_perms(initial: &RawCube, path: &[usize]) -> ([u8; 8], [u8; 12]) {
    let moves = perm_moves_18();
    let mut cp = initial.cp.map(|c| c as u8);
    let mut ep = initial.ep.map(|e| e as u8);
    for &m in path {
        let mv = &moves[m];
        cp = [
            cp[mv.cp[0] as usize],
            cp[mv.cp[1] as usize],
            cp[mv.cp[2] as usize],
            cp[mv.cp[3] as usize],
            cp[mv.cp[4] as usize],
            cp[mv.cp[5] as usize],
            cp[mv.cp[6] as usize],
            cp[mv.cp[7] as usize],
        ];
        ep = [
            ep[mv.ep[0] as usize],
            ep[mv.ep[1] as usize],
            ep[mv.ep[2] as usize],
            ep[mv.ep[3] as usize],
            ep[mv.ep[4] as usize],
            ep[mv.ep[5] as usize],
            ep[mv.ep[6] as usize],
            ep[mv.ep[7] as usize],
            ep[mv.ep[8] as usize],
            ep[mv.ep[9] as usize],
            ep[mv.ep[10] as usize],
            ep[mv.ep[11] as usize],
        ];
    }
    (cp, ep)
}
#[inline(always)]
fn odd_side_centers(centers: [i32; 6]) -> u8 {
    ((centers[1] & 1) + (centers[2] & 1) + (centers[4] & 1) + (centers[5] & 1)) as u8
}
#[inline(always)]
fn redundant(face: usize, last: usize) -> bool {
    face == last || ((3..6).contains(&last) && face + 3 == last)
}

/// 軸変換用の対称操作と手の対応表
struct AxisTables {
    /// S_URF3 の 0 / 1 / 2 乗
    sym: [RawCube; 3],
    /// 各対称操作の逆元
    sym_inv: [RawCube; 3],
    /// 変換後の座標系での手 m を、元の座標系での手に戻す対応表（S⁻¹·m·S）
    unconj_move: [[usize; 18]; 3],
}

/// URF-DBL 対角軸まわりの 120° 回転 S_URF3（Kociemba の定義）を返す。
/// 共役変換 S·C·S⁻¹ により、UD 軸 → RL 軸 → FB 軸と G1 の基準軸を切り替える。
fn sym_urf3() -> RawCube {
    use crate::coord::{Corner as C, Edge as E};
    RawCube {
        cp: [
            C::UFR,
            C::DFR,
            C::DLF,
            C::UFL,
            C::UBR,
            C::DRB,
            C::DBL,
            C::ULB,
        ],
        co: [1, 2, 1, 2, 2, 1, 2, 1],
        ep: [
            E::UF,
            E::FR,
            E::DF,
            E::FL,
            E::UB,
            E::BR,
            E::DB,
            E::BL,
            E::UR,
            E::DR,
            E::DL,
            E::UL,
        ],
        eo: [1, 0, 1, 0, 1, 0, 1, 0, 1, 1, 1, 1],
    }
}

fn axis_tables() -> &'static AxisTables {
    static TABLES: std::sync::OnceLock<AxisTables> = std::sync::OnceLock::new();
    TABLES.get_or_init(|| {
        let s1 = sym_urf3();
        let s2 = s1.multiply(&s1);
        let sym = [RawCube::default(), s1, s2];
        let sym_inv = [RawCube::default(), s2, s1];
        let mut unconj_move = [[0usize; 18]; 3];
        for a in 0..3 {
            for (m, slot) in unconj_move[a].iter_mut().enumerate() {
                let target = sym_inv[a].multiply(move_cube_18(m)).multiply(&sym[a]);
                *slot = (0..18)
                    .find(|&k| *move_cube_18(k) == target)
                    .expect("共役変換後の手が基本手に一致しない");
            }
        }
        AxisTables {
            sym,
            sym_inv,
            unconj_move,
        }
    })
}

/// キューブの逆元（C·C⁻¹ = 単位元）を返す
fn inverse_cube(cube: &RawCube) -> RawCube {
    let mut inv = RawCube::default();
    for i in 0..8 {
        let c = cube.cp[i] as usize;
        inv.cp[c] = RawCube::default().cp[i];
        inv.co[c] = (3 - cube.co[i]) % 3;
    }
    for i in 0..12 {
        let e = cube.ep[i] as usize;
        inv.ep[e] = RawCube::default().ep[i];
        inv.eo[e] = (2 - cube.eo[i]) % 2;
    }
    inv
}

/// キューブを指定軸の座標系へ共役変換する（S·C·S⁻¹）
fn conjugate_axis(cube: &RawCube, axis: usize) -> RawCube {
    let t = axis_tables();
    t.sym[axis].multiply(cube).multiply(&t.sym_inv[axis])
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

    #[test]
    fn test_inverse_cube() {
        let cube = crate::cube::apply(&RawCube::default(), &crate::cube::scramble(3));
        let inv = inverse_cube(&cube);
        assert_eq!(cube.multiply(&inv), RawCube::default());
        assert_eq!(inv.multiply(&cube), RawCube::default());
        assert_eq!(inverse_cube(&RawCube::default()), RawCube::default());
    }

    #[test]
    fn test_sym_urf3_has_order_three() {
        let s1 = sym_urf3();
        assert_ne!(s1, RawCube::default());
        assert_eq!(s1.multiply(&s1).multiply(&s1), RawCube::default());
    }

    #[test]
    fn test_unconj_move_maps_faces_to_other_axes() {
        let t = axis_tables();
        for &unconj in &t.unconj_move {
            // 対応表は 18 手の全単射で、回転量（90° / 180° / 270°）を保存する
            let mut seen = unconj;
            seen.sort_unstable();
            assert_eq!(seen.to_vec(), (0..18).collect::<Vec<_>>());
            for (m, &mapped) in unconj.iter().enumerate() {
                assert_eq!(mapped % 3, m % 3);
            }
        }
        // 軸 0 は恒等、軸 1 / 2 では U がそれぞれ別の軸の手へ移る（面 % 3 = 0: UD, 1: RL, 2: FB）
        let axis_of_u = |a: usize| t.unconj_move[a][0] / 3 % 3;
        assert_eq!(t.unconj_move[0], std::array::from_fn::<usize, 18, _>(|m| m));
        let mut axes = [axis_of_u(0), axis_of_u(1), axis_of_u(2)];
        axes.sort_unstable();
        assert_eq!(axes, [0, 1, 2]);
    }

    #[test]
    fn test_conjugated_solution_maps_back_to_original() {
        // 変換後の座標系で手順 M' を適用した結果は、元の座標系で対応手順を適用した結果の共役に一致する
        let t = axis_tables();
        let cube = crate::cube::apply(&RawCube::default(), &crate::cube::scramble(7));
        let seq = crate::cube::scramble(11);
        for (a, unconj) in t.unconj_move.iter().enumerate() {
            let mapped: Vec<usize> = seq.iter().map(|&m| unconj[m]).collect();
            assert_eq!(
                crate::cube::apply(&conjugate_axis(&cube, a), &seq),
                conjugate_axis(&crate::cube::apply(&cube, &mapped), a)
            );
        }
    }

    #[test]
    fn test_superflip_solved_within_gods_number() {
        // スーパーフリップの最短手数は 20 手。予算を極小にしても保証モードなら 20 手（= 最短）の解を返すこと
        let sc =
            crate::cube::parse_moves("U R2 F B R B2 R U2 L B2 R U' D' R2 F R' L B2 U2 F2").unwrap();
        let cube = crate::cube::apply(&RawCube::default(), &sc);
        let sol = Search::new(1)
            .with_gods_number_guarantee()
            .solve(&cube)
            .unwrap();
        assert_eq!(crate::cube::apply(&cube, &sol), RawCube::default());
        assert_eq!(sol.len(), GODS_NUMBER);
    }

    #[test]
    fn test_random_states_solved_within_gods_number() {
        // 6 方向探索の導入前は約 3 割の状態で 21〜22 手を返していた。
        // 予算 0 でも保証モードなら全状態で 20 手以内の解を返すこと（時間に依存しない決定的な検証）
        for seed in 1..=10 {
            let cube = crate::cube::apply(&RawCube::default(), &crate::cube::scramble(seed));
            let mut search = Search::new(0).with_gods_number_guarantee();
            let sol = search.solve(&cube).unwrap();
            assert_eq!(crate::cube::apply(&cube, &sol), RawCube::default());
            assert!(sol.len() <= GODS_NUMBER, "seed {seed}: {} 手", sol.len());
            // 逆状態から得た解では Phase 1 / 2 の区切りを報告しない
            if search.best_from_inverse {
                assert_eq!(search.best_phase1_len, 0);
            }
        }
    }

    #[test]
    fn test_budget_without_guarantee_still_times_out() {
        // 保証モードを指定しない場合は、従来どおり予算 0 で探索しない
        let cube = crate::cube::apply(&RawCube::default(), &crate::cube::scramble(5));
        assert!(Search::new(0).solve(&cube).is_none());
    }

    #[test]
    fn test_corr03_phase1_depth_equals_max_total() {
        // 6 手で Phase 1 のみにちょうど G1=完成へ到達する状態（例: 6 手逆手順の解決）で max_total == 6 でも解が見つかること
        let sc = crate::cube::parse_moves("R F L B D R").unwrap();
        let inv: Vec<usize> = sc.iter().rev().map(|&m| m / 3 * 3 + 2 - m % 3).collect();
        let cube = crate::cube::apply(&RawCube::default(), &inv);
        let mut search = Search::new(2000);
        search.set_max_total(6);
        let sol = search
            .solve(&cube)
            .expect("depth == max_total must not be skipped");
        assert!(sol.len() <= 6);
        assert_eq!(crate::cube::apply(&cube, &sol), RawCube::default());
    }

    #[test]
    fn test_center_oriented_6dir_conjugation() {
        for seed in 1..=10 {
            let sc = crate::cube::scramble(seed);
            let cube = crate::cube::apply(&RawCube::default(), &sc);
            let centers = crate::supercube::apply_moves_to_centers([0; 6], &sc);
            let mut search = Search::new(3000).with_target_centers(centers);
            let sol = search
                .solve(&cube)
                .unwrap_or_else(|| panic!("seed {seed} timed out"));
            crate::verify_solution(&cube, &sol, true, Some(centers))
                .unwrap_or_else(|e| panic!("seed {seed} invalid: {e}"));
        }
    }

    #[test]
    fn test_t_perm_direct_solve_skips_redundant_nodes() {
        let sc = crate::cube::parse_moves("R U R' U' R' F R2 U' R' U' R U R' F'").unwrap();
        let cube = crate::cube::apply(&RawCube::default(), &sc);
        let mut search = Search::new(2000);
        let sol = search.solve(&cube).unwrap();
        assert_eq!(crate::cube::apply(&cube, &sol), RawCube::default());
        assert!(
            search.nodes < 50_000,
            "expected pruned node count < 50000, got {}",
            search.nodes
        );
    }
}
