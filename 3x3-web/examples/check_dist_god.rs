use rubiks_cube_3x3::cube::Cube;
use rubiks_cube_3x3::kociemba::{RawCube, Search};
use std::fs;

fn main() {
    let cube_text = fs::read_to_string("cubes/cube_god.txt").expect("Failed to read cube_god.txt");
    let cube = Cube::from_file_format(&cube_text).expect("Failed to parse cube_god.txt");

    let rc = RawCube::from_cube(&cube).expect("Failed to convert to RawCube");

    println!("Checking distance for cube_god.txt using pruning tables...");

    let twist = rc.get_twist();
    let flip = rc.get_flip();
    let slice = rc.get_ud_slice();

    // We need to access Search to use its pruning table logic, but Search::new() is private?
    // No, it's public.
    let search = Search::new();

    // Search has pruning tables. Let's see if we can get the distance.
    // The pruning tables are MoveTable and PruningTable.
    // In search.rs:
    // let d1 = self.pruning_table.twist_slice[twist as usize * 495 + slice as usize];
    // let d2 = self.pruning_table.flip_slice[flip as usize * 495 + slice as usize];
    // distance = d1.max(d2)

    // However, PruningTable is not public in kociemba::tables.
    // But Search::solve calls it.

    println!("Twist: {}, Flip: {}, Slice: {}", twist, flip, slice);

    // Since I can't access PruningTable directly easily without modifying the library,
    // I will try to solve it with depth 1, 2, 3... and see if it hits pruning.

    println!("Attempting to solve with Kociemba solver...");
    let mut search = Search::new();
    let sol = search.solve(&rc, 24);

    if let Some(s) = sol {
        println!("Solution found! Length: {}", s.len());
        println!("Moves: {:?}", s);
    } else {
        println!("Solution NOT found within Node Limit.");
    }
}
