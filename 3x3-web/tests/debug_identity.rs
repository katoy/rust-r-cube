use glam::{Mat4, Vec3};
use rubiks_cube_3x3::cube::piece::calculate_orientation_with_rot;

#[test]
fn debug_identity_orientation() {
    let faces = [
        ("U", Vec3::Y),
        ("D", -Vec3::Y),
        ("L", -Vec3::X),
        ("R", Vec3::X),
        ("F", Vec3::Z),
        ("B", -Vec3::Z),
    ];

    for (name, n) in faces {
        let ori = calculate_orientation_with_rot(n, n, Mat4::IDENTITY);
        println!("Face {}: normal={:?}, ori={}", name, n, ori);
    }
}
