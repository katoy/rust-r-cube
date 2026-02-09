use rubiks_cube_3x3::cube::{Cube, Face, Move};

#[test]
fn test_comm_pattern() {
    let mut cube = Cube::new();
    // M E M' U M E' M' U'
    let moves = [
        Move::M,
        Move::E,
        Move::Mp,
        Move::U,
        Move::M,
        Move::Ep,
        Move::Mp,
        Move::Up,
    ];
    for &m in &moves {
        cube.apply_move(m);
    }
    assert!(cube.is_solved());
    for face in Face::all() {
        let start = face.start_index();
        println!(
            "Face {:?}: ori={}",
            face,
            cube.stickers[start + 4].orientation
        );
    }
}
