use rubiks_cube_3x3::cube::Cube;
use rubiks_cube_3x3::solver::is_orientation_solvable;
use std::fs;

fn main() {
    let cube_text = fs::read_to_string("cubes/cube_god.txt").unwrap();
    let cube = Cube::from_file_format(&cube_text).unwrap();

    println!("Cube state check for cube_god.txt:");
    println!("is_solved(): {}", cube.is_solved());
    println!("is_valid_state(): {:?}", cube.is_valid_state());
    println!(
        "is_orientation_solvable(): {}",
        is_orientation_solvable(&cube)
    );
}
