use rubiks_cube_3x3::cube::{Cube, Move};
use rubiks_cube_3x3::solver::{get_orientations_vec, get_solved_oris};

#[test]
fn test_solved_states_parity() {
    let solved_oris = get_solved_oris();
    println!("Number of solved states: {}", solved_oris.len());
    for (i, oris) in solved_oris.iter().enumerate() {
        let sum: u32 = oris.iter().map(|&o| o as u32).sum();
        println!("Pattern {}: {:?}, sum={}", i, oris, sum);
        assert!(
            sum % 2 == 0,
            "Solved state pattern {} has ODD parity! Sum={}",
            i,
            sum
        );
    }
}

#[test]
fn test_move_parity_change() {
    let base = Cube::new();
    let moves = Move::all_moves();
    for mv in moves {
        let mut c = base.clone();
        c.apply_move(mv);
        let oris = get_orientations_vec(&c);
        let sum: u32 = oris.iter().map(|&o| o as u32).sum();
        let name = format!("{:?}", mv);
        println!("Move {}: sum={}, parity={}", name, sum, sum % 2);
    }
}

#[test]
fn test_complex_legal_moves_parity() {
    let mut cube = Cube::new();
    // (M U M' U') * 3 -> Should change U and F orientations by 90/-90 (CW/CCW)
    let seq = [
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
    for &m in &seq {
        cube.apply_move(m);
    }
    let oris = get_orientations_vec(&cube);
    let sum: u32 = oris.iter().map(|&o| o as u32).sum();
    println!("Final oris: {:?}, sum={}", oris, sum);
    assert!(
        sum % 2 == 0,
        "Legal sequence must result in even parity sum"
    );
}

#[test]
fn test_search_odd_solved_state() {
    use rustc_hash::FxHashSet;
    use std::collections::VecDeque;

    let mut visited = FxHashSet::default();
    let mut queue = VecDeque::new();

    let base = Cube::new();
    queue.push_back(base.clone());
    visited.insert(base.clone());

    let moves = Move::all_moves();

    let mut count = 0;
    while let Some(current) = queue.pop_front() {
        count += 1;
        if count > 20000 {
            break;
        }

        if current.is_solved() {
            let oris = get_orientations_vec(&current);
            let sum: u32 = oris.iter().map(|&o| o as u32).sum();
            if sum % 2 != 0 {
                panic!("REACHED ODD SOLVED STATE! Oris={:?}, Sum={}", oris, sum);
            }
        }

        for &mv in &moves {
            let mut next = current.clone();
            next.apply_move(mv);
            if visited.insert(next.clone()) {
                queue.push_back(next);
            }
        }
    }
}
