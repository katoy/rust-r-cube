use cube_studio::{coord::RawCube, cube, cfop, initialize, supercube};
use std::time::Instant;

fn main() {
    initialize();
    let mut rejected = 0;
    let mut accepted = 0;
    let mut incorrect = 0;
    let mut max_ms = 0.0f64;
    for seed in 1..=1000 {
        let input = cube::apply(&RawCube::default(), &cube::scramble(seed));
        let start = Instant::now();
        match cfop::solve(&input, 1000) {
            Ok(result) => {
                accepted += 1;
                if cube::apply(&input, &result.moves) != RawCube::default() {
                    incorrect += 1;
                    println!("INCORRECT seed={seed}");
                }
            }
            Err(error) => {
                rejected += 1;
                println!("REJECTED seed={seed} elapsed_ms={:.3} state={} error={error}", start.elapsed().as_secs_f64()*1000.0, cube::facelets(&input));
            }
        }
        max_ms = max_ms.max(start.elapsed().as_secs_f64()*1000.0);
        if seed % 100 == 0 { println!("progress={seed} accepted={accepted} rejected={rejected}"); }
    }
    println!("CFOP accepted={accepted} rejected={rejected} incorrect={incorrect} max_ms={max_ms:.3}");

    let mut legal = 0;
    let mut illegal = 0;
    for code in 0u32..4096 {
        let centers = std::array::from_fn(|i| ((code >> (2*i)) & 3) as i32);
        let result = supercube::solve_center_orientations(centers);
        if centers.iter().sum::<i32>() % 2 != 0 {
            assert!(result.is_err());
            illegal += 1;
            continue;
        }
        let moves = result.unwrap();
        assert_eq!(cube::apply(&RawCube::default(), &moves), RawCube::default());
        let mut after = centers;
        for m in moves { after[m/3] = (after[m/3] + (m%3+1) as i32) % 4; }
        assert_eq!(after, [0;6]);
        legal += 1;
    }
    println!("CENTER_EXHAUSTIVE legal_solved={legal} illegal_rejected={illegal}");
}
