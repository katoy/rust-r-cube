use super::coord::RawCube;
use super::tables::{MoveTable, PruningTable};
use crate::cube::Move;

pub const DEFAULT_MAX_NODES: usize = 10_000_000;

pub struct Search {
    pub move_table: &'static MoveTable,
    pub pruning_table: &'static PruningTable,
    pub max_nodes: usize,
    nodes_count: usize,
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
        let cp = rc.get_cp();
        let twist = rc.get_twist();

        self.nodes_count = 0;
        for depth in 0..=max_depth {
            self.solution.clear();
            if self.ida_star(cp, twist, depth, 255) {
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

    fn ida_star(&mut self, cp: u16, twist: u16, depth: usize, last_face: u8) -> bool {
        self.nodes_count += 1;
        if self.nodes_count >= self.max_nodes {
            return false;
        }

        let h_cp = self.pruning_table.cp[cp as usize];
        let h_twist = self.pruning_table.twist[twist as usize];
        let h = h_cp.max(h_twist) as usize;

        if h == 0 {
            return true;
        }
        if h > depth {
            return false;
        }

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
                let next_cp = self.move_table.cp[cp as usize][m];
                let next_twist = self.move_table.twist[twist as usize][m];

                self.solution.push(m);
                if self.ida_star(next_cp, next_twist, depth - 1, face as u8) {
                    return true;
                }
                self.solution.pop();
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_default() {
        let _ = Search::default();
    }
}
