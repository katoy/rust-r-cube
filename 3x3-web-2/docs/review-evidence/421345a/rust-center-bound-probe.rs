// Run from 3x3-web-2 against the release library:
// rustc --edition=2021 --crate-name rust_center_bound_probe -O \
//   docs/review-evidence/421345a/rust-center-bound-probe.rs \
//   --extern cube_studio=target/release/deps/libcube_studio.rlib \
//   -L dependency=target/release/deps -o .rust-center-bound-probe
// ./.rust-center-bound-probe
// rm .rust-center-bound-probe
use cube_studio::{coord::RawCube, cube, solve_state_with_algorithm};

fn main() {
    let result =
        solve_state_with_algorithm(cube::SOLVED, 5000, true, Some([1; 6]), "cfop").unwrap();
    let moves = cube::parse_moves(&result.moves.join(" ")).unwrap();
    assert_eq!(cube::apply(&RawCube::default(), &moves), RawCube::default());
    let mut centers = [1; 6];
    for m in moves {
        centers[m / 3] = (centers[m / 3] + (m % 3 + 1) as i32) % 4;
    }
    assert_eq!(centers, [0; 6]);
    println!(
        "Public API SOLVED + centers=[1;6] cfop returned_moves={} final_state_solved={}",
        result.moves.len(),
        result.state == cube::SOLVED
    );
}
