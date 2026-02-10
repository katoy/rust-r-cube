use rubiks_cube_3x3::cube::Cube;
use rubiks_cube_3x3::solver::solve;
use std::fs;
use std::time::Instant;

fn main() {
    let cube_text = fs::read_to_string("cubes/cube_god.txt").unwrap();
    let cube = Cube::from_file_format(&cube_text).unwrap();

    println!("=== Searching for Color-Only Solution for God Cube ===");
    println!("Using high node limit from the start.");

    // solver.rs に追加した段階的引き上げロジックを待つのではなく、
    // ここで直接重い探索を試みる。
    // ただし、現在の solve インターフェースでは外部からノード制限を指定できないため、
    // solve_internal の trial 0 から重くなるように solver.rs を微調整した後の状態を想定するか、
    // あるいは solve を呼び出す回数を調整する。

    let start = Instant::now();
    // 深度を 20 に固定（God's Number）
    let solution = solve(&cube, 20, true);
    let duration = start.elapsed();

    println!("Time: {:?}", duration);
    if solution.found {
        println!("Solution found! ({} moves)", solution.moves.len());
        println!("Moves: {:?}", solution.moves);
    } else {
        println!("Solution not found.");
        println!("Message: {}", solution.message);
    }
}
