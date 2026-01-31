use rubiks_cube_3x3::cube::{Cube, NUM_STICKERS};

#[test]
fn dump_solved_orientations() {
    let cube = Cube::new();
    for i in 0..NUM_STICKERS {
        let s = cube.get_sticker(i);
        println!(
            "Sticker {}: face={}, color={:?}, ori={}",
            i,
            i / 9,
            s.color,
            s.orientation
        );
    }
}
