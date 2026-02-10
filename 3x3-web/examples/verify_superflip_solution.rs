use rubiks_cube_3x3::cube::{Cube, Move};
use std::fs;

fn main() {
    let cube_text = fs::read_to_string("cubes/cube_god.txt").expect("Failed to read cube_god.txt");
    let target_cube = Cube::from_file_format(&cube_text).expect("Failed to parse cube_god.txt");

    println!("Verifying Superflip sequence...");

    // many sources say: (M' U')*4 + (rotate) + ... is superflip
    // Standard 20-move superflip sequence (Singmaster's):
    // U R2 F B R B2 R U2 L B2 R U' D' R2 F R' L B2 U2 F2
    let moves = vec![
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
        Move::D,
        Move::R2,
        Move::F,
        Move::Rp,
        Move::L,
        Move::B2,
        Move::U2,
        Move::F2,
    ];

    let mut cube = Cube::new();
    // Superflip depends on having centers right. Cube::new() has standard centers.
    for &mv in &moves {
        cube.apply_move(mv);
    }

    println!("Applied Superflip sequence to solved cube.");
    println!("Is solved (colors)? {}", cube.is_solved());

    // Check if it matches target_cube
    let mut matches = true;
    for face in rubiks_cube_3x3::cube::Face::all() {
        for idx in 0..9 {
            if cube.get_sticker(face.start_index() + idx).color
                != target_cube.get_sticker(face.start_index() + idx).color
            {
                matches = false;
                // println!("Mismatch at {:?} index {}: Expected {:?}, Found {:?}", face, idx, target_cube.get_sticker(face.start_index() + idx).color, cube.get_sticker(face.start_index() + idx).color);
            }
        }
    }

    println!("Does it match cube_god.txt exactly? {}", matches);

    if matches {
        println!("SUCCESS! The sequence is a solution (in reverse).");
        let solution_moves: Vec<Move> = moves.iter().rev().map(|m| m.inverse()).collect();
        println!("Solution moves: {:?}", solution_moves);
    } else {
        println!("Trying another variant...");
        // try M' U M' U M' U M' U x y ...
    }
}
