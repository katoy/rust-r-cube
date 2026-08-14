use rubiks_cube_3x3::cube::{Cube, Face, Move};
fn main() {
    let mut cube = Cube::new();
    // (L R U2 Lp Rp U) x 2
    let moves = [
        Move::L,
        Move::R,
        Move::U2,
        Move::Lp,
        Move::Rp,
        Move::U,
        Move::L,
        Move::R,
        Move::U2,
        Move::Lp,
        Move::Rp,
        Move::U,
    ];
    for &m in &moves {
        cube.apply_move(m);
    }
    println!("Is solved (colors): {}", cube.is_solved());
    println!(
        "Up center ori: {}",
        cube.stickers[Face::Up.start_index() + 4].orientation
    );
    for face in Face::all() {
        let ori = cube.stickers[face.start_index() + 4].orientation;
        if ori != 0 {
            println!("Face {:?} rotated: {}", face, ori);
        }
    }
}
