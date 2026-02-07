use crate::cube::{Cube, Face, Move};
use crate::kociemba::{RawCube, Search};
use std::sync::OnceLock;

pub const DEFAULT_MAX_DEPTH: usize = 32;

// ランダム探索パラメータ
const RANDOM_TRIALS: usize = 3000;
const RANDOM_SEED: usize = 999888777;
const LCG_MULTIPLIER: usize = 1103515245;
const LCG_INCREMENT: usize = 12345;
const PROGRESS_WEIGHT: f32 = 0.9;
const MAX_SETUP_MOVES: usize = 6;
const TOTAL_BASIC_MOVES: usize = 18;
const TOTAL_ROTATIONS: usize = 24;

#[derive(Debug, Clone)]
pub struct Solution {
    pub moves: Vec<Move>,
    pub found: bool,
}

#[cfg(any(target_arch = "wasm32", test))]
pub struct SolverState {
    raw_cube: Result<RawCube, String>,
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
    if !cube.is_solved() {
        return false;
    }
    let oris = get_orientations_vec(cube);
    oris.iter().all(|&o| o == 0)
}

pub fn solve_with_progress(
    start_cube: &Cube,
    max_depth: usize,
    ignore_orientation: bool,
    progress_tx: Option<std::sync::mpsc::Sender<f32>>,
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
        let _ = tx.send(0.0);
    }
    if is_fully_solved(start_cube) {
        if let Some(ref tx) = progress_tx {
            let _ = tx.send(1.0);
        }
        return Solution {
            moves: vec![],
            found: true,
        };
    }
    if ignore_orientation && start_cube.is_solved() {
        if let Some(ref tx) = progress_tx {
            let _ = tx.send(1.0);
        }
        return Solution {
            moves: vec![],
            found: true,
        };
    }

    let mut search = Search::new();
    let rotations = get_all_rotations();
    let mut color_only_solution: Option<Solution> = None;

    // 1. 直近方位試行
    for rot in &rotations {
        let mut test_cube = start_cube.clone();
        for &m in rot {
            test_cube.apply_move(m);
        }
        if let Ok(rc) = RawCube::from_cube(&test_cube) {
            let color_solve_limit = max_depth;
            if let Some(m_fix) = search.solve(&rc, color_solve_limit.saturating_sub(rot.len())) {
                let mut moves = rot.clone();
                moves.extend(m_fix);
                let mut check_cube = start_cube.clone();
                for &m in &moves {
                    check_cube.apply_move(m);
                }
                if is_fully_solved(&check_cube) && moves.len() <= max_depth {
                    if let Some(ref tx) = progress_tx {
                        let _ = tx.send(1.0);
                    }
                    return Solution { moves, found: true };
                }
                if ignore_orientation && check_cube.is_solved() && moves.len() <= max_depth {
                    if let Some(ref tx) = progress_tx {
                        let _ = tx.send(1.0);
                    }
                    return Solution {
                        moves: moves.clone(),
                        found: true,
                    };
                }
                if check_cube.is_solved() {
                    let fixes = apply_supercube_fixes(&check_cube, &mut search);
                    let mut final_moves = moves.clone();
                    final_moves.extend(fixes.clone());
                    let mut final_cube = check_cube.clone();
                    for &m in &fixes {
                        final_cube.apply_move(m);
                    }
                    if is_fully_solved(&final_cube) {
                        if final_moves.len() <= max_depth {
                            if let Some(ref tx) = progress_tx {
                                let _ = tx.send(1.0);
                            }
                            return Solution {
                                moves: final_moves,
                                found: true,
                            };
                        }
                        if color_only_solution.is_none() {
                            color_only_solution = Some(Solution {
                                moves: final_moves,
                                found: true,
                            });
                        }
                    }
                } else if color_only_solution.is_none() {
                    color_only_solution = Some(Solution {
                        moves: moves.clone(),
                        found: true,
                    });
                }
            }
        }
    }

    if ignore_orientation {
        if let Some(sol) = color_only_solution {
            return sol;
        }
    }

    // 2. 超軽量・高密度試行 (2000試行)
    let all_moves = Move::all_moves();
    let mut seed: usize = RANDOM_SEED;
    let next_rn = |s: &mut usize| -> usize {
        *s = s.wrapping_mul(LCG_MULTIPLIER).wrapping_add(LCG_INCREMENT);
        (*s / 65536) % 32768
    };

    for trial_iter in 0..RANDOM_TRIALS {
        if let Some(ref tx) = progress_tx {
            let _ = tx.send(trial_iter as f32 / RANDOM_TRIALS as f32 * PROGRESS_WEIGHT);
        }
        let n_random = (next_rn(&mut seed) % MAX_SETUP_MOVES) + 1;
        let mut setup_moves = Vec::with_capacity(n_random);
        let mut trial_cube = start_cube.clone();
        for _ in 0..n_random {
            let m = all_moves[next_rn(&mut seed) % TOTAL_BASIC_MOVES];
            trial_cube.apply_move(m);
            setup_moves.push(m);
        }
        let rot = &rotations[next_rn(&mut seed) % TOTAL_ROTATIONS];
        if setup_moves.len() + rot.len() >= max_depth {
            continue;
        }
        let mut test_cube = trial_cube.clone();
        for &m in rot {
            test_cube.apply_move(m);
        }
        if let Ok(rc) = RawCube::from_cube(&test_cube) {
            let color_solve_limit = max_depth;
            let needed = setup_moves.len() + rot.len();
            if color_solve_limit > needed {
                if let Some(m_fix) = search.solve(&rc, color_solve_limit.saturating_sub(needed)) {
                    let mut base_moves = setup_moves.clone();
                    base_moves.extend(rot.clone());
                    base_moves.extend(m_fix);
                    let mut check_cube = start_cube.clone();
                    for &m in &base_moves {
                        check_cube.apply_move(m);
                    }
                    if is_fully_solved(&check_cube) && base_moves.len() <= max_depth {
                        if let Some(ref tx) = progress_tx {
                            let _ = tx.send(1.0);
                        }
                        return Solution {
                            moves: base_moves,
                            found: true,
                        };
                    }
                    if ignore_orientation && check_cube.is_solved() && base_moves.len() <= max_depth
                    {
                        if let Some(ref tx) = progress_tx {
                            let _ = tx.send(1.0);
                        }
                        return Solution {
                            moves: base_moves,
                            found: true,
                        };
                    }
                    if check_cube.is_solved() {
                        let fixes = apply_supercube_fixes(&check_cube, &mut search);
                        let mut final_moves = base_moves.clone();
                        final_moves.extend(fixes.clone());
                        let mut final_cube = check_cube.clone();
                        for &m in &fixes {
                            final_cube.apply_move(m);
                        }
                        if is_fully_solved(&final_cube) && final_moves.len() <= max_depth {
                            if let Some(ref tx) = progress_tx {
                                let _ = tx.send(1.0);
                            }
                            return Solution {
                                moves: final_moves,
                                found: true,
                            };
                        }
                    }
                }
            }
        }
    }
    if let Some(ref tx) = progress_tx {
        let _ = tx.send(1.0);
    }
    Solution {
        moves: vec![],
        found: false,
    }
}

pub fn get_orientations_vec(cube: &Cube) -> Vec<u8> {
    Face::all()
        .iter()
        .map(|f| {
            let start = f.start_index();
            let c_ori = cube.stickers[start + 4].orientation;
            let f_ori = cube.stickers[start].orientation;
            (c_ori + 4 - f_ori) % 4
        })
        .collect()
}

fn undo_setup(mut setup: Vec<Move>) -> Vec<Move> {
    for m in &mut setup {
        *m = m.inverse();
    }
    setup.reverse();
    setup
}

fn get_all_rotations() -> Vec<Vec<Move>> {
    vec![
        vec![],
        vec![Move::X],
        vec![Move::X2],
        vec![Move::Xp],
        vec![Move::Y],
        vec![Move::Y2],
        vec![Move::Yp],
        vec![Move::Z],
        vec![Move::Z2],
        vec![Move::Zp],
        vec![Move::X, Move::Y],
        vec![Move::X, Move::Y2],
        vec![Move::X, Move::Yp],
        vec![Move::X, Move::Z],
        vec![Move::X, Move::Z2],
        vec![Move::X, Move::Zp],
        vec![Move::Xp, Move::Y],
        vec![Move::Xp, Move::Y2],
        vec![Move::Xp, Move::Yp],
        vec![Move::Xp, Move::Z],
        vec![Move::Xp, Move::Z2],
        vec![Move::Xp, Move::Zp],
        vec![Move::X2, Move::Y],
        vec![Move::X2, Move::Z],
    ]
}

fn apply_supercube_fixes(cube: &Cube, _search: &mut Search) -> Vec<Move> {
    let mut current_cube = cube.clone();
    let mut final_moves = Vec::new();
    for _ in 0..10 {
        let oris = get_orientations_vec(&current_cube);
        if let Some(idx) = oris.iter().position(|&o| o == 2) {
            let f = Face::from_index(idx * 9);
            let fix = get_fix_180(f);
            for &m in &fix {
                current_cube.apply_move(m);
            }
            final_moves.extend(fix);
        } else {
            break;
        }
    }
    for _ in 0..6 {
        let oris = get_orientations_vec(&current_cube);
        let d90s: Vec<(usize, u8)> = oris
            .iter()
            .enumerate()
            .filter(|(_, &o)| o == 1 || o == 3)
            .map(|(i, &o)| (i, o))
            .collect();
        if d90s.len() >= 2 {
            let i1 = d90s[0].0;
            let o1 = d90s[0].1;
            let f1 = Face::from_index(i1 * 9);
            let mut i2_opt = None;
            for (j, &(i2, _)) in d90s.iter().enumerate().skip(1) {
                let f2 = Face::from_index(i2 * 9);
                if !is_opposite_face(f1, f2) {
                    i2_opt = Some(j);
                    break;
                }
            }
            if let Some(j) = i2_opt {
                let f2 = Face::from_index(d90s[j].0 * 9);
                let fix = if o1 == 1 {
                    get_rotation_and_seq_cw_ccw(f1, f2)
                } else {
                    get_rotation_and_seq_cw_ccw(f2, f1)
                };
                for &m in &fix {
                    current_cube.apply_move(m);
                }
                final_moves.extend(fix);
            } else {
                let f2 = Face::from_index(d90s[1].0 * 9);
                let buffer = get_buffer_face(f1, f2);
                let fix1 = if o1 == 1 {
                    get_rotation_and_seq_cw_ccw(f1, buffer)
                } else {
                    get_rotation_and_seq_cw_ccw(buffer, f1)
                };
                for &m in &fix1 {
                    current_cube.apply_move(m);
                }
                final_moves.extend(fix1);
            }
        } else {
            break;
        }
    }
    final_moves
}

fn is_opposite_face(f1: Face, f2: Face) -> bool {
    matches!(
        (f1, f2),
        (Face::Up, Face::Down)
            | (Face::Down, Face::Up)
            | (Face::Front, Face::Back)
            | (Face::Back, Face::Front)
            | (Face::Right, Face::Left)
            | (Face::Left, Face::Right)
    )
}

fn get_buffer_face(f1: Face, f2: Face) -> Face {
    for &f in &[
        Face::Up,
        Face::Down,
        Face::Front,
        Face::Back,
        Face::Right,
        Face::Left,
    ] {
        if !is_opposite_face(f1, f) && !is_opposite_face(f2, f) && f != f1 && f != f2 {
            return f;
        }
    }
    Face::Up
}

fn get_fix_180(face: Face) -> Vec<Move> {
    let rot = get_setup_to_up(face);
    let mut moves = rot.clone();
    let seq = vec![
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
    moves.extend(seq);
    moves.extend(undo_setup(rot));
    moves
}

fn get_rotation_and_seq_cw_ccw(f_up: Face, f_front: Face) -> Vec<Move> {
    let rot = get_setup_to_up_front(f_up, f_front);
    let mut moves = rot.clone();
    let seq = vec![
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
    moves.extend(seq);
    moves.extend(undo_setup(rot));
    moves
}

fn get_setup_to_up(face: Face) -> Vec<Move> {
    let base = Cube::new();
    let target = base.stickers[face.start_index() + 4].color;
    for rot in get_all_rotations() {
        let mut c = base.clone();
        for &m in &rot {
            c.apply_move(m);
        }
        if c.stickers[Face::Up.start_index() + 4].color == target {
            return rot;
        }
    }
    vec![]
}

fn get_setup_to_up_front(f_up: Face, f_front: Face) -> Vec<Move> {
    let base = Cube::new();
    let up_c = base.stickers[f_up.start_index() + 4].color;
    let front_c = base.stickers[f_front.start_index() + 4].color;
    for rot in get_all_rotations() {
        let mut c = base.clone();
        for &m in &rot {
            c.apply_move(m);
        }
        if c.stickers[Face::Up.start_index() + 4].color == up_c
            && c.stickers[Face::Front.start_index() + 4].color == front_c
        {
            return rot;
        }
    }
    vec![]
}

#[cfg(any(target_arch = "wasm32", test))]
impl SolverState {
    pub fn new(start_cube: &Cube, max_depth: usize, ignore_orientation: bool) -> Self {
        let rc = RawCube::from_cube(start_cube);
        Self {
            raw_cube: rc,
            initial_cube: start_cube.clone(),
            max_depth,
            ignore_orientation,
            solution: None,
            finished: false,
        }
    }
    pub fn error(&self) -> Option<String> {
        match &self.raw_cube {
            Ok(_) => None,
            Err(e) => Some(e.clone()),
        }
    }
    pub fn process_chunk(&mut self, _: usize) -> (usize, bool) {
        if self.finished {
            return (0, true);
        }
        let result = solve_internal(
            &self.initial_cube,
            self.max_depth,
            self.ignore_orientation,
            None,
        );
        self.solution = Some(result);
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_solve_center_orientation_90_pair() {
        let mut cube = Cube::new();
        cube.apply_move(Move::U);
        cube.apply_move(Move::Dp);
        cube.apply_move(Move::Ep);
        let sol = solve(&cube, 32, false);
        assert!(sol.found);
        let mut final_cube = cube.clone();
        for &mv in &sol.moves {
            final_cube.apply_move(mv);
        }
        assert!(is_fully_solved(&final_cube));
    }
    #[test]
    fn test_solve_normal_cube() {
        let mut cube = Cube::new();
        cube.apply_move(Move::U);
        cube.apply_move(Move::R);
        let sol = solve(&cube, 32, false);
        assert!(sol.found);
        let mut final_cube = cube.clone();
        for &mv in &sol.moves {
            final_cube.apply_move(mv);
        }
        assert!(is_fully_solved(&final_cube));
    }
    #[test]
    fn test_solve_scrambled_cube_full_completeness() {
        for i in 0..5 {
            let mut cube = Cube::new();
            cube.scramble(20 + i);
            let sol = solve(&cube, 64, false);
            assert!(sol.found, "Solution not found for scramble {}", i);
            let mut check_cube = cube.clone();
            for &mv in &sol.moves {
                check_cube.apply_move(mv);
            }
            assert!(
                is_fully_solved(&check_cube),
                "Scramble {} moves found but not fully solved: {:?}",
                i,
                sol.moves
            );
        }
    }
}
