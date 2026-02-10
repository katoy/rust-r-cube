use rubiks_cube_3x3::cube::{Cube, Face, Move};
use std::fs;

fn main() {
    let cube_text = fs::read_to_string("cubes/cube_god.txt").expect("Failed to read cube_god.txt");
    let target_cube = Cube::from_file_format(&cube_text).expect("Failed to parse cube_god.txt");

    println!("Testing (M' U')*3 variant for Superflip...");

    // Mp Up Mp Up Mp Up Mp Up flips 4 edges on the M ring and U/D layers?
    // Let's use the sequence from a reliable source for Superflip:
    // ((M' U) * 4  x y') * 3

    let mut cube = Cube::new();

    for _ in 0..3 {
        // (Mp U)*4
        for _ in 0..4 {
            cube.apply_move(Move::Mp);
            cube.apply_move(Move::U);
        }
        // x y'
        cube.apply_move(Move::X);
        cube.apply_move(Move::Yp);
    }

    println!("Applied ((Mp U)*4 x Yp)*3");

    let mut matches = true;
    for face in Face::all() {
        for idx in 0..9 {
            if cube.get_sticker(face.start_index() + idx).color
                != target_cube.get_sticker(face.start_index() + idx).color
            {
                matches = false;
            }
        }
    }

    if matches {
        println!("SUCCESS! Matches cube_god.txt exactly.");
    } else {
        println!("Mismatch. Corner orientation check:");
        let mut corners_ok = true;
        for face in Face::all() {
            let center = cube.get_sticker(face.start_index() + 4).color;
            for &idx in &[0, 2, 6, 8] {
                if cube.get_sticker(face.start_index() + idx).color != center {
                    corners_ok = false;
                }
            }
        }
        println!("Corners OK? {}", corners_ok);

        // try another rotation variant
        println!("Trying ((Mp Up)*4 x y')*3 ...");
        let mut cube2 = Cube::new();
        for _ in 0..3 {
            for _ in 0..4 {
                cube2.apply_move(Move::Mp);
                cube2.apply_move(Move::Up);
            }
            cube2.apply_move(Move::X);
            cube2.apply_move(Move::Yp);
        }

        let mut matches2 = true;
        for face in Face::all() {
            for idx in 0..9 {
                if cube2.get_sticker(face.start_index() + idx).color
                    != target_cube.get_sticker(face.start_index() + idx).color
                {
                    matches2 = false;
                }
            }
        }
        if matches2 {
            println!("SUCCESS (variant 2)! Matches cube_god.txt exactly.");
        }
    }
}
