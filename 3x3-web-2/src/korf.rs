use crate::coord::{move_cube_18, RawCube};
use crate::search::Search;
use web_time::Instant;

pub struct KorfSearch {
    start: Instant,
    budget_ms: f64,
    pub nodes: u64,
    pub timed_out: bool,
    pub path: Vec<usize>,
}

impl KorfSearch {
    pub fn new(budget_ms: u32) -> Self {
        Self {
            start: Instant::now(),
            budget_ms: f64::from(budget_ms),
            nodes: 0,
            timed_out: false,
            path: Vec::with_capacity(32),
        }
    }

    fn exhausted(&mut self) -> bool {
        if self.timed_out {
            return true;
        }
        if self.nodes & 4095 == 0 && self.start.elapsed().as_secs_f64() * 1000.0 > self.budget_ms {
            self.timed_out = true;
            return true;
        }
        false
    }

    /// 数学的に厳密なアドミッシブル・ヒューリスティック関数（決して過大評価しない距離下界）
    fn heuristic(&self, cube: &RawCube) -> u8 {
        let bad_co = cube.co.iter().filter(|&&o| o != 0).count() as u8;
        let bad_eo = cube.eo.iter().filter(|&&o| o != 0).count() as u8;
        let bad_cp = (0..8).filter(|&i| cube.cp[i] as usize != i).count() as u8;
        let bad_ep = (0..12).filter(|&i| cube.ep[i] as usize != i).count() as u8;

        // 1手で最大4つのコーナー、4つのエッジしか直らない
        let h_co = (bad_co + 3) / 4;
        let h_eo = (bad_eo + 3) / 4;
        let h_cp = (bad_cp + 3) / 4;
        let h_ep = (bad_ep + 3) / 4;

        h_co.max(h_eo).max(h_cp).max(h_ep)
    }

    pub fn solve(&mut self, cube: &RawCube) -> Option<Vec<usize>> {
        if *cube == RawCube::default() {
            return Some(Vec::new());
        }

        // 深さ 1 から順に完全最短手を反復深化探索 (IDA*)
        let max_depth = 12; // ブラウザのレスポンス内で探索可能な深さ
        for depth in 1..=max_depth {
            self.path.clear();
            if self.search(cube, depth, 99) {
                return Some(self.path.clone());
            }
            if self.exhausted() {
                break;
            }
        }

        // 深さ制限または時間制限を超えた場合、Kociemba 2段階探索で確実に解を導出（準最短解フォールバック）
        let remaining_ms = (self.budget_ms - self.start.elapsed().as_secs_f64() * 1000.0).max(100.0) as u32;
        let mut fallback = Search::new(remaining_ms);
        let sol = fallback.solve(cube);
        self.nodes += fallback.nodes;
        sol
    }

    fn search(&mut self, cube: &RawCube, depth: u8, last_face: usize) -> bool {
        self.nodes += 1;
        if self.exhausted() {
            return false;
        }

        if depth == 0 {
            return *cube == RawCube::default();
        }

        let h = self.heuristic(cube);
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
                if self.search(&next_cube, depth - 1, face) {
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
        assert!(res.is_empty());
    }

    #[test]
    fn test_korf_short_scramble() {
        let cube = RawCube::default();
        let sc = parse_moves("R U R' F'").unwrap();
        let scrambled = apply(&cube, &sc);
        let mut korf = KorfSearch::new(3000);
        let sol = korf.solve(&scrambled).unwrap();
        assert_eq!(sol.len(), 4); // 完全最短手4手
        let final_cube = apply(&scrambled, &sol);
        assert_eq!(final_cube, RawCube::default());
    }

    #[test]
    fn test_korf_fallback_on_deep_scramble() {
        // 深さ12を超えるスクランブルではKociembaにフォールバックして解く
        let cube = RawCube::default();
        let sc = parse_moves("R U F B L D R U F B L D R").unwrap();
        let scrambled = apply(&cube, &sc);
        let mut korf = KorfSearch::new(3000);
        korf.timed_out = true; // タイムアウトフラグを直接立ててIDA*ループを即座にbreakさせフォールバックさせる
        let sol = korf.solve(&scrambled);
        assert!(sol.is_some());
    }

    #[test]
    fn test_korf_exhausted_and_redundant() {
        let mut korf = KorfSearch::new(0);
        // timed_out = false で nodes & 4095 == 0 かつ elapsed > budget
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
}

