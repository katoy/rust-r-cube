// This validates the exact 72-move solution captured in the failing Playwright
// snapshot. It performs no search and does not depend on machine speed.
// rustc --edition=2021 --crate-name rust_superflip_fallback_probe -O \
//   docs/review-evidence/421345a/rust-superflip-fallback-probe.rs \
//   --extern cube_studio=target/release/deps/libcube_studio.rlib \
//   -L dependency=target/release/deps -o .rust-superflip-fallback-probe
// ./.rust-superflip-fallback-probe
// rm .rust-superflip-fallback-probe
use cube_studio::{coord::RawCube, cube, supercube};

fn main() {
    let state = "UBULURUFURURFRBRDRFUFLFRFDFDFDLDRDBDLULBLFLDLBUBRBLBDB";
    let initial_centers = [0, 2, 0, 3, 2, 3];
    let color_moves =
        cube::parse_moves("R L F U D' R2 F2 R F B' U B2 R2 D2 B2 U B2 U' F2 B2 U B2").unwrap();
    let center_moves = cube::parse_moves(
        "B L F' L' F' D F2 L' F' L' F L F' D' B' F L F U' F' U F U F2 L' F U F D F2 U' D' F U D F2 U' D' L U D L2 U' D' L U D L2 U' D'",
    )
    .unwrap();
    let input = cube::parse_state(state).unwrap();
    assert_eq!(cube::apply(&input, &color_moves), RawCube::default());
    let mut remaining = initial_centers;
    for &m in &color_moves {
        remaining[m / 3] = (remaining[m / 3] + (m % 3 + 1) as i32) % 4;
    }
    let generated_correction = supercube::solve_center_orientations(remaining).unwrap();
    assert_eq!(generated_correction, center_moves);
    let mut all_moves = color_moves.clone();
    all_moves.extend(&center_moves);
    assert_eq!(cube::apply(&input, &all_moves), RawCube::default());
    let mut final_centers = initial_centers;
    for &m in &all_moves {
        final_centers[m / 3] = (final_centers[m / 3] + (m % 3 + 1) as i32) % 4;
    }
    assert_eq!(final_centers, [0; 6]);
    println!("snapshot_input_state={state}");
    println!("initial_centers={initial_centers:?}");
    println!(
        "color_moves={} remaining_centers={remaining:?}",
        color_moves.len()
    );
    println!(
        "center_moves={} exact_current_correction_match=true",
        center_moves.len()
    );
    println!(
        "total_moves={} pieces_solved=true final_centers={final_centers:?}",
        all_moves.len()
    );
}
