use rubiks_cube_3x3::cube::{Cube, Face, Move};
use std::collections::{HashMap, VecDeque};

fn main() {
    let mut visited = HashMap::new();
    let mut queue = VecDeque::new();

    let start = Cube::new();
    visited.insert(start.clone(), Vec::new());
    queue.push_back(start.clone());

    println!("Searching for pure center orientation procedures (Outer moves only)...");

    let mut count = 0;
    while let Some(current) = queue.pop_front() {
        count += 1;
        let path = visited.get(&current).unwrap().clone();

        if path.len() > 10 {
            continue;
        }

        if current.is_solved() {
            let mut mismatch = Vec::new();
            for f in Face::all() {
                let start_idx = f.start_index();
                if current.stickers[start_idx + 4].orientation
                    != current.stickers[start_idx].orientation
                {
                    mismatch.push(f);
                }
            }
            if !mismatch.is_empty() {
                println!("FOUND PURE OR FIX! Path: {:?}", path);
                for f in mismatch {
                    let start_idx = f.start_index();
                    println!(
                        "Face {:?}: diff={}",
                        f,
                        (current.stickers[start_idx + 4].orientation + 4
                            - current.stickers[start_idx].orientation)
                            % 4
                    );
                }
                if is_target(&current) {
                    return;
                }
            }
        }

        let outer_moves = [
            Move::U,
            Move::Up,
            Move::U2,
            Move::D,
            Move::Dp,
            Move::D2,
            Move::L,
            Move::Lp,
            Move::L2,
            Move::R,
            Move::Rp,
            Move::R2,
            Move::F,
            Move::Fp,
            Move::F2,
            Move::B,
            Move::Bp,
            Move::B2,
        ];

        for &mv in &outer_moves {
            // 逆転手順を避けるための簡単な枝刈り
            if let Some(&last) = path.last() {
                if mv.inverse() == last {
                    continue;
                }
            }

            let mut next = current.clone();
            next.apply_move(mv);
            if !visited.contains_key(&next) {
                let mut next_path = path.clone();
                next_path.push(mv);
                visited.insert(next.clone(), next_path);
                queue.push_back(next);
            }
        }

        if count % 100000 == 0 {
            println!("Checked {} states, depth {}", count, path.len());
        }
    }
}

fn is_target(cube: &Cube) -> bool {
    let mut d90s = 0;
    let mut d180s = 0;
    for f in Face::all() {
        let start_idx = f.start_index();
        let diff = (cube.stickers[start_idx + 4].orientation + 4
            - cube.stickers[start_idx].orientation)
            % 4;
        if diff == 1 || diff == 3 {
            d90s += 1;
        }
        if diff == 2 {
            d180s += 1;
        }
    }
    d90s == 2 && d180s == 0
}
