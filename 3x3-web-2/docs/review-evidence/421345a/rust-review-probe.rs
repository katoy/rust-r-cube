extern crate cube_studio;
use cube_studio::{coord::RawCube, cube};
fn main() {
    cube_studio::initialize();
    let mut count = 0;
    for eo in 0..8 {
        for co in 0..27 {
            let mut c = RawCube::default();
            let mut e = eo;
            let mut parity = 0;
            for i in 0..3 {
                c.eo[i] = (e % 2) as u8;
                parity += c.eo[i];
                e /= 2;
            }
            c.eo[3] = parity % 2;
            let mut t = co;
            let mut sum = 0;
            for i in 0..3 {
                c.co[i] = (t % 3) as u8;
                sum += c.co[i];
                t /= 3;
            }
            c.co[3] = (3 - sum % 3) % 3;
            match cube_studio::cfop::solve(&c, 1000) {
                Ok(r) => assert_eq!(cube::apply(&c, &r.moves), RawCube::default()),
                Err(e) => {
                    println!(
                        "CFOP_FAIL eo={eo} co={co} state={} err={e}",
                        cube::facelets(&c)
                    );
                    return;
                }
            }
            count += 1;
        }
    }
    println!("CFOP last-layer orientation samples passed={count}");
    let mut max = (0, [0; 6]);
    for code in 0..4096 {
        let mut n = code;
        let mut centers = [0; 6];
        for c in &mut centers {
            *c = n % 4;
            n /= 4;
        }
        if centers.iter().sum::<i32>() % 2 != 0 {
            continue;
        }
        let moves = cube_studio::supercube::solve_center_orientations(centers).unwrap();
        assert_eq!(cube::apply(&RawCube::default(), &moves), RawCube::default());
        let mut final_c = centers;
        for m in &moves {
            final_c[m / 3] = (final_c[m / 3] + (m % 3 + 1) as i32) % 4;
        }
        assert_eq!(final_c, [0; 6]);
        if moves.len() > max.0 {
            max = (moves.len(), centers);
        }
    }
    println!(
        "Center configurations checked=2048 max_correction_moves={} centers={:?}",
        max.0, max.1
    );
    let documented_bound_case =
        cube_studio::solve_state_with_algorithm(cube::SOLVED, 5000, true, Some([1; 6]), "cfop")
            .unwrap();
    println!(
        "Public API SOLVED + centers=[1;6] cfop returned_moves={} final_state_solved={}",
        documented_bound_case.moves.len(),
        documented_bound_case.state == cube::SOLVED
    );
    for seed in 1..=100 {
        let c = cube::apply(&RawCube::default(), &cube::scramble(seed));
        match cube_studio::cfop::solve(&c, 1000) {
            Ok(r) => assert_eq!(cube::apply(&c, &r.moves), RawCube::default()),
            Err(e) => {
                println!(
                    "CFOP_SCRAMBLE_FAIL seed={seed} state={} err={e}",
                    cube::facelets(&c)
                );
                return;
            }
        }
    }
    println!("CFOP scramble samples passed=100");
    let mut thistle_count = 0;
    let mut korf_count = 0;
    let mut kociemba_count = 0;
    for seed in 1..=20 {
        let full = cube::scramble(seed);
        let c = cube::apply(&RawCube::default(), &full[..4]);
        for alg in ["kociemba", "korf", "thistlethwaite"] {
            let state = cube::facelets(&c);
            let r =
                cube_studio::solve_state_with_algorithm(&state, 1000, false, None, alg).unwrap();
            let m = cube::parse_moves(&r.moves.join(" ")).unwrap();
            assert_eq!(cube::apply(&c, &m), RawCube::default());
            let mut end = 0;
            for phase in &r.phases {
                assert_eq!(phase.start, end);
                assert!(phase.end <= m.len());
                end = phase.end;
                if phase.name == "Kociemba Phase 1 (G1縮約)" {
                    let intermediate = cube::apply(&c, &m[..end]);
                    assert_eq!(
                        (
                            intermediate.get_twist(),
                            intermediate.get_flip(),
                            intermediate.get_ud_slice()
                        ),
                        (0, 0, 0)
                    );
                }
                if phase.name == "Korf 最短探索 (IDA*)" {
                    assert!(m.len() <= 4);
                }
            }
            assert_eq!(end, m.len());
            match alg {
                "kociemba" => kociemba_count += 1,
                "korf" => korf_count += 1,
                _ => thistle_count += 1,
            }
        }
    }
    println!("Short-scramble correctness and phase continuity: kociemba={kociemba_count} korf={korf_count} thistlethwaite={thistle_count}");
    for (set, upper) in [
        ("twist", 2187),
        ("flip", 2048),
        ("slice", 495),
        ("cp", 40320),
        ("ep8", 40320),
        ("sp", 24),
    ] {
        for code in 0..upper {
            let mut c = RawCube::default();
            match set {
                "twist" => {
                    c.set_twist(code);
                    assert_eq!(c.get_twist(), code);
                }
                "flip" => {
                    c.set_flip(code);
                    assert_eq!(c.get_flip(), code);
                }
                "slice" => {
                    c.set_ud_slice(code);
                    assert_eq!(c.get_ud_slice(), code);
                }
                "cp" => {
                    c.set_cp(code);
                    assert_eq!(c.get_cp(), code);
                }
                "ep8" => {
                    c.set_ep8(code);
                    assert_eq!(c.get_ep8(), code);
                }
                _ => {
                    c.set_slice_p(code);
                    assert_eq!(c.get_slice_p(), code);
                }
            }
        }
    }
    println!("All coordinate encode/decode values passed: 2187+2048+495+40320+40320+24");
    let seq = cube::parse_moves("R U F").unwrap();
    let c = cube::apply(&RawCube::default(), &seq);
    for alg in ["kociemba", "cfop", "thistlethwaite", "korf"] {
        let begin = std::time::Instant::now();
        let res = cube_studio::solve_state_with_algorithm(&cube::facelets(&c), 0, false, None, alg);
        println!(
            "Zero budget alg={alg} ok={} elapsed_ms={:.3}",
            res.is_ok(),
            begin.elapsed().as_secs_f64() * 1000.0
        );
    }
}
