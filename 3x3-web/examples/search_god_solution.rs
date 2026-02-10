use rubiks_cube_3x3::cube::Cube;
use rubiks_cube_3x3::solver::{solve, DEFAULT_MAX_DEPTH};
use std::fs;
use std::time::Instant;

fn main() {
    let cube_text = fs::read_to_string("cubes/cube_god.txt").unwrap();
    let cube = Cube::from_file_format(&cube_text).unwrap();

    println!("=== Searching for Color-Only Solution (ignore_orientation = true) ===");
    let start = Instant::now();
    // 探索の深さを20程度に制限してみる（God's Numberは20）
    let solution = solve(&cube, 22, true);
    let duration = start.elapsed();

    println!("Time: {:?}", duration);
    if solution.found {
        println!("Solution found! ({} moves)", solution.moves.len());
        println!("Moves: {:?}", solution.moves);

        let mut test_cube = cube.clone();
        for mv in &solution.moves {
            test_cube.apply_move(*mv);
        }
        println!("Is solved (colors): {}", test_cube.is_solved());
        println!(
            "Is solved (supercube): {}",
            test_cube.is_solved_with_orientation()
        );
    } else {
        println!("Solution not found within depth limit.");
        println!("Message: {}", solution.message);
    }
}
