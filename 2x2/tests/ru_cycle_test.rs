use rubiks_cube_2x2::cube::{Cube, Move};
use rubiks_cube_2x2::solver;

#[test]
fn test_ru_cycle() {
    // R U の繰り返しの周期性を確認
    let mut cube = Cube::new();

    println!("=== R U の周期性テスト ===");

    for cycle in 1..=12 {
        cube.apply_move(Move::R);
        cube.apply_move(Move::U);

        if cube.is_solved() {
            println!("✓ R U を{}回繰り返すと完成状態に戻ります", cycle);
        }

        let solution = solver::solve(&cube, solver::DEFAULT_MAX_DEPTH, true);
        println!("{}回後: 解法手数 = {} 手", cycle, solution.moves.len());
    }
}
