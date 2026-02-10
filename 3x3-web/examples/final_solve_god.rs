use rubiks_cube_3x3::cube::Cube;
use rubiks_cube_3x3::kociemba::{RawCube, Search};
use std::fs;
use std::time::Instant;

fn main() {
    let cube_text = fs::read_to_string("cubes/cube_god.txt").expect("Failed to read cube_god.txt");
    let cube = Cube::from_file_format(&cube_text).expect("Failed to parse cube_god.txt");
    let rc = RawCube::from_cube(&cube).unwrap();

    println!("=== Final Search for God Cube Solution ===");
    println!("Node Limit: 1,000,000,000 (1 Billion)");

    let mut search = Search::new();
    search.max_nodes = 1_000_000_000;

    let start = Instant::now();
    let sol = search.solve(&rc, 22);
    let duration = start.elapsed();

    println!("Time: {:?}", duration);
    if let Some(s) = sol {
        println!("Solution found! Length: {}", s.len());
        println!("Moves: {:?}", s);

        // Apply moves to verify
        let mut test_cube = cube.clone();
        for &m in &s {
            test_cube.apply_move(m);
        }
        println!("Colors solved? {}", test_cube.is_solved());
    } else {
        println!("NOT FOUND within 1 Billion nodes.");
    }
}
