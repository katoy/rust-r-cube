use cube_studio::{coord::RawCube, cube, supercube};
fn main() {
    let mode = std::env::args().nth(1).unwrap_or_default();
    if mode == "native-error" {
        let r = cube_studio::apply_moves(cube::SOLVED, "INVALID");
        println!("returned_error={}", r.is_err());
        return;
    }
    if mode == "korf-metadata" {
        let sc = cube::parse_moves("U R2 F B R B2 R U2 L B2 R U' D' R2 F R' L B2 U2 F2").unwrap();
        let state = cube::facelets(&cube::apply(&RawCube::default(), &sc));
        let r = cube_studio::solve_state_with_algorithm(&state, 1000, false, None, "korf").unwrap();
        println!("known_solution_upper_bound={} returned_moves={} algorithm={} phases={:?} elapsed_ms={}", sc.len(), r.moves.len(),r.algorithm,r.phases,r.elapsed_ms);
        return;
    }
    for centers in [
        [1, 0, 3, 0, 0, 0],
        [1, 0, 0, 3, 0, 0],
        [1, 1, 0, 0, 0, 0],
        [2, 0, 0, 0, 0, 0],
    ] {
        let fixes = supercube::solve_center_orientations(centers);
        println!(
            "centers={centers:?} correction_moves={} pieces_solved={}",
            fixes.len(),
            cube::apply(&RawCube::default(), &fixes) == RawCube::default()
        );
    }
    let mut failures = Vec::new();
    let mut lengths = Vec::new();
    let mut times = Vec::new();
    for seed in 1..=100 {
        let c = cube::apply(&RawCube::default(), &cube::scramble(seed));
        let now = std::time::Instant::now();
        match cube_studio::cfop::solve(&c, 5000) {
            Ok(r) => {
                assert_eq!(cube::apply(&c, &r.moves), RawCube::default());
                lengths.push(r.moves.len());
            }
            Err(e) => failures.push((seed, e)),
        }
        times.push(now.elapsed().as_secs_f64() * 1000.0);
    }
    lengths.sort();
    times.sort_by(f64::total_cmp);
    println!(
        "cfop100 failures={failures:?} moves_p50={} moves_range={:?} ms_p50={} ms_max={}",
        lengths[lengths.len() / 2],
        (lengths.first(), lengths.last()),
        times[50],
        times[99]
    );
}
