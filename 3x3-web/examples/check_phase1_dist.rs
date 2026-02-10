use rubiks_cube_3x3::cube::Cube;
use rubiks_cube_3x3::kociemba::{RawCube, Search};
use std::fs;

fn main() {
    let cube_text = fs::read_to_string("cubes/cube_god.txt").expect("Failed to read cube_god.txt");
    let cube = Cube::from_file_format(&cube_text).expect("Failed to parse cube_god.txt");
    let rc = RawCube::from_cube(&cube).unwrap();

    let mut search = Search::new();

    let twist = rc.get_twist();
    let flip = rc.get_flip();
    let slice = rc.get_ud_slice();

    println!(
        "Initial Phase 1 Coords: twist={}, flip={}, slice={}",
        twist, flip, slice
    );

    // We want to know the distance.
    // Since we can't access PruningTable directly, we can use search_phase1 with increasing depth.
    for depth in 0..=15 {
        search.node_count = 0;
        search.max_nodes = 1_000_000;
        if search.search_phase1(twist, flip, slice, depth as u8, 99) {
            println!("Phase 1 Solution found at depth {}", depth);
            return;
        }
        println!(
            "Depth {}: No solution (Nodes: {})",
            depth, search.node_count
        );
    }
}
