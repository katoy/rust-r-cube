use crate::cube::{Cube, Face, Move};
use crate::kociemba::{RawCube, Search};
use std::sync::mpsc::Sender;
use std::sync::OnceLock;

pub const DEFAULT_MAX_DEPTH: usize = 24;

#[derive(Debug, Clone)]
pub struct Solution {
    pub moves: Vec<Move>,
    pub found: bool,
}

#[cfg(any(target_arch = "wasm32", test))]
pub struct SolverState {
    search: Search,
    raw_cube: RawCube,
    initial_cube: Cube,
    max_depth: usize,
    ignore_orientation: bool,
    solution: Option<Solution>,
    finished: bool,
}

static SOLVED_STATES: OnceLock<Vec<Cube>> = OnceLock::new();

pub fn get_solved_states() -> &'static [Cube] {
    SOLVED_STATES.get_or_init(generate_all_solved_states)
}

fn generate_all_solved_states() -> Vec<Cube> {
    use rustc_hash::FxHashSet;
    use std::collections::VecDeque;
    let base = Cube::new();
    let mut states = Vec::new();
    let mut queue = VecDeque::new();
    let mut visited: FxHashSet<Cube> = FxHashSet::default();

    queue.push_back(base.clone());
    visited.insert(base.clone());
    states.push(base);

    let rotations = vec![vec![Move::X], vec![Move::Y], vec![Move::Z]];
    while let Some(current) = queue.pop_front() {
        for rot_moves in &rotations {
            let mut next = current.clone();
            for &mv in rot_moves {
                next.apply_move(mv);
            }
            if visited.insert(next.clone()) {
                states.push(next.clone());
                queue.push_back(next);
            }
        }
    }
    states
}

pub fn is_fully_solved(cube: &Cube) -> bool {
    get_solved_states().contains(cube)
}

pub fn solve_with_progress(
    start_cube: &Cube,
    max_depth: usize,
    ignore_orientation: bool,
    progress_tx: Option<Sender<f32>>,
) -> Solution {
    solve_internal(start_cube, max_depth, ignore_orientation, progress_tx)
}

pub fn solve(start_cube: &Cube, max_depth: usize, ignore_orientation: bool) -> Solution {
    solve_internal(start_cube, max_depth, ignore_orientation, None)
}

fn solve_internal(
    start_cube: &Cube,
    max_depth: usize,
    ignore_orientation: bool,
    progress_tx: Option<std::sync::mpsc::Sender<f32>>,
) -> Solution {
    if let Some(ref tx) = progress_tx {
        let _ = tx.send(0.1);
    }
    let rc = match RawCube::from_cube(start_cube) {
        Ok(rc) => rc,
        Err(_) => {
            return Solution {
                moves: vec![],
                found: false,
            }
        }
    };
    if let Some(ref tx) = progress_tx {
        let _ = tx.send(0.4);
    }
    let mut search = Search::new();
    let moves = search.solve(&rc, max_depth);
    if let Some(ref tx) = progress_tx {
        let _ = tx.send(1.0);
    }

    match moves {
        Some(m) => {
            let mut final_moves = m;
            if !ignore_orientation {
                solve_supercube_orientations(start_cube, &mut final_moves, &mut search, max_depth);
            }
            Solution {
                moves: final_moves,
                found: true,
            }
        }
        None => Solution {
            moves: vec![],
            found: false,
        },
    }
}

#[cfg(any(target_arch = "wasm32", test))]
impl SolverState {
    pub fn new(start_cube: &Cube, max_depth: usize, ignore_orientation: bool) -> Self {
        let rc = RawCube::from_cube(start_cube).unwrap_or_default();
        Self {
            search: Search::new(),
            raw_cube: rc,
            initial_cube: start_cube.clone(),
            max_depth,
            ignore_orientation,
            solution: None,
            finished: false,
        }
    }
    pub fn process_chunk(&mut self, _: usize) -> (usize, bool) {
        let moves = self.search.solve(&self.raw_cube, self.max_depth);
        self.solution = Some(match moves {
            Some(m) => {
                let mut final_moves = m;
                if !self.ignore_orientation {
                    solve_supercube_orientations(
                        &self.initial_cube,
                        &mut final_moves,
                        &mut self.search,
                        self.max_depth,
                    );
                }
                Solution {
                    moves: final_moves,
                    found: true,
                }
            }
            None => Solution {
                moves: vec![],
                found: false,
            },
        });
        self.finished = true;
        (1, true)
    }
    pub fn get_solution(&self) -> Option<Solution> {
        self.solution.clone()
    }
    pub fn estimate_progress(&self) -> f32 {
        if self.finished {
            1.0
        } else {
            0.5
        }
    }
}

fn solve_supercube_orientations(
    start_cube: &Cube,
    final_moves: &mut Vec<Move>,
    search: &mut Search,
    max_depth: usize,
) {
    let mut cube = start_cube.clone();
    for &mv in &*final_moves {
        cube.apply_move(mv);
    }

    if is_fully_solved(&cube) {
        return;
    }

    // Phase 1: 180度補正
    let centers = [
        (Face::Up, 4),
        (Face::Down, 13),
        (Face::Left, 22),
        (Face::Right, 31),
        (Face::Front, 40),
        (Face::Back, 49),
    ];
    let mut applied_any = false;
    for (face, idx) in centers {
        let base_ori = cube.stickers[face.start_index()].orientation;
        let center_ori = cube.stickers[idx].orientation;
        if (center_ori + 4 - base_ori) % 4 == 2 {
            let seq = get_pure_180_move(face);
            for &m in &seq {
                cube.apply_move(m);
            }
            final_moves.extend(seq);
            applied_any = true;
        }
    }

    // Phase 2: 90度ペア補正 (1ペアずつ)
    for _ in 0..3 {
        let mut d90s = Vec::new();
        for (face, idx) in centers {
            let base_ori = cube.stickers[face.start_index()].orientation;
            let center_ori = cube.stickers[idx].orientation;
            let diff = (center_ori + 4 - base_ori) % 4;
            if diff == 1 || diff == 3 {
                d90s.push((face, diff));
            }
        }
        if d90s.len() < 2 {
            break;
        }

        let (f1, d1) = d90s[0];
        let (f2, _) = d90s[1];
        let seq = get_center_commutator_90_pair(f1, d1, f2);
        for &m in &seq {
            cube.apply_move(m);
        }
        final_moves.extend(seq);
        applied_any = true;
    }

    // Phase 3: 色修正
    if applied_any && !cube.is_solved() {
        if let Ok(rc) = RawCube::from_cube(&cube) {
            if let Some(m_fix) = search.solve(&rc, max_depth.max(20)) {
                for &mv in &m_fix {
                    cube.apply_move(mv);
                }
                final_moves.extend(m_fix);
            }
        }
    }

    // Phase 4: まだ 90 度ズレ（パリティ）がある場合、任意の面を 90 度回して再解決
    if !is_fully_solved(&cube) {
        let mut d90s = Vec::new();
        for (face, idx) in centers {
            let base_ori = cube.stickers[face.start_index()].orientation;
            let center_ori = cube.stickers[idx].orientation;
            let diff = (center_ori + 4 - base_ori) % 4;
            if diff != 0 {
                d90s.push((face, diff));
            }
        }
        if !d90s.is_empty() {
            let m = Move::U;
            cube.apply_move(m);
            final_moves.push(m);
            if let Ok(rc) = RawCube::from_cube(&cube) {
                if let Some(m_fix) = search.solve(&rc, 20) {
                    for &mv in &m_fix {
                        cube.apply_move(mv);
                    }
                    final_moves.extend(m_fix);
                }
            }
        }
    }

    // Final Phase: 残った 180 度を掃除
    for (face, idx) in centers {
        let base_ori = cube.stickers[face.start_index()].orientation;
        let center_ori = cube.stickers[idx].orientation;
        if (center_ori + 4 - base_ori) % 4 == 2 {
            let seq = get_pure_180_move(face);
            for &m in &seq {
                cube.apply_move(m);
            }
            final_moves.extend(seq);
        }
    }
}

fn get_pure_180_move(face: Face) -> Vec<Move> {
    let setup = get_setup_to_up(face);
    let mut seq = vec![
        Move::L,
        Move::R,
        Move::U2,
        Move::Lp,
        Move::Rp,
        Move::U,
        Move::L,
        Move::R,
        Move::U2,
        Move::Lp,
        Move::Rp,
        Move::U,
    ];
    let mut result = setup.clone();
    result.append(&mut seq);
    result.append(&mut undo_setup(setup));
    result
}

fn get_center_commutator_90_pair(f1: Face, d1: u8, f2: Face) -> Vec<Move> {
    let mut setup = get_setup_to_up(f1);
    let s2 = match f2 {
        Face::Front => vec![],
        Face::Back => vec![Move::Y2],
        Face::Left => vec![Move::Yp],
        Face::Right => vec![Move::Y],
        _ => vec![],
    };
    setup.extend(s2);
    let mut seq = vec![
        Move::Mp,
        Move::U,
        Move::M,
        Move::Up,
        Move::Mp,
        Move::U,
        Move::M,
        Move::Up,
        Move::Mp,
        Move::U,
        Move::M,
        Move::Up,
    ];
    if d1 == 1 {
        let mut inv_seq = Vec::new();
        for m in seq.iter().rev() {
            inv_seq.push(m.inverse());
        }
        seq = inv_seq;
    }
    let mut result = setup.clone();
    result.append(&mut seq);
    result.append(&mut undo_setup(setup));
    result
}

fn get_setup_to_up(face: Face) -> Vec<Move> {
    match face {
        Face::Up => vec![],
        Face::Down => vec![Move::X2],
        Face::Left => vec![Move::Z],
        Face::Right => vec![Move::Zp],
        Face::Front => vec![Move::X],
        Face::Back => vec![Move::Xp],
    }
}

fn undo_setup(mut setup: Vec<Move>) -> Vec<Move> {
    for m in &mut setup {
        *m = m.inverse();
    }
    setup.reverse();
    setup
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_solve_center_orientation_180() {
        let mut cube = Cube::new();
        let seq = get_pure_180_move(Face::Up);
        for &m in &seq {
            cube.apply_move(m);
        }
        assert_eq!(cube.stickers[4].orientation, 2);
        let sol = solve(&cube, 24, false);
        let mut final_cube = cube.clone();
        for &mv in &sol.moves {
            final_cube.apply_move(mv);
        }
        assert!(is_fully_solved(&final_cube));
    }

    #[test]
    fn test_solve_center_orientation_90_pair() {
        let mut cube = Cube::new();
        let seq = get_center_commutator_90_pair(Face::Up, 1, Face::Front);
        for &m in &seq {
            cube.apply_move(m);
        }
        let sol = solve(&cube, 24, false);
        assert!(sol.found);
        let mut final_cube = cube.clone();
        for &mv in &sol.moves {
            final_cube.apply_move(mv);
        }
        if !is_fully_solved(&final_cube) {
            println!("90_pair test failed output:");
            for face in Face::all() {
                let start = face.start_index();
                println!(
                    "Face {:?}: color={:?}, center_ori={}, corner_ori={}",
                    face,
                    final_cube.stickers[start + 4].color,
                    final_cube.stickers[start + 4].orientation,
                    final_cube.stickers[start].orientation
                );
            }
        }
        assert!(is_fully_solved(&final_cube));
    }

    #[test]
    fn test_solve_center_orientation_complex() {
        let mut cube = Cube::new();
        let m180 = get_pure_180_move(Face::Up);
        for &m in &m180 {
            cube.apply_move(m);
        }
        let m90pair = get_center_commutator_90_pair(Face::Left, 1, Face::Right);
        for &m in &m90pair {
            cube.apply_move(m);
        }
        let sol = solve(&cube, 24, false);
        assert!(sol.found);
        let mut final_cube = cube.clone();
        for &mv in &sol.moves {
            final_cube.apply_move(mv);
        }
        if !is_fully_solved(&final_cube) {
            println!("Complex test failed output:");
            for face in Face::all() {
                let start = face.start_index();
                println!(
                    "Face {:?}: color={:?}, center_ori={}, corner_ori={}",
                    face,
                    final_cube.stickers[start + 4].color,
                    final_cube.stickers[start + 4].orientation,
                    final_cube.stickers[start].orientation
                );
            }
        }
        assert!(is_fully_solved(&final_cube));
        assert!(sol.moves.len() < 100);
    }
}
