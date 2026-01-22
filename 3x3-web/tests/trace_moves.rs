use rubiks_cube_2x2::cube::{Cube, Move};
use rubiks_cube_2x2::kociemba::RawCube;

#[test]
fn test_trace_moves_step_by_step() {
    let mut cube = Cube::new();
    let moves = [Move::U, Move::R];

    println!("Initial state:");
    assert!(RawCube::from_cube(&cube).is_ok());

    println!("Applying Move::U...");
    cube.apply_move(Move::U);
    match RawCube::from_cube(&cube) {
        Ok(_) => println!("U is consistent"),
        Err(e) => panic!("U corrupted the cube: {}", e),
    }

    println!("Applying Move::R...");
    cube.apply_move(Move::R);
    match RawCube::from_cube(&cube) {
        Ok(_) => println!("R is consistent"),
        Err(e) => {
            println!("Cube state after U, R:");
            for face in 0..6 {
                print!("Face {}: ", face);
                for i in 0..9 {
                    print!("{:?} ", cube.stickers[face * 9 + i].color);
                }
                println!();
            }
            panic!("R corrupted the cube after U: {}", e);
        }
    }
}
