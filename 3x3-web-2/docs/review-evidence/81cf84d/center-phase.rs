fn main() {
    let c = cube_studio::cube::apply(&cube_studio::coord::RawCube::default(), &[3]);
    let state = cube_studio::cube::facelets(&c);
    let r =
        cube_studio::solve_state_with_centers(&state, 0, true, Some([1, 0, 0, 0, 0, 0])).unwrap();
    println!("moves={} phases={:?}", r.moves.len(), r.phases);
}
