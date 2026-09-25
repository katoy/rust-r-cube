use cube_studio::{coord::RawCube, cube, solve_state_with_algorithm, PhaseInfo, ResultData};
use std::time::Instant;

fn measure_result(
    cube: &cube_studio::coord::RawCube,
    state: &str,
    moves: &[usize],
    elapsed_ms: f64,
    nodes: u64,
    algorithm: &str,
    phases: Vec<PhaseInfo>,
) -> ResultData {
    let mut current_cube = *cube;
    let mut states = Vec::with_capacity(moves.len() + 1);
    states.push(state.to_owned());
    for m in moves {
        current_cube = cube::apply(&current_cube, &[*m]);
        states.push(cube::facelets(&current_cube));
    }
    let final_state = states.last().cloned().unwrap_or_else(|| state.to_owned());
    ResultData {
        state: final_state,
        moves: moves.iter().map(|m| cube::notation(*m)).collect(),
        states,
        elapsed_ms,
        nodes,
        algorithm: algorithm.to_string(),
        phases,
    }
}

fn main() {
    cube_studio::initialize();

    let scramble_20 = cube::scramble(42);
    let state = cube::facelets(&cube::apply(&RawCube::default(), &scramble_20));

    println!("=== Serialization & ResultData Allocation Benchmark ===");

    for alg in &["kociemba", "cfop", "thistlethwaite"] {
        let res = solve_state_with_algorithm(&state, 5000, true, None, alg).unwrap();
        let moves_count = res.moves.len();
        let json_str = serde_json::to_string(&res).unwrap();
        let json_bytes = json_str.len();

        let raw_cube = cube::parse_state(&state).unwrap();
        let move_indices = cube::parse_moves(&res.moves.join(" ")).unwrap();

        // 10,000回の ResultData 生成
        let start = Instant::now();
        let iters = 10_000;
        for _ in 0..iters {
            let r = measure_result(&raw_cube, &state, &move_indices, 0.0, 0, alg, Vec::new());
            std::hint::black_box(r);
        }
        let result_gen_us = start.elapsed().as_micros() as f64 / iters as f64;

        // 10,000回の serde_json::to_string
        let start = Instant::now();
        for _ in 0..iters {
            let s = serde_json::to_string(&res).unwrap();
            std::hint::black_box(s);
        }
        let json_us = start.elapsed().as_micros() as f64 / iters as f64;

        let total_overhead_us = result_gen_us + json_us;
        let solver_time_us = res.elapsed_ms * 1000.0;
        let overhead_pct = (total_overhead_us / solver_time_us) * 100.0;

        println!(
            "[{}] moves: {:>3} | size: {:>5} B | result_gen: {:>5.2} µs | to_string: {:>5.2} µs | total overhead: {:>5.2} µs ({:.3}% of solver)",
            alg,
            moves_count,
            json_bytes,
            result_gen_us,
            json_us,
            total_overhead_us,
            overhead_pct
        );
    }
}
