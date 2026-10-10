use super::coord::{
    get_group_a_idx, get_group_b_idx, get_group_c_idx, get_group_d_idx, move_cube_18, Corner,
    RawCube,
};
use super::tables::PruningTable;
use std::collections::{HashSet, VecDeque};
use std::sync::OnceLock;

pub const DEFAULT_MAX_NODES: u64 = 10_000_000;
pub const MAX_DEPTH: usize = 14;

pub struct Search {
    pruning_table: &'static PruningTable,
    pub max_nodes: u64,
    pub nodes: u64,
    pub timed_out: bool,
    start: Option<web_time::Instant>,
    budget_ms: Option<u128>,
    solution: Vec<usize>,
}

impl Default for Search {
    fn default() -> Self {
        Self::new()
    }
}

impl Search {
    pub fn new() -> Self {
        Self {
            pruning_table: PruningTable::get(),
            max_nodes: DEFAULT_MAX_NODES,
            nodes: 0,
            timed_out: false,
            start: None,
            budget_ms: None,
            solution: Vec::with_capacity(MAX_DEPTH),
        }
    }

    pub fn with_budget(mut self, budget_ms: u32) -> Self {
        self.budget_ms = Some(budget_ms as u128);
        self
    }

    /// 2x2 キューブの最短解を探索します。
    /// `include_orientation`:
    ///   - true: 標準向き（白上・赤右・緑前など）に完全一致する完成状態を目指す
    ///   - false: 6面が揃っていればどの向きでもよい最短解を探索
    pub fn solve(&mut self, rc: &RawCube, include_orientation: bool) -> Option<Vec<usize>> {
        self.nodes = 0;
        self.timed_out = false;
        self.start = Some(web_time::Instant::now());

        if *rc == RawCube::default() {
            return Some(Vec::new());
        }

        if include_orientation {
            self.solve_single_target(rc, MAX_DEPTH)
        } else {
            let orientations = get_all_orientations();
            // すでにいずれかの向きで完成しているかチェック
            if orientations
                .iter()
                .any(|rot| rot.multiply(rc) == RawCube::default())
            {
                return Some(Vec::new());
            }

            // 2x2 では、向きを問わない最短手数は DBL コーナーを位置 6・向き 0 に合わせたときの
            // 最短手数と完全に一致する。24 向きのループを行わず、DBL を固定する 1 向きのみ探索する
            let rot = orientations
                .iter()
                .find(|rot| {
                    let oriented = rot.multiply(rc);
                    oriented.cp[Corner::DBL as usize] == Corner::DBL
                        && oriented.co[Corner::DBL as usize] == 0
                })
                .copied()
                .unwrap_or_default();

            let oriented_rc = rot.multiply(rc);
            self.solve_single_target(&oriented_rc, MAX_DEPTH)
        }
    }

    /// 単一の目標状態（RawCube::default()）に対する IDA* 最短探索
    pub fn solve_single_target(&mut self, rc: &RawCube, max_depth: usize) -> Option<Vec<usize>> {
        for depth in 0..=max_depth {
            self.solution.clear();
            if self.ida_star(*rc, depth, 255) {
                return Some(self.solution.clone());
            }
            if self.nodes >= self.max_nodes || self.timed_out {
                break;
            }
        }
        None
    }

    fn ida_star(&mut self, rc: RawCube, depth: usize, last_face: usize) -> bool {
        self.nodes += 1;
        if self.nodes >= self.max_nodes {
            return false;
        }
        if self.nodes & 4095 == 0 {
            if let (Some(start), Some(budget)) = (self.start, self.budget_ms) {
                if start.elapsed().as_millis() >= budget {
                    self.timed_out = true;
                    return false;
                }
            }
        }

        let idx_a = get_group_a_idx(&rc);
        let idx_b = get_group_b_idx(&rc);
        let idx_c = get_group_c_idx(&rc);
        let idx_d = get_group_d_idx(&rc);

        let h_a = self.pruning_table.group_a[idx_a] as usize;
        let h_b = self.pruning_table.group_b[idx_b] as usize;
        let h_c = self.pruning_table.group_c[idx_c] as usize;
        let h_d = self.pruning_table.group_d[idx_d] as usize;

        let h = h_a.max(h_b).max(h_c).max(h_d);

        if h == 0 {
            return true;
        }
        if h > depth {
            return false;
        }

        for face in 0..6 {
            // 同一面の連続回転を排除
            if face == last_face {
                continue;
            }
            // 対面回転の順序を固定（U-D, R-L, F-B）
            // 0:U と 3:D, 1:R と 4:L, 2:F と 5:B
            if (face == 3 && last_face == 0)
                || (face == 4 && last_face == 1)
                || (face == 5 && last_face == 2)
            {
                continue;
            }

            for m_offset in 0..3 {
                let m = face * 3 + m_offset;
                let next_rc = rc.multiply(move_cube_18(m));

                self.solution.push(m);
                if self.ida_star(next_rc, depth - 1, face) {
                    return true;
                }
                self.solution.pop();
            }
        }

        false
    }
}

/// 2x2 キューブの24通りの空間回転（Orientation）を取得します。
pub fn get_all_orientations() -> &'static [RawCube] {
    static ORIENTATIONS: OnceLock<Vec<RawCube>> = OnceLock::new();
    ORIENTATIONS.get_or_init(generate_all_orientations)
}

fn generate_all_orientations() -> Vec<RawCube> {
    // 基本空間回転:
    // X (R L'), Y (U D'), Z (F B')
    // U=0, R=1, F=2, D=3, L=4, B=5
    // R: 3, L': 4*3+2=14
    // U: 0, D': 3*3+2=11
    // F: 6, B': 5*3+2=17
    let rot_x = move_cube_18(3).multiply(move_cube_18(14));
    let rot_y = move_cube_18(0).multiply(move_cube_18(11));
    let rot_z = move_cube_18(6).multiply(move_cube_18(17));

    let generators = [rot_x, rot_y, rot_z];

    let mut list = Vec::with_capacity(24);
    let mut visited = HashSet::new();
    let mut q = VecDeque::new();

    let start = RawCube::default();
    visited.insert(start);
    list.push(start);
    q.push_back(start);

    while let Some(cur) = q.pop_front() {
        for gen in &generators {
            let next = cur.multiply(gen);
            if visited.insert(next) {
                list.push(next);
                q.push_back(next);
            }
        }
    }

    assert_eq!(list.len(), 24, "Failed to generate 24 cube orientations");
    list
}
