use rubiks_cube_3x3::cube::{Cube, Face};
use std::fs;

fn main() {
    let cube_text = fs::read_to_string("cubes/cube_god.txt").expect("Failed to read cube_god.txt");
    let cube = Cube::from_file_format(&cube_text).expect("Failed to parse cube_god.txt");

    println!("Checking if cube_god.txt is Superflip...");

    let centers = [
        cube.get_sticker(Face::Up.start_index() + 4).color,
        cube.get_sticker(Face::Down.start_index() + 4).color,
        cube.get_sticker(Face::Left.start_index() + 4).color,
        cube.get_sticker(Face::Right.start_index() + 4).color,
        cube.get_sticker(Face::Front.start_index() + 4).color,
        cube.get_sticker(Face::Back.start_index() + 4).color,
    ];

    println!("Centers (U,D,L,R,F,B): {:?}", centers);

    let mut corners_correct = true;
    for face in Face::all() {
        let center_color = cube.get_sticker(face.start_index() + 4).color;
        for &idx in &[0, 2, 6, 8] {
            let sticker_color = cube.get_sticker(face.start_index() + idx).color;
            if sticker_color != center_color {
                corners_correct = false;
                println!(
                    "Corner mismatch at {:?} index {}: {:?}",
                    face, idx, sticker_color
                );
            }
        }
    }

    println!(
        "All corners correct (color matches center): {}",
        corners_correct
    );

    let mut edges_flipped = true;
    for face in Face::all() {
        let center_color = cube.get_sticker(face.start_index() + 4).color;
        for &idx in &[1, 3, 5, 7] {
            let sticker_color = cube.get_sticker(face.start_index() + idx).color;
            if sticker_color == center_color {
                edges_flipped = false;
                println!(
                    "Edge matches center at {:?} index {}: {:?}",
                    face, idx, center_color
                );
            }
        }
    }

    println!(
        "All edge stickers MISMATCH their centers (flipped hint): {}",
        edges_flipped
    );
}
