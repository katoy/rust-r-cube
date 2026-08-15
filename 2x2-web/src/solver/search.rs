use super::coord::RawCube;
use super::tables::{MoveTable, PruningTable};
use crate::cube::Move;

pub const DEFAULT_MAX_NODES: usize = 10_000_000;

pub struct Search {
    pub move_table: &'static MoveTable,
    pub pruning_table: &'static PruningTable,
    pub max_nodes: usize,
    pub nodes_count: usize,
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
            move_table: MoveTable::get(),
            pruning_table: PruningTable::get(),
            max_nodes: DEFAULT_MAX_NODES,
            nodes_count: 0,
            solution: Vec::new(),
        }
    }

    pub fn solve(&mut self, rc: &RawCube, max_depth: usize) -> Option<Vec<Move>> {
        self.nodes_count = 0;
        for depth in 0..=max_depth {
            self.solution.clear();
            if self.ida_star(*rc, depth, 255) {
                let moves = self
                    .solution
                    .iter()
                    .map(|&m| Move::all_moves()[m])
                    .collect();
                return Some(moves);
            }
            if self.nodes_count >= self.max_nodes {
                break;
            }
        }
        None
    }

    fn ida_star(&mut self, rc: RawCube, depth: usize, last_face: u8) -> bool {
        self.nodes_count += 1;
        if self.nodes_count >= self.max_nodes {
            return false;
        }

        use super::coord::{get_group_a_idx, get_group_b_idx, get_group_c_idx, get_group_d_idx};
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

        use super::coord::move_cube_18;
        for face in 0..6 {
            if face == last_face as usize {
                continue;
            }
            // 対面の回転は、順序を固定することで重複を避ける（例：U D と D U は同じ）
            // U(0) < D(1), L(2) < R(3), F(4) < B(5)
            if face % 2 == 1 && (face - 1) == last_face as usize {
                continue;
            }

            for m_offset in 0..3 {
                let m = face * 3 + m_offset;
                let next_rc = rc.multiply(move_cube_18(m));

                self.solution.push(m);
                if self.ida_star(next_rc, depth - 1, face as u8) {
                    return true;
                }
                self.solution.pop();
            }
        }

        false
    }
}
