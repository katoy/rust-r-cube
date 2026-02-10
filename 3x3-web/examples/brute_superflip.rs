use rubiks_cube_3x3::cube::{Cube, Face, Move};
use rubiks_cube_3x3::solver::get_all_rotations;
use std::fs;

fn main() {
    let cube_text = fs::read_to_string("cubes/cube_god.txt").expect("Failed to read cube_god.txt");
    let target_cube = Cube::from_file_format(&cube_text).expect("Failed to parse cube_god.txt");

    println!("Brute-forcing Superflip orientation...");

    let rotations = get_all_rotations();

    for (i, rot) in rotations.iter().enumerate() {
        let mut cube = Cube::new();
        // Apply cube rotation
        for &m in rot {
            cube.apply_move(m);
        }

        // Apply Superflip generator sequence in THIS orientation
        // Base sequence: (M' U M' U M' U M' U x y') * 3
        // Note: x y' here are inner rotations, but let's just use the moves.
        // Actually, we can just apply (Mp U)*4, then rotate the whole cube x y', repeat.
        for _ in 0..3 {
            for _ in 0..4 {
                cube.apply_move(Move::Mp);
                cube.apply_move(Move::U);
            }
            cube.apply_move(Move::X);
            cube.apply_move(Move::Yp);
        }

        // check if it matches target
        let mut matches = true;
        for face in Face::all() {
            for idx in 0..9 {
                if cube.get_sticker(face.start_index() + idx).color
                    != target_cube.get_sticker(face.start_index() + idx).color
                {
                    matches = false;
                    break;
                }
            }
            if !matches {
                break;
            }
        }

        if matches {
            println!("MATCH FOUND at rotation index {}!", i);
            println!("Rotation moves: {:?}", rot);

            // Now we have the generator. The solution is the inverse.
            return;
        }
    }

    println!("No match found. Trying with (Mp Up)*4...");
    for (i, rot) in rotations.iter().enumerate() {
        let mut cube = Cube::new();
        for &m in rot {
            cube.apply_move(m);
        }
        for _ in 0..3 {
            for _ in 0..4 {
                cube.apply_move(Move::Mp);
                cube.apply_move(Move::Up);
            }
            cube.apply_move(Move::X);
            cube.apply_move(Move::Yp);
        }
        let mut matches = true;
        for face in Face::all() {
            for idx in 0..9 {
                if cube.get_sticker(face.start_index() + idx).color
                    != target_cube.get_sticker(face.start_index() + idx).color
                {
                    matches = false;
                    break;
                }
            }
            if !matches {
                break;
            }
        }
        if matches {
            println!("MATCH FOUND (variant 2) at rotation index {}!", i);
            return;
        }
    }
}
