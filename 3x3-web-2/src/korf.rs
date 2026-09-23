use crate::coord::{move_cube_18, RawCube};
use crate::search::Search;
use crate::tables::PruningTable;
use web_time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KorfSolution {
    pub moves: Vec<usize>,
    pub is_optimal: bool,
}

pub struct KorfSearch {
    start: Instant,
    budget_ms: f64,
    main_budget_ms: f64,
    pub nodes: u64,
    pub timed_out: bool,
    path: Vec<usize>,
}

impl KorfSearch {
    pub fn new(budget_ms: u32) -> Self {
        let b = f64::from(budget_ms);
        let main_b = (b * 0.7).min((b - 50.0).max(0.0));
        Self {
            start: Instant::now(),
            budget_ms: b,
            main_budget_ms: main_b,
            nodes: 0,
            timed_out: false,
            path: Vec::with_capacity(32),
        }
    }

    fn exhausted(&mut self) -> bool {
        if self.timed_out {
            return true;
        }
        if self.budget_ms == 0.0
            || (self.nodes & 4095 == 0
                && self.start.elapsed().as_secs_f64() * 1000.0 >= self.main_budget_ms)
        {
            self.timed_out = true;
            return true;
        }
        false
    }

    /// 数学的に厳密なアドミッシブル・ヒューリスティック関数（決して過大評価しない距離下界）
    ///
    /// Kociemba Phase 1 の目標群 G1 は完成状態 (Solved) を内包するため、
    /// 完成状態に至る任意の手順は G1 への到達を内包する。
    /// したがって、Twist/Slice および Flip/Slice 枝刈りテーブルによる下界値は、
    /// 完成状態までの最短手数の真の下界（アドミッシブル）として 100% 有効である。
    fn heuristic(&self, cube: &RawCube, prun: &PruningTable) -> u8 {
        let twist = cube.get_twist() as usize;
        let flip = cube.get_flip() as usize;
        let slice = cube.get_ud_slice() as usize;

        let h_ts = prun.get_twist_slice(twist, slice);
        let h_fs = prun.get_flip_slice(flip, slice);

        let bad_cp = (0..8).filter(|&i| cube.cp[i] as usize != i).count() as u8;
        let bad_ep = (0..12).filter(|&i| cube.ep[i] as usize != i).count() as u8;

        let h_cp = bad_cp.div_ceil(4);
        let h_ep = bad_ep.div_ceil(4);

        h_ts.max(h_fs).max(h_cp).max(h_ep)
    }

    pub fn solve(&mut self, cube: &RawCube) -> Option<KorfSolution> {
        if *cube == RawCube::default() {
            return Some(KorfSolution {
                moves: Vec::new(),
                is_optimal: true,
            });
        }

        let prun = PruningTable::get();

        // 深さ 1 から順に完全最短手を反復深化探索 (IDA*)
        let max_depth = 12; // ブラウザのレスポンス内で探索可能な深さ
        for depth in 1..=max_depth {
            self.path.clear();
            if self.search(cube, depth, 99, prun) {
                return Some(KorfSolution {
                    moves: self.path.clone(),
                    is_optimal: true,
                });
            }
            if self.exhausted() {
                break;
            }
        }

        // 深さ制限または時間制限を超えた場合、Kociemba 2段階探索で確実に解を導出（残余予算内）
        let elapsed_ms = self.start.elapsed().as_millis() as u32;
        let remaining_ms = (self.budget_ms as u32).saturating_sub(elapsed_ms);
        if remaining_ms == 0 {
            return None;
        }
        let mut fallback = Search::new(remaining_ms);
        let sol = fallback.solve(cube)?;
        self.nodes += fallback.nodes;
        Some(KorfSolution {
            moves: sol,
            is_optimal: false,
        })
    }

    fn search(&mut self, cube: &RawCube, depth: u8, last_face: usize, prun: &PruningTable) -> bool {
        self.nodes += 1;
        if self.exhausted() {
            return false;
        }

        if depth == 0 {
            return *cube == RawCube::default();
        }

        let h = self.heuristic(cube, prun);
        if h > depth {
            return false;
        }

        for face in 0..6 {
            if redundant(face, last_face) {
                continue;
            }
            for turn in 0..3 {
                let m = face * 3 + turn;
                let next_cube = cube.multiply(move_cube_18(m));
                self.path.push(m);
                if self.search(&next_cube, depth - 1, face, prun) {
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
}

fn redundant(face: usize, last: usize) -> bool {
    face == last || ((3..6).contains(&last) && face + 3 == last)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cube::{apply, parse_moves};

    #[test]
    fn test_korf_solved() {
        let cube = RawCube::default();
        let mut korf = KorfSearch::new(1000);
        let res = korf.solve(&cube).unwrap();
        assert!(res.moves.is_empty());
        assert!(res.is_optimal);
    }

    #[test]
    fn test_korf_short_scramble() {
        let cube = RawCube::default();
        let sc = parse_moves("R U R' F'").unwrap();
        let scrambled = apply(&cube, &sc);
        let mut korf = KorfSearch::new(3000);
        let sol = korf.solve(&scrambled).unwrap();
        assert_eq!(sol.moves.len(), 4); // 完全最短手4手
        assert!(sol.is_optimal);
        let final_cube = apply(&scrambled, &sol.moves);
        assert_eq!(final_cube, RawCube::default());
    }

    #[test]
    fn test_korf_medium_scramble_optimal() {
        // 深さ6〜7手のスクランブルも PDB 枝刈りにより IDA* で瞬時に最短解を導出
        let cube = RawCube::default();
        let sc = parse_moves("R U2 R' D F2 D'").unwrap();
        let scrambled = apply(&cube, &sc);
        let mut korf = KorfSearch::new(3000);
        let sol = korf.solve(&scrambled).unwrap();
        assert_eq!(sol.moves.len(), 6);
        assert!(sol.is_optimal);
        let final_cube = apply(&scrambled, &sol.moves);
        assert_eq!(final_cube, RawCube::default());
    }

    #[test]
    fn test_korf_timeout_in_search() {
        let cube = RawCube::default();
        let mut korf = KorfSearch::new(1);
        korf.nodes = 4094;
        korf.start = Instant::now() - std::time::Duration::from_secs(1);
        let prun = PruningTable::get();
        assert!(!korf.search(&cube, 2, 99, prun));
        assert!(korf.timed_out);
    }

    #[test]
    fn test_korf_fallback_on_deep_scramble() {
        // 深さ12を超えるスクランブルではKociembaにフォールバックして解く
        let cube = RawCube::default();
        let sc = parse_moves("R U F B L D R U F B L D R").unwrap();
        let scrambled = apply(&cube, &sc);
        let mut korf = KorfSearch::new(30_000);
        korf.start = Instant::now();
        korf.timed_out = true; // タイムアウトフラグを直接立ててIDA*ループを即座にbreakさせフォールバックさせる
        let sol = korf.solve(&scrambled).unwrap();
        assert!(!sol.moves.is_empty());
        assert!(!sol.is_optimal);
    }

    #[test]
    fn test_korf_exhausted_and_redundant() {
        let mut korf = KorfSearch::new(0);
        korf.start = Instant::now() - std::time::Duration::from_millis(10);
        korf.nodes = 4096;
        assert!(korf.exhausted());
        // すでに timed_out = true の場合
        assert!(korf.exhausted());

        // redundant の全条件
        assert!(redundant(0, 0)); // face == last
        assert!(redundant(0, 3)); // 0 + 3 == 3 (UとD)
        assert!(redundant(1, 4)); // 1 + 3 == 4 (RとL)
        assert!(redundant(2, 5)); // 2 + 3 == 5 (FとB)
        assert!(!redundant(0, 1)); // UとRは非冗長
        assert!(!redundant(3, 0)); // DとU
    }

    #[test]
    fn test_korf_zero_budget_returns_none() {
        let cube = RawCube::default();
        let sc = parse_moves("R U F").unwrap();
        let scrambled = apply(&cube, &sc);
        let mut korf = KorfSearch::new(0);
        assert!(korf.solve(&scrambled).is_none());
    }
}
