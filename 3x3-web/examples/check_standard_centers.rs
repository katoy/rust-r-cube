use rubiks_cube_3x3::cube::{Color, Cube, Face};

fn main() {
    let cube = Cube::new();
    println!("Standard Completed Cube Centers:");
    for face in Face::all() {
        let center_idx = face.start_index() + 4;
        let color = cube.get_sticker(center_idx).color;
        println!("{:?}: {:?}", face, color);
    }
}
