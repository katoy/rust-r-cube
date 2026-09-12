use cube_studio::{coord::RawCube, cube, initialize, solve_state};
fn main() {
    let start = std::time::Instant::now();
    initialize();
    println!(
        "initialization_ms={:.3}",
        start.elapsed().as_secs_f64() * 1000.0
    );
    let mut times = Vec::new();
    let mut lengths = Vec::new();
    for seed in 1..=1000 {
        let state = cube::facelets(&cube::apply(&RawCube::default(), &cube::scramble(seed)));
        let result = solve_state(&state, 5000, true).expect("must solve");
        times.push(result.elapsed_ms);
        lengths.push(result.moves.len());
    }
    times.sort_by(f64::total_cmp);
    lengths.sort();
    println!(
        "1000 states: p50={:.3}ms p95={:.3}ms max={:.3}ms; moves p50={} max={}",
        times[500], times[950], times[999], lengths[500], lengths[999]
    );
}
