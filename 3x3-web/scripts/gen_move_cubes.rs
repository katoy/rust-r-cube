use rubiks_cube_3x3::cube::{Cube, Move};
use rubiks_cube_3x3::kociemba::RawCube;

fn main() {
    let moves = [Move::U, Move::R, Move::F, Move::D, Move::L, Move::B];

    println!("pub static MOVE_CUBES: [RawCube; 6] = [");
    for mv in moves {
        let mut cube = Cube::new();
        cube.apply_move(mv);
        let rc = RawCube::from_cube(&cube).expect("from_cube failed for move");

        println!("    RawCube {{");
        print!("        cp: [");
        for (i, cp) in rc.cp.iter().enumerate() {
            print!("{}", cp);
            if i < 7 {
                print!(", ");
            }
        }
        println!("],");

        print!("        co: [");
        for (i, co) in rc.co.iter().enumerate() {
            print!("{}", co);
            if i < 7 {
                print!(", ");
            }
        }
        println!("],");

        print!("        ep: [");
        for (i, ep) in rc.ep.iter().enumerate() {
            print!("{}", ep);
            if i < 11 {
                print!(", ");
            }
        }
        println!("],");

        print!("        eo: [");
        for (i, eo) in rc.eo.iter().enumerate() {
            print!("{}", eo);
            if i < 11 {
                print!(", ");
            }
        }
        println!("],");
        println!("    }},");
    }
    println!("];");
}
