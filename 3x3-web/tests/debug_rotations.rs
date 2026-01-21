use rubiks_cube_2x2::cube::{Cube, Move};

#[test]
fn debug_x_rotation() {
    let mut cube = Cube::new();
    println!("Initial: solved={}", cube.is_solved());
    cube.apply_move(Move::X);
    println!("After X: solved={}", cube.is_solved());
    if !cube.is_solved() {
        for face in rubiks_cube_2x2::cube::Face::all() {
            println!(
                "Face {:?}: {:?}",
                face,
                (0..9)
                    .map(|i| cube.get_sticker(face.start_index() + i).color)
                    .collect::<Vec<_>>()
            );
        }
    }
    assert!(cube.is_solved());
}
