use rubiks_cube_2x2::cube::{Cube, Move};
use rubiks_cube_2x2::solver::solve;

#[test]
fn test_solve_random_scramble_6_moves() {
    let mut cube = Cube::new();
    let moves = [Move::R, Move::U, Move::F, Move::L, Move::D, Move::B];
    for &mv in &moves {
        cube.apply_move(mv);
    }

    let solution = solve(&cube, 24, true);
    if !solution.found {
        println!("Cube state before failure:");
        for face in 0..6 {
            print!("Face {}: ", face);
            for i in 0..9 {
                print!("{:?} ", cube.stickers[face * 9 + i].color);
            }
            println!();
        }
    }
    assert!(
        solution.found,
        "Solution should be found for 6-move scramble"
    );

    // 見つかった解を適用する
    for &mv in &solution.moves {
        cube.apply_move(mv);
    }
    assert!(
        cube.is_solved(),
        "Cube should be solved after applying solution: {:?}",
        solution.moves
    );
}

#[test]
fn test_solve_random_scramble_10_moves() {
    let mut cube = Cube::new();
    let moves = [
        Move::U,
        Move::R,
        Move::F,
        Move::D,
        Move::L,
        Move::B,
        Move::U2,
        Move::R2,
        Move::F2,
        Move::D2,
    ];
    for &mv in &moves {
        cube.apply_move(mv);
    }

    let solution = solve(&cube, 24, true);
    assert!(
        solution.found,
        "Solution should be found for 10-move scramble"
    );

    for &mv in &solution.moves {
        cube.apply_move(mv);
    }
    assert!(
        cube.is_solved(),
        "Cube should be solved after applying solution: {:?}",
        solution.moves
    );
}

#[test]
fn test_solve_random_scramble_20_moves() {
    let mut cube = Cube::new();
    // 20手のランダムスクランブル
    let moves = [
        Move::U,
        Move::R,
        Move::F,
        Move::D,
        Move::L,
        Move::B,
        Move::U2,
        Move::R2,
        Move::F2,
        Move::D2,
        Move::L2,
        Move::B2,
        Move::Up,
        Move::Rp,
        Move::Fp,
        Move::Dp,
        Move::Lp,
        Move::Bp,
        Move::R,
        Move::U,
    ];
    for &mv in &moves {
        cube.apply_move(mv);
    }

    let solution = solve(&cube, 24, true);
    assert!(
        solution.found,
        "Solution should be found for 20-move scramble"
    );
    println!(
        "Found solution ({} moves): {:?}",
        solution.moves.len(),
        solution.moves
    );

    // 見つかった解を適用する
    for &mv in &solution.moves {
        cube.apply_move(mv);
    }
    assert!(
        cube.is_solved(),
        "Cube should be solved after applying solution: {:?}",
        solution.moves
    );
}

#[test]
fn test_solve_superflip() {
    // スーパーフリップ状態（エッジが全て反転しているが位置は正しい）
    // 公式手順: U R2 F B R B2 R U2 L B2 R U' D' R2 F R' L B2 U2 F2
    let mut cube = Cube::new();
    let superflip_moves = [
        Move::U,
        Move::R2,
        Move::F,
        Move::B,
        Move::R,
        Move::B2,
        Move::R,
        Move::U2,
        Move::L,
        Move::B2,
        Move::R,
        Move::Up,
        Move::Dp,
        Move::R2,
        Move::F,
        Move::Rp,
        Move::L,
        Move::B2,
        Move::U2,
        Move::F2,
    ];
    for &mv in &superflip_moves {
        cube.apply_move(mv);
    }

    let solution = solve(&cube, 24, true);
    assert!(solution.found, "Solution should be found for superflip");

    for &mv in &solution.moves {
        cube.apply_move(mv);
    }
    assert!(
        cube.is_solved(),
        "Cube should be solved after applying solution: {:?}",
        solution.moves
    );
}
